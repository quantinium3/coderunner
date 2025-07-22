use thiserror::Error;

#[derive(Error, Debug)]
pub enum InfraError {
    #[error("Compilation failed: {0}")]
    CompilationError(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("Failed to convert string: {0}")]
    StringParseError(#[from] std::string::FromUtf8Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to find the binary: {0}")]
    CompilerNotFound(#[from] which::Error),
}
