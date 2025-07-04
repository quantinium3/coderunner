use std::{io, net::AddrParseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("FileSystem error: {0}")]
    FileSystemError(#[from] io::Error),

    #[error("Internal server error: {0}")]
    InternalServerError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to parse socketaddr: {0}")]
    AddrParse(#[from] AddrParseError)
}
