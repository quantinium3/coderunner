use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_lua(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".lua")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let mut cmd = Command::new(which("lua")?)
        .arg(&source_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = cmd.stdin.take() {
        stdin.write_all(stdin_input.as_bytes()).await?;
        stdin.flush().await?;
        drop(stdin);
    }

    let output = cmd.wait_with_output().await?;
    match output.status.code() {
        Some(0) => Ok(String::from_utf8(output.stdout)?),
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!(
                    "Lua program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Lua program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod lua_tests {
    use crate::infra::lua::compile_lua;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let lua_code = r#"
print("Hello, World!")
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let lua_code = r#"
local a = 5
local b = 3
print(a + b)
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let lua_code = r#"
local num = io.read("*number")
print("You entered: " .. num)
"#;

        let result = compile_lua(lua_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let lua_code = r#"
local name = io.read()
print("Hello, " .. name .. "!")
"#;

        let result = compile_lua(lua_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let lua_code = r#"
for i = 1, 3 do
    print("Line " .. i)
end
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_lua_code = r#"
print("Missing closing quote
"#;

        let result = compile_lua(invalid_lua_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compiletime_error() {
        let lua_code = r#"
error("compiletime error")
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let lua_code = r#"
local input = io.read()
local a, b = input:match("(%d+)%s+(%d+)")
a, b = tonumber(a), tonumber(b)
print("Sum: " .. (a + b))
print("Product: " .. (a * b))
"#;

        let result = compile_lua(lua_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let lua_code = r#"
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let lua_code = r#"
local str = "Hello"
print("Length: " .. #str)
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let lua_code = r#"
local x = 16.0
print(string.format("Square root of %s is %s", x, math.sqrt(x)))
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16 is 4");
    }

    #[tokio::test]
    async fn test_program_with_coroutines() {
        let lua_code = r#"
local co = coroutine.create(function()
    print("Coroutine compilening")
end)
coroutine.resume(co)
"#;

        let result = compile_lua(lua_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Coroutine compilening");
    }
}
