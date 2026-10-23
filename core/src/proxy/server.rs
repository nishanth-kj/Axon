use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn start_socks5_server(port: u16, use_tor: bool) {
    let addr = format!("0.0.0.0:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            log::error!("Failed to bind SOCKS5 relay on {addr}: {error}");
            return;
        }
    };
    log::info!("SOCKS5 Proxy listening on {addr}");

    loop {
        match listener.accept().await {
            Ok((mut stream, peer_addr)) => {
                log::debug!("Accepted connection from {peer_addr}");
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(&mut stream, use_tor).await {
                        log::warn!("Connection from {peer_addr} ended: {error}");
                    }
                });
            }
            Err(error) => log::warn!("Failed to accept SOCKS5 connection: {error}"),
        }
    }
}

async fn handle_connection(stream: &mut TcpStream, use_tor: bool) -> std::io::Result<()> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await?;

    if header[0] != 0x05 {
        if matches!(header[0], b'G' | b'P' | b'H' | b'D' | b'C' | b'O' | b'T') {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nThis is a SOCKS5 proxy. Configure your client to use SOCKS5.\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected SOCKS5 handshake",
        ));
    }

    let mut methods = vec![0u8; header[1] as usize];
    stream.read_exact(&mut methods).await?;
    if !methods.contains(&0x00) {
        stream.write_all(&[0x05, 0xFF]).await?;
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Client does not offer no-authentication mode",
        ));
    }
    stream.write_all(&[0x05, 0x00]).await?;

    let mut request = [0u8; 4];
    stream.read_exact(&mut request).await?;
    if request[0] != 0x05 || request[1] != 0x01 || request[2] != 0x00 {
        send_failure(stream, 0x07).await?;
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Only SOCKS5 CONNECT is supported",
        ));
    }

    let (host, port) = match request[3] {
        0x01 => {
            let mut ip = [0u8; 4];
            stream.read_exact(&mut ip).await?;
            (Ipv4Addr::from(ip).to_string(), stream.read_u16().await?)
        }
        0x03 => {
            let length = stream.read_u8().await? as usize;
            if length == 0 {
                send_failure(stream, 0x08).await?;
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Empty hostname"));
            }
            let mut domain = vec![0u8; length];
            stream.read_exact(&mut domain).await?;
            let host = String::from_utf8(domain)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            (host, stream.read_u16().await?)
        }
        0x04 => {
            let mut ip = [0u8; 16];
            stream.read_exact(&mut ip).await?;
            (Ipv6Addr::from(ip).to_string(), stream.read_u16().await?)
        }
        _ => {
            send_failure(stream, 0x08).await?;
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unsupported address type"));
        }
    };

    let mut target = if use_tor {
        connect_via_tor(&host, port).await?
    } else {
        match TcpStream::connect((host.as_str(), port)).await {
            Ok(stream) => stream,
            Err(error) => {
                send_failure(stream, 0x05).await?;
                return Err(error);
            }
        }
    };

    stream.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;

    let (mut client_read, mut client_write) = stream.split();
    let (mut target_read, mut target_write) = target.split();
    let client_to_target = tokio::io::copy(&mut client_read, &mut target_write);
    let target_to_client = tokio::io::copy(&mut target_read, &mut client_write);
    let _ = tokio::try_join!(client_to_target, target_to_client);
    Ok(())
}

async fn connect_via_tor(host: &str, port: u16) -> std::io::Result<TcpStream> {
    let mut tor = match TcpStream::connect(("127.0.0.1", 9050)).await {
        Ok(stream) => stream,
        Err(error) => {
            return Err(std::io::Error::new(
                error.kind(),
                format!("Tor SOCKS5 service is unavailable at 127.0.0.1:9050: {error}"),
            ));
        }
    };

    // Negotiate SOCKS5 without authentication with the local Tor daemon.
    tor.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut greeting = [0u8; 2];
    tor.read_exact(&mut greeting).await?;
    if greeting != [0x05, 0x00] {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Tor SOCKS5 service rejected no-authentication mode",
        ));
    }

    // Send the hostname to Tor, allowing Tor to resolve it remotely.
    let host_bytes = host.as_bytes();
    let mut request = Vec::with_capacity(host_bytes.len() + 7);
    request.extend_from_slice(&[0x05, 0x01, 0x00]);
    if let Ok(ipv4) = host.parse::<Ipv4Addr>() {
        request.push(0x01);
        request.extend_from_slice(&ipv4.octets());
    } else if let Ok(ipv6) = host.parse::<Ipv6Addr>() {
        request.push(0x04);
        request.extend_from_slice(&ipv6.octets());
    } else {
        if host_bytes.is_empty() || host_bytes.len() > u8::MAX as usize {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid hostname length"));
        }
        request.push(0x03);
        request.push(host_bytes.len() as u8);
        request.extend_from_slice(host_bytes);
    }
    request.extend_from_slice(&port.to_be_bytes());
    tor.write_all(&request).await?;

    let mut reply = [0u8; 4];
    tor.read_exact(&mut reply).await?;
    if reply[0] != 0x05 || reply[1] != 0x00 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Tor SOCKS5 connection failed with code {}", reply[1]),
        ));
    }

    match reply[3] {
        0x01 => {
            let mut address_and_port = [0u8; 6];
            tor.read_exact(&mut address_and_port).await?;
        }
        0x03 => {
            let length = tor.read_u8().await? as usize;
            let mut address_and_port = vec![0u8; length + 2];
            tor.read_exact(&mut address_and_port).await?;
        }
        0x04 => {
            let mut address_and_port = [0u8; 18];
            tor.read_exact(&mut address_and_port).await?;
        }
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Tor SOCKS5 reply address")),
    }

    Ok(tor)
}

async fn send_failure(stream: &mut TcpStream, code: u8) -> std::io::Result<()> {
    stream.write_all(&[0x05, code, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tor_connection_fails_closed_when_daemon_is_unavailable() {
        // This validates error propagation without requiring a working Tor daemon;
        // if a local Tor daemon is running, the handshake proceeds and reaches connect.
        let result = connect_via_tor("axon-invalid-test-hostname.invalid", 80).await;
        if let Err(error) = result {
            assert!(
                error.to_string().contains("Tor SOCKS5")
                    || error.kind() == std::io::ErrorKind::ConnectionRefused
                    || error.kind() == std::io::ErrorKind::NotFound
            );
        }
    }
}
