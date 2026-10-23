use server::controller;
use server::model::custom::config::ServiceConfig;
use server::service::AppState;
use server::utils::logger;
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use std::io::{Read, Write};
use std::time::Duration;

struct Config {
    bind: String,
    master_secret: String,
    debug: bool,
    failure_limit: u32,
    health_interval: Duration,
}

fn config_from_env() -> Config {
    let bind = std::env::var("AXON_BIND").unwrap_or_else(|_| "0.0.0.0:7400".into());
    let master_secret = std::env::var("SECRET_KEY").unwrap_or_else(|_| "Nishanth".into());
    let debug = matches!(
        std::env::var("DEBUG").ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes")
    );
    let failure_limit = std::env::var("HEALTH_FAILS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &u32| *value > 0)
        .unwrap_or(3);
    let health_secs = std::env::var("HEALTH_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &u64| *value > 0)
        .unwrap_or(10);
    Config {
        bind,
        master_secret,
        debug,
        failure_limit,
        health_interval: Duration::from_secs(health_secs),
    }
}

fn healthcheck(bind: &str) -> bool {
    let port = bind.rsplit(':').next().unwrap_or("7400");
    let address = format!("127.0.0.1:{port}");
    let mut stream = match std::net::TcpStream::connect(address) {
        Ok(stream) => stream,
        Err(_) => return false,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    if stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut buf = [0u8; 128];
    match stream.read(&mut buf) {
        Ok(size) => std::str::from_utf8(&buf[..size])
            .unwrap_or("")
            .contains("200"),
        Err(_) => false,
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config_from_env();
    if std::env::args().skip(1).any(|arg| arg == "--healthcheck") {
        std::process::exit(if healthcheck(&config.bind) { 0 } else { 1 });
    }

    logger::init();
    if config.master_secret == "Nishanth" {
        logger::warn("SECRET_KEY is the development default. Set SECRET_KEY before a public deployment.");
    }
    logger::info(format!(
        "Axon registry listening on {} (debug={})",
        config.bind, config.debug
    ));

    let state = web::Data::new(AppState::new(ServiceConfig {
        master_secret: config.master_secret.clone(),
        debug: config.debug,
        failure_limit: config.failure_limit,
    }));

    let health_state = state.clone();
    let health_interval = config.health_interval;
    actix_web::rt::spawn(async move {
        let mut ticker = actix_web::rt::time::interval(health_interval);
        loop {
            ticker.tick().await;
            health_state.nodes.probe_health().await;
        }
    });

    let bind = config.bind.clone();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(state.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(65_536)
                    .error_handler(controller::json_error),
            )
            .configure(controller::configure)
    })
    .bind(bind)?
    .run()
    .await
}
