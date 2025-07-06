use super::{c::compile_c, cpp::compile_cpp, error::InfraError, javascript::compile_javascript, python::compile_python};

pub async fn compile_lang(lang: &str, content: &str, stdin: &str) -> Result<String, InfraError> {
    match lang {
        "python" => compile_python(content, stdin).await,
        "javascript" => compile_javascript(content, stdin).await,
        "typescript" => compile_javascript(content, stdin).await,
        "c" => compile_c(content, stdin).await,
        "cpp" => compile_cpp(content, stdin).await,
        _ => Err(InfraError::UnsupportedLanguage(format!(
            "{} languages is not supported",
            lang
        ))),
    }
}
