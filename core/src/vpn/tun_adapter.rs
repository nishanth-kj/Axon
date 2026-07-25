use tun::Configuration;
use log::{info, error};

pub async fn start_vpn_service(server_url: String) {
    info!("Starting Axon Transparent VPN Adapter...");

    let mut config = Configuration::default();
    config.address((10, 0, 0, 2))
          .netmask((255, 255, 255, 0))
          .tun_name("AxonVPN")
          .up();

    match tun::create(&config) {
        Ok(dev) => {
            info!("TUN Adapter 'AxonVPN' created successfully!");
            let async_dev = tun::AsyncDevice::new(dev).unwrap();
            super::ipstack_handler::handle_ipstack(async_dev, server_url).await;
        }
        Err(e) => {
            error!("Failed to create TUN device. Ensure you run as Administrator and wintun.dll is present. Error: {}", e);
        }
    }
}
