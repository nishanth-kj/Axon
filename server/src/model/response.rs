use actix_web::HttpResponse;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error_code: u16,
    pub error_message: String,
    pub error_field: Option<String>,
}

pub fn success(data: Value, status: u16) -> HttpResponse {
    HttpResponse::build(status_code(status)).json(json!({
        "status": 1,
        "data": data,
        "error": null
    }))
}

pub fn failure(error_code: u16, message: &str, field: Option<&str>, status: u16) -> HttpResponse {
    let body = ApiErrorBody {
        error_code,
        error_message: message.to_string(),
        error_field: field.map(str::to_string),
    };
    HttpResponse::build(status_code(status)).json(json!({
        "status": 0,
        "data": null,
        "error": body,
    }))
}

fn status_code(status: u16) -> actix_web::http::StatusCode {
    actix_web::http::StatusCode::from_u16(status).unwrap_or(actix_web::http::StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;

    #[actix_web::test]
    async fn success_envelope_uses_status_one() {
        let response = success(json!({"nodes": []}), 200);
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], 1);
        assert!(body["error"].is_null());
        assert_eq!(body["data"]["nodes"], json!([]));
    }

    #[actix_web::test]
    async fn failure_envelope_keeps_the_field() {
        let response = failure(401, "missing token", Some("authorization"), 401);
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], 0);
        assert!(body["data"].is_null());
        assert_eq!(body["error"]["error_code"], 401);
        assert_eq!(body["error"]["error_field"], "authorization");
    }
}
