use super::{c::compile_c, cpp::compile_cpp, d::compile_d, error::InfraError, go::compile_go, groovy::compile_groovy, javascript::compile_javascript, nix::compile_nix, python::compile_python, rust::compile_rust, scala::compile_scala, zig::compile_zig};

pub async fn compile_lang(lang: &str, content: &str, stdin: &str) -> Result<String, InfraError> {
    match lang {
        "python" => compile_python(content, stdin).await,
        "javascript" => compile_javascript(content, stdin).await,
        "typescript" => compile_javascript(content, stdin).await,
        "c" => compile_c(content, stdin).await,
        "cpp" => compile_cpp(content, stdin).await,
        "rust" => compile_rust(content, stdin).await,
        "nix" => compile_nix(content, stdin).await,
        "go" => compile_go(content, stdin).await,
        "zig" => compile_zig(content, stdin).await,
        "d" => compile_d(content, stdin).await,
        "scala" => compile_scala(content, stdin).await,
        "groovy" => compile_groovy(content, stdin).await,
        _ => Err(InfraError::UnsupportedLanguage(format!(
            "{} languages is not supported",
            lang
        ))),
    }
}
