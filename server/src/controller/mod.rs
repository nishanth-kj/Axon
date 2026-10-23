mod heartbeat_controller;
mod health_controller;
mod nodes_controller;
mod registry_controller;

use crate::model::response;
use actix_web::error::JsonPayloadError;
use actix_web::{web, HttpRequest};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_controller::health))
        .service(
            web::scope("/api/v1")
                .route("/registry", web::post().to(registry_controller::registry))
                .route("/heartbeat", web::get().to(heartbeat_controller::alive))
                .route("/heartbeat", web::post().to(heartbeat_controller::heartbeat))
                .route("/nodes", web::get().to(nodes_controller::nodes))
                .route("/nodes/details", web::post().to(nodes_controller::details)),
        );
}

pub fn json_error(err: JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let http = actix_web::error::InternalError::from_response(
        err,
        response::failure(400, "Invalid request payload", None, 400),
    );
    http.into()
}

pub(crate) fn bearer_token(req: &HttpRequest) -> Option<String> {
    let header = req.headers().get("authorization")?.to_str().ok()?;
    let value = header.strip_prefix("Token ")?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub(crate) fn request_ip(req: &HttpRequest) -> String {
    let forwarded = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok());
    let real = req
        .headers()
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok());
    crate::utils::net::client_ip(req.peer_addr(), forwarded, real)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::custom::config::ServiceConfig;
    use crate::service::{secret_for, AppState};
    use actix_web::{http::StatusCode, test, App};
    use serde_json::json;

    fn test_state() -> web::Data<AppState> {
        web::Data::new(AppState::new(ServiceConfig {
            master_secret: "Nishanth".into(),
            debug: false,
            failure_limit: 3,
        }))
    }

    fn peer() -> std::net::SocketAddr {
        "203.0.113.8:50000".parse().unwrap()
    }

    macro_rules! test_app {
        ($state:expr) => {
            test::init_service(
                App::new()
                    .app_data($state)
                    .app_data(web::JsonConfig::default().limit(65_536).error_handler(json_error))
                    .configure(configure),
            )
            .await
        };
    }

    #[actix_web::test]
    async fn registry_rejects_the_wrong_master_secret() {
        let app = test_app!(test_state());
        let req = test::TestRequest::post()
            .uri("/api/v1/registry")
            .peer_addr(peer())
            .insert_header(("authorization", "Token wrong"))
            .set_json(json!({"status": 1, "port": 8081, "verification_port": 8080}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], 0);
        assert_eq!(body["error"]["error_field"], "authorization");
    }

    #[actix_web::test]
    async fn register_heartbeat_and_discovery() {
        let app = test_app!(test_state());
        let register = test::TestRequest::post()
            .uri("/api/v1/registry")
            .peer_addr(peer())
            .insert_header(("authorization", "Token Nishanth"))
            .set_json(json!({
                "status": 1,
                "node_id": "node-1",
                "port": 8081,
                "verification_port": 8080
            }))
            .to_request();
        let registered = test::call_service(&app, register).await;
        assert_eq!(registered.status(), StatusCode::ACCEPTED);
        let body: serde_json::Value = test::read_body_json(registered).await;
        assert_eq!(body["data"]["node_id"], "node-1");
        assert_eq!(body["data"]["message"], "Registration pending");

        let empty = test::TestRequest::get()
            .uri("/api/v1/nodes")
            .to_request();
        let empty_body: serde_json::Value = test::read_body_json(test::call_service(&app, empty).await).await;
        assert_eq!(empty_body["data"]["nodes"].as_array().unwrap().len(), 0);

        let secret = secret_for("Nishanth", "node-1");
        let beat = test::TestRequest::post()
            .uri("/api/v1/heartbeat")
            .peer_addr(peer())
            .insert_header(("authorization", format!("Token {secret}")))
            .to_request();
        let beat_resp = test::call_service(&app, beat).await;
        assert_eq!(beat_resp.status(), StatusCode::OK);

        let listed = test::TestRequest::get()
            .uri("/api/v1/nodes")
            .to_request();
        let listed_body: serde_json::Value =
            test::read_body_json(test::call_service(&app, listed).await).await;
        assert_eq!(listed_body["data"]["nodes"][0]["node_id"], "node-1");
        assert_eq!(listed_body["data"]["nodes"][0]["port"], 8081);
    }

    #[actix_web::test]
    async fn details_do_not_return_the_secret() {
        let app = test_app!(test_state());
        let register = test::TestRequest::post()
            .uri("/api/v1/registry")
            .peer_addr(peer())
            .insert_header(("authorization", "Token Nishanth"))
            .set_json(json!({"status": 1, "node_id": "node-1", "port": 8081}))
            .to_request();
        assert_eq!(test::call_service(&app, register).await.status(), StatusCode::ACCEPTED);

        let details = test::TestRequest::post()
            .uri("/api/v1/nodes/details")
            .set_json(json!({"page": 1, "size": 10, "sortBy": "node_id", "sortOrder": "asc"}))
            .to_request();
        let body: serde_json::Value = test::read_body_json(test::call_service(&app, details).await).await;
        assert_eq!(body["data"]["pagination"]["total"], 1);
        assert!(body["data"]["nodes"][0].get("secret_key").is_none());
        assert_eq!(body["data"]["nodes"][0]["node_id"], "node-1");
    }

    #[actix_web::test]
    async fn invalid_json_uses_the_error_envelope() {
        let app = test_app!(test_state());
        let req = test::TestRequest::post()
            .uri("/api/v1/registry")
            .insert_header(("content-type", "application/json"))
            .set_payload("{")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], 0);
        assert_eq!(body["error"]["error_message"], "Invalid request payload");
    }

    #[actix_web::test]
    async fn health_and_heartbeat_alive_respond() {
        let app = test_app!(test_state());
        let health = test::call_service(
            &app,
            test::TestRequest::get().uri("/health").to_request(),
        )
        .await;
        assert_eq!(health.status(), StatusCode::OK);

        let alive = test::call_service(
            &app,
            test::TestRequest::get().uri("/api/v1/heartbeat").to_request(),
        )
        .await;
        assert_eq!(alive.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(alive).await;
        assert_eq!(body["data"]["status"], "alive");
    }
}
