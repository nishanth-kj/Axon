use tokio::net::{TcpListener, TcpStream};

use serde::Deserialize;
use reqwest::Client;

#[derive(Deserialize, Debug, Clone)]
pub struct ActiveNode {
    pub node_id: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Deserialize, Debug)]
struct NodesResponse {
    nodes: Vec<ActiveNode>,
}

pub async fn start_client_proxy(local_port: u16, registry_url: &str) {
    let addr = format!("127.0.0.1:{}", local_port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind Client SOCKS5 server");
    log::info!("Local Client Proxy listening on {}", addr);

    let client = Client::new();
    let registry_url = registry_url.to_string();

    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let client_clone = client.clone();
            let url_clone = registry_url.clone();
            
            tokio::spawn(async move {
                // Fetch active nodes
                let nodes_url = format!("{}/api/v1/nodes", url_clone);
                let nodes_result = client_clone.get(&nodes_url).send().await;
                
                if let Ok(resp) = nodes_result {
                    if let Ok(json) = resp.json::<NodesResponse>().await {
                        if json.nodes.is_empty() {
                            log::warn!("No active nodes available in the Axon network.");
                            return;
                        }
                        
                        // Pick a random node (For simplicity, just pick the first one right now)
                        let target_node = &json.nodes[0];
                        log::info!("Routing local request through Axon peer: {}:{}", target_node.ip, target_node.port);
                        
                        // Forward to remote peer
                        if let Ok(mut remote_stream) = TcpStream::connect(format!("{}:{}", target_node.ip, target_node.port)).await {
                            let (mut client_read, mut client_write) = stream.split();
                            let (mut remote_read, mut remote_write) = remote_stream.split();

                            let c_to_r = tokio::io::copy(&mut client_read, &mut remote_write);
                            let r_to_c = tokio::io::copy(&mut remote_read, &mut client_write);

                            let _ = tokio::try_join!(c_to_r, r_to_c);
                        } else {
                            log::error!("Failed to connect to Axon peer.");
                        }
                    }
                }
            });
        }
    }
}

pub async fn test_random_proxy_request() {
    let registry_url = "http://localhost:3000";
    log::info!("Fetching proxy list from {}", registry_url);

    let client = Client::new();
    let nodes_url = format!("{}/api/v1/nodes", registry_url);
    
    if let Ok(resp) = client.get(&nodes_url).send().await {
        if let Ok(json) = resp.json::<NodesResponse>().await {
            if json.nodes.is_empty() {
                log::warn!("Test: No proxies available.");
                return;
            }

            // Pick random proxy (first one for now)
            let proxy_node = &json.nodes[0];
            let proxy_url = format!("socks5h://{}:{}", proxy_node.ip, proxy_node.port);
            log::info!("Test: Connecting through random proxy: {}", proxy_url);

            // Connect using reqwest configured with the remote proxy
            let proxied_client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).unwrap())
                .build()
                .unwrap();

            log::info!("Test: Performing request to http://api.ipify.org to check IP...");
            match proxied_client.get("http://api.ipify.org").send().await {
                Ok(resp) => {
                    let body = resp.text().await.unwrap_or_default();
                    log::info!("Test Success! IP returned via proxy: {}", body);
                }
                Err(e) => {
                    log::error!("Test Request Failed: {}", e);
                }
            }
        }
    } else {
        log::error!("Test: Failed to fetch nodes from registry.");
    }
}

pub async fn route_vpn_stream(vpn_stream: ipstack::IpStackTcpStream, target: std::net::SocketAddr, registry_url: &str) {
    let client = Client::new();
    let nodes_url = format!("{}/api/v1/nodes", registry_url);
    
    if let Ok(resp) = client.get(&nodes_url).send().await {
        if let Ok(json) = resp.json::<NodesResponse>().await {
            if json.nodes.is_empty() {
                log::warn!("No active nodes available in the Axon network for VPN route.");
                return;
            }
            
            // Pick a random node
            let target_node = &json.nodes[0];
            log::info!("VPN routing request for {} through Axon peer: {}:{}", target, target_node.ip, target_node.port);
            
            if let Ok(mut remote_stream) = TcpStream::connect(format!("{}:{}", target_node.ip, target_node.port)).await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                // Synthesize SOCKS5 handshake (No Auth)
                let _ = remote_stream.write_all(&[0x05, 0x01, 0x00]).await;
                let mut reply = [0u8; 2];
                let _ = remote_stream.read_exact(&mut reply).await;
                
                if reply[0] != 0x05 || reply[1] != 0x00 {
                    log::error!("Remote peer rejected SOCKS5 handshake");
                    return;
                }
                
                // Synthesize SOCKS5 CONNECT request
                let mut connect_req = vec![0x05, 0x01, 0x00];
                match target {
                    std::net::SocketAddr::V4(v4) => {
                        connect_req.push(0x01); // IPv4
                        connect_req.extend_from_slice(&v4.ip().octets());
                        connect_req.extend_from_slice(&v4.port().to_be_bytes());
                    }
                    std::net::SocketAddr::V6(v6) => {
                        connect_req.push(0x04); // IPv6
                        connect_req.extend_from_slice(&v6.ip().octets());
                        connect_req.extend_from_slice(&v6.port().to_be_bytes());
                    }
                }
                
                let _ = remote_stream.write_all(&connect_req).await;
                
                // Read CONNECT reply. It starts with [0x05, rep, rsv, atyp, ...]
                let mut connect_reply_header = [0u8; 4];
                let _ = remote_stream.read_exact(&mut connect_reply_header).await;
                
                if connect_reply_header[1] != 0x00 {
                    log::error!("Remote peer failed to connect to target");
                    return;
                }
                
                // Consume the rest of the bind address from the reply
                match connect_reply_header[3] {
                    0x01 => { // IPv4 + Port (4 + 2 bytes)
                        let mut remainder = [0u8; 6];
                        let _ = remote_stream.read_exact(&mut remainder).await;
                    }
                    0x03 => { // Domain Name + Port
                        let mut len_buf = [0u8; 1];
                        let _ = remote_stream.read_exact(&mut len_buf).await;
                        let mut remainder = vec![0u8; len_buf[0] as usize + 2];
                        let _ = remote_stream.read_exact(&mut remainder).await;
                    }
                    0x04 => { // IPv6 + Port (16 + 2 bytes)
                        let mut remainder = [0u8; 18];
                        let _ = remote_stream.read_exact(&mut remainder).await;
                    }
                    _ => {
                        log::error!("Unknown ATYP in SOCKS5 reply");
                        return;
                    }
                }
                
                // The SOCKS5 tunnel is established! Now pipe the VPN stream into the remote peer.
                let (mut vpn_read, mut vpn_write) = tokio::io::split(vpn_stream);
                let (mut remote_read, mut remote_write) = remote_stream.split();

                let c_to_r = tokio::io::copy(&mut vpn_read, &mut remote_write);
                let r_to_c = tokio::io::copy(&mut remote_read, &mut vpn_write);

                let _ = tokio::try_join!(c_to_r, r_to_c);
            } else {
                log::error!("Failed to connect to Axon peer.");
            }
        }
    }
}
