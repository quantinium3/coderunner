use std::str::FromStr;

use crate::infra::{compile::compile_lang, error::InfraError};
use axum::Json;
use serde::{Deserialize, Serialize};

use super::error::ApiError;

#[derive(Serialize)]
pub struct CompilerResponse {
    result: String,
}

#[derive(Deserialize)]
pub struct CompilerRequest {
    lang: String,
    content: String,
    #[serde(default)]
    stdin: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Language {
    Python,
    JAVASCRIPT,
    TYPESCRIPT,
    C,
    CPP,
    RUST,
    NIX,
    GO,
    ZIG,
    D,
    SCALA,
    GROOVY,
    DART,
    RUBY,
    LUA,
    JULIA,
    R,
}

impl FromStr for Language {
    type Err = InfraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "python" => Ok(Language::Python),
            "javascript" => Ok(Language::JAVASCRIPT),
            "typescript" => Ok(Language::TYPESCRIPT),
            "c" => Ok(Language::C),
            "cpp" => Ok(Language::CPP),
            "rust" => Ok(Language::RUST),
            "nix" => Ok(Language::NIX),
            "go" => Ok(Language::GO),
            "zig" => Ok(Language::ZIG),
            "d" => Ok(Language::D),
            "scala" => Ok(Language::SCALA),
            "groovy" => Ok(Language::GROOVY),
            "dart" => Ok(Language::DART),
            "ruby" => Ok(Language::RUBY),
            "lua" => Ok(Language::LUA),
            "julia" => Ok(Language::JULIA),
            "r" => Ok(Language::R),
            _ => Err(InfraError::UnsupportedLanguage(
                format!("{} language is not supported", s).into(),
            )),
        }
    }
}

pub async fn compile(
    Json(payload): Json<CompilerRequest>,
) -> Result<Json<CompilerResponse>, ApiError> {
    payload.lang.parse::<Language>()?;
    let res = compile_lang(&payload.lang, &payload.content, &payload.stdin).await?;

    Ok(Json(CompilerResponse {
        result: res.to_string(),
    }))
}
