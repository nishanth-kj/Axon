pub mod logger;
pub mod node;
pub mod proxy;

pub fn run_core_logic() {
    log::info!("Executing core logic...");
    log::debug!("This is a debug message from the core library.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_connection() {
        // Initialize the logger so we can see output during the test
        let _ = env_logger::builder().is_test(true).try_init();
        
        log::info!("Starting proxy connection test...");
        
        // Call the proxy test function we implemented
        proxy::client::test_random_proxy_request().await;
    }
}
