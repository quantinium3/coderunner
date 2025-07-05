use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;
use tracing;

use crate::infra::error::InfraError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not Acceptable: {0}")]
    NotAcceptible(String),

    #[error("Not Acceptable: {0}")]
    InternalServerError(#[from] InfraError)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("API Error: {}", self);

        let (status, err_msg) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, format!("Not found: {}", msg)),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, format!("Bad request: {}", msg)),
            Self::ValidationError(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid input: {}", err))
            }
            Self::NotAcceptible(msg) => (
                StatusCode::NOT_ACCEPTABLE,
                format!("Not Acceptable: {}", msg),
            ),
            Self::InternalServerError(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", err))
            }
        };

        (status, Json(json!({ "message": err_msg }))).into_response()
    }
}
