use reqwest::Client;

pub enum TorClientImpl {
    Proxied(Client),
    Embedded,
}

pub struct TorClient {
    inner: TorClientImpl,
}

impl TorClient {
    /// Creates a new TorClient.
    /// If `proxy_url` is provided, it routes traffic through the specified proxy.
    /// If `proxy_url` is `None`, it creates an embedded Tor client to connect directly to the Tor network.
    pub fn new(proxy_url: Option<&str>) -> Self {
        if let Some(url) = proxy_url {
            let proxy = reqwest::Proxy::all(url)
                .expect("Failed to configure Tor proxy");
            let client = Client::builder()
                .proxy(proxy)
                .build()
                .expect("Failed to build Tor HTTP client");
            Self { inner: TorClientImpl::Proxied(client) }
        } else {
            Self { inner: TorClientImpl::Embedded }
        }
    }

    /// Forwards a GET request through the Tor network and returns the response.
    pub async fn forward_get(&self, url: &str) -> Result<String, String> {
        match &self.inner {
            TorClientImpl::Proxied(client) => {
                let response = client.get(url).send().await.map_err(|e| e.to_string())?;
                let body = response.text().await.map_err(|e| e.to_string())?;
                Ok(body)
            }
            TorClientImpl::Embedded => {
                let response = artiqwest::get(url, None, None).await.map_err(|e| e.to_string())?;
                Ok(response.to_string())
            }
        }
    }
    
    /// Forwards a POST request with data through the Tor network and returns the response.
    pub async fn forward_post(&self, url: &str, body_data: String) -> Result<String, String> {
        match &self.inner {
            TorClientImpl::Proxied(client) => {
                let response = client.post(url)
                    .body(body_data)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                let body = response.text().await.map_err(|e| e.to_string())?;
                Ok(body)
            }
            TorClientImpl::Embedded => {
                let response = artiqwest::post(url, &body_data, None, None).await.map_err(|e| e.to_string())?;
                Ok(response.to_string())
            }
        }
    }
}
