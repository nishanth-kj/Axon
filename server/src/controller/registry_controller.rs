use crate::controller::{bearer_token, request_ip};
use crate::model::request::RegistryRequest;
use crate::model::response;
use crate::service::{AppState, RegisterOutcome};
use crate::utils::logger;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

pub async fn registry(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<RegistryRequest>,
) -> HttpResponse {
    if bearer_token(&req).as_deref() != Some(state.master_secret.as_str()) {
        return response::failure(403, "Unauthorized registry access", Some("authorization"), 401);
    }

    let ip = request_ip(&req);
    match state.nodes.apply_registration(&body, &ip) {
        RegisterOutcome::InvalidPayload => {
            response::failure(400, "Invalid request payload", None, 400)
        }
        RegisterOutcome::MissingNodeId => response::failure(
            400,
            "node_id is required for unregistering",
            Some("node_id"),
            400,
        ),
        RegisterOutcome::InvalidStatus => {
            response::failure(400, "Invalid status value", Some("status"), 400)
        }
        RegisterOutcome::Unregistered => response::success(json!("Unregistered successfully"), 200),
        RegisterOutcome::Pending(node) => {
            logger::info(format!(
                "Node {} pending at {}:{} (verify {})",
                node.node_id, node.ip, node.port, node.verification_port
            ));
            state.nodes.spawn_verification(&node);
            response::success(
                json!({
                    "message": "Registration pending",
                    "node_id": node.node_id,
                }),
                202,
            )
        }
    }
}
