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
