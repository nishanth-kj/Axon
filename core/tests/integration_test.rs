use axon_core::proxy::client::test_random_proxy_request;

#[tokio::test]
async fn test_proxy_routing() {
    // This is an integration test to ensure the proxy logic is valid.
    // It will attempt to connect to the node and perform a request.
    test_random_proxy_request().await;
}
