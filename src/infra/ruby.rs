use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_ruby(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".rb")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let mut cmd = Command::new(which("ruby")?)
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
                    "Ruby program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Ruby program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod ruby_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let ruby_code = r#"
puts "Hello, World!"
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let ruby_code = r#"
a = 5
b = 3
puts a + b
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let ruby_code = r#"
num = gets.to_i
puts "You entered: #{num}"
"#;

        let result = compile_ruby(ruby_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let ruby_code = r#"
name = gets.chomp
puts "Hello, #{name}!"
"#;

        let result = compile_ruby(ruby_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let ruby_code = r#"
(1..3).each do |i|
  puts "Line #{i}"
end
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_ruby_code = r#"
puts "Missing closing quote
"#;

        let result = compile_ruby(invalid_ruby_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compiletime_error() {
        let ruby_code = r#"
raise "compiletime error"
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let ruby_code = r#"
a, b = gets.split.map(&:to_i)
puts "Sum: #{a + b}"
puts "Product: #{a * b}"
"#;

        let result = compile_ruby(ruby_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let ruby_code = r#"
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let ruby_code = r#"
str = "Hello"
puts "Length: #{str.length}"
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let ruby_code = r#"
x = 16.0
puts "Square root of #{x} is #{Math.sqrt(x)}"
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_threads() {
        let ruby_code = r#"
thread = Thread.new { puts "Thread compilening" }
thread.join
"#;

        let result = compile_ruby(ruby_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Thread compilening");
    }
}
