use crate::model::request::DetailsRequest;
use crate::model::response;
use crate::service::AppState;
use actix_web::{web, HttpResponse};
use serde_json::json;

pub async fn nodes(state: web::Data<AppState>) -> HttpResponse {
    response::success(json!({ "nodes": state.nodes.active_nodes() }), 200)
}

pub async fn details(
    state: web::Data<AppState>,
    body: Option<web::Json<DetailsRequest>>,
) -> HttpResponse {
    let query = body.map(|body| body.into_inner()).unwrap_or_default();
    let page = state.nodes.list_details(&query);
    response::success(
        json!({
            "nodes": page.nodes,
            "pagination": {
                "total": page.total,
                "page": page.page,
                "size": page.size,
                "totalPages": page.total_pages,
            }
        }),
        200,
    )
}
