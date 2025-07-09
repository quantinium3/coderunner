use axum::{
    Router,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::Method;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{compile::compile, health::healthz};

pub fn app_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/api/v1/healthz", get(healthz))
        .route("/api/v1/compile", post(compile))
        .layer(cors)
        .fallback(handler_404)
}

pub fn test_router() -> Router {
    app_router()
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The Requested resource was not found",
    )
}
