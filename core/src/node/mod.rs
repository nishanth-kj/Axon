use axum::{
    routing::{post, get},
    Router, Json,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use crate::proxy;

#[derive(Deserialize, Debug)]
struct VerifyPayload {
    #[serde(rename = "secretKey")]
    secret_key: String,
}

#[derive(Serialize)]
struct RegistryPayload {
    status: u8,
    port: u16,
    verification_port: u16,
}

struct AppState {
    secret_key: Option<String>,
    last_health_check: Option<std::time::Instant>,
}

/// Settings for one Axon node. The CLI, desktop app, and Android app all
/// build this before calling [`NodeConfig::run`].
#[derive(Clone)]
pub struct NodeConfig {
    pub registry_url: String,
    pub secret: String,
    pub verify_port: u16,
    pub relay_port: u16,
    pub socks_port: u16,
    pub use_tor: bool,
    pub use_vpn: bool,
}

impl NodeConfig {
    pub fn parse(
        registry_url: &str,
        secret: &str,
        verify_port: u16,
        relay_port: u16,
        socks_port: u16,
        use_tor: bool,
        use_vpn: bool,
    ) -> Result<Self, String> {
        let registry_url = registry_url.trim().trim_end_matches('/').to_string();
        if !(registry_url.starts_with("http://") || registry_url.starts_with("https://")) {
            return Err("Registry URL must start with http:// or https://.".into());
        }
        let secret = secret.trim().to_string();
        if secret.is_empty() {
            return Err(
                "Master secret is required. It must match the registry SECRET_KEY.".into(),
            );
        }
        for (name, port) in [
            ("Verify", verify_port),
            ("Relay", relay_port),
            ("SOCKS", socks_port),
        ] {
            if port == 0 {
                return Err(format!(
                    "{name} port must be a number from 1 to 65535."
                ));
            }
        }
        if verify_port == relay_port || verify_port == socks_port || relay_port == socks_port {
            return Err("Verify, relay, and SOCKS ports must be different.".into());
        }
        Ok(Self {
            registry_url,
            secret,
            verify_port,
            relay_port,
            socks_port,
            use_tor,
            use_vpn,
        })
    }

    pub async fn run(self) {
        run_node(
            &self.registry_url,
            self.verify_port,
            self.relay_port,
            self.socks_port,
            &self.secret,
            self.use_tor,
            self.use_vpn,
        )
        .await
    }
}

pub async fn run_node(server_url: &str, axum_port: u16, server_port: u16, client_port: u16, master_secret: &str, use_tor: bool, use_vpn: bool) {
    let state = Arc::new(Mutex::new(AppState { secret_key: None, last_health_check: None }));
    let state_clone = state.clone();

    // Create HTTP Client (with optional Tor SOCKS proxy)
    let client = if use_tor {
        log::info!("Configuring Tor SOCKS proxy for outbound requests...");
        reqwest::Client::builder()
            .proxy(reqwest::Proxy::all("socks5h://127.0.0.1:9050").expect("Failed to configure Tor proxy"))
            .build()
            .unwrap_or_default()
    } else {
        reqwest::Client::new()
    };

    let client_clone = client.clone();

    // Axum router for the verification callback
    let app = Router::new()
        .route("/api/v1/verify", post({
            let state = state.clone();
            move |Json(payload): Json<VerifyPayload>| {
                let state = state.clone();
                async move {
                    log::info!("Received verification callback with secretKey!");
                    let mut s = state.lock().await;
                    s.secret_key = Some(payload.secret_key);
                    s.last_health_check = Some(std::time::Instant::now());
                    "OK"
                }
            }
        }))
        .route("/api/v1/health", get({
            let state = state.clone();
            move || {
                let state = state.clone();
                async move {
                    let mut s = state.lock().await;
                    s.last_health_check = Some(std::time::Instant::now());
                    "OK"
                }
            }
        }));

    // Start background task to register and then heartbeat
    let server_url_bg = server_url.to_string();
    let master_secret_bg = master_secret.to_string();
    tokio::spawn(async move {
        // Give the local server a moment to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        loop {
            let (secret_key, _last_check) = {
                let s = state_clone.lock().await;
                (s.secret_key.clone(), s.last_health_check)
            };

            if let Some(secret) = &secret_key {
                let heartbeat_url = format!("{}/api/v1/heartbeat", server_url_bg);
                let res = client_clone.post(&heartbeat_url)
                    .header("Authorization", format!("Token {}", secret))
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            log::warn!("Heartbeat failed ({}). Clearing secret key to re-register...", resp.status());
                            let mut s = state_clone.lock().await;
                            s.secret_key = None;
                            s.last_health_check = None;
                        } else {
                            log::debug!("Heartbeat successful!");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to send heartbeat to server: {}", e);
                    }
                }

            } else {
                log::info!("No active secret key. Registering with server {}...", server_url_bg);
                let registry_url = format!("{}/api/v1/registry", server_url_bg);
                let res = client_clone.post(&registry_url)
                    .header("Authorization", format!("Token {}", master_secret_bg))
                    .json(&RegistryPayload { status: 1, port: server_port, verification_port: axum_port })
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            log::info!("Registration pending! Waiting for verification callback...");
                            let mut s = state_clone.lock().await;
                            s.last_health_check = Some(std::time::Instant::now());
                        } else {
                            log::error!("Registration failed: {:?}", resp.status());
                        }
                    }
                    Err(e) => log::error!("Failed to reach server for registration: {}", e),
                }
            }

            let interval = std::env::var("HEARTBEAT_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or_else(|| if cfg!(debug_assertions) { 5 } else { 30 });
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });

    // Start SOCKS5 Server Proxy (for incoming P2P traffic)
    tokio::spawn(async move {
        proxy::server::start_socks5_server(server_port, use_tor).await;
    });

    // Start SOCKS5 Client Proxy (for local user traffic)
    let server_url_client = server_url.to_string();
    tokio::spawn(async move {
        proxy::client::start_client_proxy(client_port, &server_url_client).await;
    });

    if use_vpn {
        let server_url_vpn = server_url.to_string();
        tokio::spawn(async move {
            crate::vpn::tun_adapter::start_vpn_service(server_url_vpn).await;
        });
    }

    // Start local Axum server
    let addr = format!("0.0.0.0:{}", axum_port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            log::error!("Failed to bind verification listener on {addr}: {error}");
            return;
        }
    };
    log::info!("Node verification callback listening on {}", addr);
    if let Err(error) = axum::serve(listener, app).await {
        log::error!("Verification listener stopped: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::NodeConfig;

    #[test]
    fn rejects_a_registry_without_a_scheme() {
        let error = match NodeConfig::parse(
            "127.0.0.1:7400",
            "secret",
            8080,
            8081,
            1080,
            false,
            false,
        ) {
            Err(error) => error,
            Ok(_) => panic!("a registry without a scheme should be rejected"),
        };
        assert!(error.contains("http"));
    }

    #[test]
    fn rejects_duplicate_ports() {
        assert!(
            NodeConfig::parse("http://127.0.0.1:7400", "secret", 8080, 8080, 1080, false, false)
                .is_err()
        );
    }

    #[test]
    fn trims_the_registry_url_and_secret() {
        let config = NodeConfig::parse(
            " http://127.0.0.1:7400/ ",
            " secret ",
            8080,
            8081,
            1080,
            true,
            false,
        )
        .unwrap();
        assert_eq!(config.registry_url, "http://127.0.0.1:7400");
        assert_eq!(config.secret, "secret");
        assert!(config.use_tor);
        assert!(!config.use_vpn);
    }
}
