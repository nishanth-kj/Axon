use axon_core::logger;

#[tokio::main]
async fn main() {
    logger::init();
    println!("CLI starting up...");
    
    // Server URL, Axum Port, Server Port, Client Port, Secret Key, Use Tor, Use VPN
    axon_core::node::run_node("http://127.0.0.1:3000", 8080, 8081, 1080, "Nishanth", false, false).await;
}
