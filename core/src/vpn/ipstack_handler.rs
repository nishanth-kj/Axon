use ipstack::{IpStack, IpStackConfig, IpStackStream};
use tun::AsyncDevice;
use log::info;
use std::net::SocketAddr;

pub async fn handle_ipstack(dev: AsyncDevice, _server_url: String) {
    info!("Initializing userspace TCP/IP stack...");
    
    let ipstack_config = IpStackConfig::default();
    let mut ip_stack = IpStack::new(ipstack_config, dev);

    info!("VPN Stack ready! Listening for packets...");

    while let Ok(stream) = ip_stack.accept().await {
        match stream {
            IpStackStream::Tcp(tcp) => {
                let target_addr: SocketAddr = tcp.peer_addr();
                info!("Intercepted TCP stream bound for {}", target_addr);

                let server_url_clone = _server_url.clone();
                tokio::spawn(async move {
                    crate::proxy::client::route_vpn_stream(tcp, target_addr, &server_url_clone).await;
                });
            }
            IpStackStream::Udp(_) => {
                // Not supported yet
            }
            _ => {
                // Ignore unknown packets
            }
        }
    }
}
