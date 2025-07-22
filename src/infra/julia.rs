use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_julia(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".jl")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let mut cmd = Command::new(which("julia")?)
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
                    "Julia program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Julia program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod julia_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let julia_code = r#"
println("Hello, World!")
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let julia_code = r#"
a = 5
b = 3
println(a + b)
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let julia_code = r#"
num = parse(Int, readline())
println("You entered: $num")
"#;

        let result = compile_julia(julia_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let julia_code = r#"
for i in 1:3
    println("Line $i")
end
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_julia_code = r#"
println("Missing closing quote
"#;

        let result = compile_julia(invalid_julia_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compiletime_error() {
        let julia_code = r#"
throw(ErrorException("compiletime error"))
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let julia_code = r#"
a, b = parse.(Int, split(readline()))
println("Sum: $(a + b)")
println("Product: $(a * b)")
"#;

        let result = compile_julia(julia_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let julia_code = r#"
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let julia_code = r#"
str = "Hello"
println("Length: $(length(str))")
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let julia_code = r#"
x = 16.0
println("Square root of $x is $(sqrt(x))")
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_tasks() {
        let julia_code = r#"
@async begin
    println("Task compilening")
end
sleep(0.1)
"#;

        let result = compile_julia(julia_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Task compilening");
    }
}
