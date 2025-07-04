use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

pub fn app_router() -> Router {
    Router::new().route("/", get(root)).fallback(handler_404)
}

async fn root() -> &'static str {
    "hello world"
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The Requested resource was not found",
    )
}
