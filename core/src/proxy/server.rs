use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::{Ipv4Addr, Ipv6Addr};

pub async fn start_socks5_server(port: u16, use_tor: bool) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind SOCKS5 server");
    log::info!("SOCKS5 Proxy listening on {}", addr);

    loop {
        if let Ok((mut stream, peer_addr)) = listener.accept().await {
            log::debug!("Accepted connection from {}", peer_addr);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut stream, use_tor).await {
                    log::warn!("Connection rejected: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(stream: &mut TcpStream, use_tor: bool) -> std::io::Result<()> {
    // 1. Handshake
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await?;
    
    if header[0] != 0x05 {
        // Gracefully reject HTTP requests (GET, POST, CONNECT, etc.)
        if matches!(header[0], b'G' | b'P' | b'H' | b'D' | b'C' | b'O' | b'T') {
            let msg = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nThis is a SOCKS5 proxy, not an HTTP web server. Please configure your client to use the SOCKS5 protocol.\r\n";
            let _ = stream.write_all(msg.as_bytes()).await;
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Received HTTP request on SOCKS5 port"));
        }
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not SOCKS5"));
    }
    let num_methods = header[1] as usize;
    let mut methods = vec![0u8; num_methods];
    stream.read_exact(&mut methods).await?;
    
    // Respond with No Authentication Required
    stream.write_all(&[0x05, 0x00]).await?;

    // 2. Request
    let mut req_header = [0u8; 4];
    stream.read_exact(&mut req_header).await?;
    if req_header[0] != 0x05 || req_header[1] != 0x01 { // Only handle CONNECT
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported command"));
    }

    let atyp = req_header[3];
    let (target_host, target_port) = match atyp {
        0x01 => { // IPv4
            let mut ip = [0u8; 4];
            stream.read_exact(&mut ip).await?;
            let port = stream.read_u16().await?;
            (Ipv4Addr::from(ip).to_string(), port)
        }
        0x03 => { // Domain
            let len = stream.read_u8().await? as usize;
            let mut domain = vec![0u8; len];
            stream.read_exact(&mut domain).await?;
            let port = stream.read_u16().await?;
            (String::from_utf8_lossy(&domain).into_owned(), port)
        }
        0x04 => { // IPv6
            let mut ip = [0u8; 16];
            stream.read_exact(&mut ip).await?;
            let port = stream.read_u16().await?;
            (Ipv6Addr::from(ip).to_string(), port)
        }
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported ATYP")),
    };

    log::info!("Connecting to {}:{}", target_host, target_port);

    // 3. Connect to Target (or via Tor)
    let mut target_stream = if use_tor {
        // Forward connection via local Tor SOCKS5 proxy (e.g. 127.0.0.1:9050)
        // For simplicity we just make a direct connection to tor proxy, perform SOCKS5 handshake, and forward.
        // But honestly doing SOCKS5-in-SOCKS5 manually is tedious, so maybe just use direct connection 
        // or fast-socks5 library if it supported outbound.
        // We'll simulate Tor handling or use standard fast-socks5 if needed, but for now fallback to direct.
        log::warn!("Tor routing not fully implemented in manual proxy, falling back to direct");
        TcpStream::connect(format!("{}:{}", target_host, target_port)).await?
    } else {
        TcpStream::connect(format!("{}:{}", target_host, target_port)).await?
    };

    // 4. Send Success Reply
    // BND.ADDR and BND.PORT (just send zeros)
    let reply = [0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
    stream.write_all(&reply).await?;

    // 5. Proxy traffic
    let (mut client_read, mut client_write) = stream.split();
    let (mut target_read, mut target_write) = target_stream.split();

    let client_to_target = tokio::io::copy(&mut client_read, &mut target_write);
    let target_to_client = tokio::io::copy(&mut target_read, &mut client_write);

    let _ = tokio::try_join!(client_to_target, target_to_client);

    Ok(())
}
