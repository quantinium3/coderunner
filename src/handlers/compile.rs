use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing;

#[derive(Serialize)]
struct CompilerResponse {
    lang: String,
}

#[derive(Deserialize)]
pub struct CompilerRequest {
    lang: String,
}

pub async fn compile(Json(payload): Json<CompilerRequest>) -> impl IntoResponse {
    tracing::debug!("lang: {}", payload.lang);

    let res = CompilerResponse { lang: payload.lang };
    Json(res)
}
