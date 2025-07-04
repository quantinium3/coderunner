use axum::{Router, http::StatusCode, response::IntoResponse, routing::{get, post}};

use crate::handlers::{compile::compile, health::healthz};

pub fn app_router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/compile", post(compile))
        .fallback(handler_404)
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The Requested resource was not found",
    )
}
