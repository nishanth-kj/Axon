use crate::controller::{bearer_token, request_ip};
use crate::model::custom::node::HeartbeatError;
use crate::model::response;
use crate::service::AppState;
use crate::utils::time::now_iso;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

pub async fn alive() -> HttpResponse {
    response::success(
        json!({
            "status": "alive",
            "timestamp": now_iso(),
        }),
        200,
    )
}

pub async fn heartbeat(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let Some(secret) = bearer_token(&req) else {
        return response::failure(
            401,
            "Missing or invalid Token in Authorization header",
            Some("authorization"),
            401,
        );
    };
    let ip = request_ip(&req);
    match state.nodes.heartbeat(&secret, &ip) {
        Ok(()) => response::success(json!("Proxy node verified successfully"), 200),
        Err(HeartbeatError::UnknownToken) => response::failure(
            401,
            "Node is not registered or session expired",
            Some("secretKey"),
            401,
        ),
        Err(HeartbeatError::IpMismatch) => response::failure(403, "IP mismatch", Some("ip"), 403),
    }
}
