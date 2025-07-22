use super::{
    brainfuck::compile_brainfuck, c::compile_c, cpp::compile_cpp, crystal::compile_crystal, d::compile_d, dart::compile_dart, error::InfraError, go::compile_go, groovy::compile_groovy, haskell::compile_haskell, javascript::compile_javascript, julia::compile_julia, lua::compile_lua, nix::compile_nix, perl::compile_perl, python::compile_python, r::compile_r, ruby::compile_ruby, rust::compile_rust, scala::compile_scala, zig::compile_zig
};

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
        "dart" => compile_dart(content, stdin).await,
        "ruby" => compile_ruby(content, stdin).await,
        "lua" => compile_lua(content, stdin).await,
        "julia" => compile_julia(content, stdin).await,
        "r" => compile_r(content, stdin).await,
        "perl" => compile_perl(content, stdin).await,
        "crystal" => compile_crystal(content, stdin).await,
        "haskell" => compile_haskell(content, stdin).await,
        "brainfuck" => compile_brainfuck(content, stdin).await,
        _ => Err(InfraError::UnsupportedLanguage(format!(
            "{} languages is not supported",
            lang
        ))),
    }
}
