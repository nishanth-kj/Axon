use axum::{
    routing::post,
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
}

pub async fn run_node(server_url: &str, axum_port: u16, server_port: u16, client_port: u16, master_secret: &str, use_tor: bool) {
    let state = Arc::new(Mutex::new(AppState { secret_key: None }));
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
            let secret_key = {
                let s = state_clone.lock().await;
                s.secret_key.clone()
            };

            if let Some(key) = secret_key {
                log::debug!("Sending heartbeat...");
                let heartbeat_url = format!("{}/api/v1/heartbeat", server_url_bg);
                let hb_res = client_clone.post(&heartbeat_url)
                    .header("Authorization", format!("Token {}", key))
                    .send()
                    .await;

                match hb_res {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            log::warn!("Heartbeat rejected ({}). Clearing secret key to trigger re-registration.", resp.status());
                            let mut s = state_clone.lock().await;
                            s.secret_key = None;
                        } else {
                            log::debug!("Heartbeat acknowledged.");
                        }
                    }
                    Err(e) => log::error!("Heartbeat failed to send: {}", e),
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
                        } else {
                            log::error!("Registration failed: {:?}", resp.status());
                        }
                    }
                    Err(e) => log::error!("Failed to reach server for registration: {}", e),
                }
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
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

    // Start local Axum server
    let addr = format!("0.0.0.0:{}", axum_port);
    log::info!("Node verification callback listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
