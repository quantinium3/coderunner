use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_crystal(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".cr")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let executable_file = NamedTempFile::new()?;
    let executable_path = executable_file.path().to_path_buf();
    drop(executable_file);

    let compile_output = Command::new(which("crystal")?)
        .arg("build")
        .arg(&source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Crystal compilation failed:\n{}", stderr).into(),
        ));
    }

    let mut cmd = Command::new(&executable_path)
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
                    "Crystal program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Crystal program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod crystal_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let crystal_code = r#"
puts "Hello, World!"
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let crystal_code = r#"
a = 5
b = 3
puts (a + b)
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let crystal_code = r#"
num = gets.not_nil!.to_i
puts "You entered: #{num}"
"#;

        let result = compile_crystal(crystal_code, "42\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let crystal_code = r#"
name = gets.not_nil!.chomp
puts "Hello, #{name}!"
"#;

        let result = compile_crystal(crystal_code, "Alice\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let crystal_code = r#"
(1..3).each do |i|
  puts "Line #{i}"
end
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let crystal_code = r#"
exit(1)  # This should cause a runtime error
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let crystal_code = r#"
input = gets.not_nil!.split
a = input[0].to_i
b = input[1].to_i
puts "Sum: #{a + b}"
puts "Product: #{a * b}"
"#;

        let result = compile_crystal(crystal_code, "7 3\n").await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let crystal_code = r#"
# Empty Crystal program
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_includes() {
        let crystal_code = r#"
require "string"
str = "Hello"
puts "Length: #{str.size}"
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math_includes() {
        let crystal_code = r#"
require "math"
x = 16.0
puts "Square root of #{x} is #{Math.sqrt(x)}"
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_fibers() {
        let crystal_code = r#"
spawn do
  puts "Thread running"
end
Fiber.yield
"#;

        let result = compile_crystal(crystal_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Thread running");
    }
}
