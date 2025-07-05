use super::{error::InfraError, python::compile_python};

pub async fn compile_lang(lang: &str, content: &str, stdin: &str) -> Result<String, InfraError> {
    match lang {
        "python" => compile_python(content, stdin).await,
        _ => Err(InfraError::UnsupportedLanguage(format!(
            "{} languages is not supported",
            lang
        ))),
    }
}
