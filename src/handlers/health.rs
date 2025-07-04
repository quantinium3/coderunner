use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    status: &'static str,
}

pub async fn healthz() -> impl IntoResponse {
    let status = Status { status: "Ok" };

    Json(status)
}
