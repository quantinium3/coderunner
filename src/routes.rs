use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

use crate::handlers::health::healthz;

pub fn app_router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .fallback(handler_404)
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The Requested resource was not found",
    )
}
