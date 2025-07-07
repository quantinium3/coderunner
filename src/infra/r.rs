use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn compile_r(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".R")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let mut cmd = Command::new("Rscript")
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
                    "R program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("R program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod r_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let r_code = r#"
cat("Hello, World!\n")
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let r_code = r#"
a <- 5
b <- 3
cat(a + b, "\n")
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let r_code = r#"
num <- as.integer(readLines("stdin", n=1))
cat(sprintf("You entered: %d\n", num))
"#;

        let result = compile_r(r_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let r_code = r#"
name <- readLines("stdin", n=1)
cat(sprintf("Hello, %s!\n", name))
"#;

        let result = compile_r(r_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let r_code = r#"
for (i in 1:3) {
    cat(sprintf("Line %d\n", i))
}
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_r_code = r#"
cat("Missing closing quote
"#;

        let result = compile_r(invalid_r_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compiletime_error() {
        let r_code = r#"
stop("compiletime error")
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let r_code = r#"
input <- as.integer(strsplit(readLines("stdin", n=1), " ")[[1]])
a <- input[1]
b <- input[2]
cat(sprintf("Sum: %d\n", a + b))
cat(sprintf("Product: %d\n", a * b))
"#;

        let result = compile_r(r_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let r_code = r#"
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let r_code = r#"
str <- "Hello"
cat(sprintf("Length: %d\n", nchar(str)))
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let r_code = r#"
x <- 16.0
cat(sprintf("Square root of %s is %s\n", x, sqrt(x)))
"#;

        let result = compile_r(r_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16 is 4");
    }
}
