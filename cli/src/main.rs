use axon_core::logger;

#[tokio::main]
async fn main() {
    logger::init();
    println!("CLI starting up...");
    
    let registry = std::env::var("REGISTRY_URL").unwrap_or_else(|_| "http://127.0.0.1:7400".to_string());
    let secret = std::env::var("SECRET_KEY").unwrap_or_else(|_| "Nishanth".to_string());
    // Server URL, Axum Port, Server Port, Client Port, Secret Key, Use Tor, Use VPN
    axon_core::node::run_node(&registry, 8080, 8081, 1080, &secret, false, false).await;
}
