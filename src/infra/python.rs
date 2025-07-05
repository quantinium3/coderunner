use super::error::InfraError;
use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn compile_python(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let mut cmd = Command::new("python3")
        .arg(temp_file.path())
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
                    "Python execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Python process terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compile_python_success() {
        let code = r#"print("Hello, World!")"#;
        let stdin = "";

        let res = compile_python(code, stdin).await;
        assert!(res.is_ok());
        let out = res.unwrap();
        assert_eq!(out.trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_compile_with_stdin() {
        let code = r#"
user_input = input()
print(f"Received: {user_input}")
"#;
        let res = compile_python(code, "Test Input").await;
        assert!(res.is_ok());
        let out = res.unwrap();
        assert_eq!(out.trim(), "Received: Test Input")
    }

    #[tokio::test]
    async fn test_compile_with_syntax_error() {
        let code = r#"print("Missing closing quote)"#;
        let res = compile_python(code, "").await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_compile_with_empty_code() {
        let code = "";
        let res = compile_python(code, "").await;

        assert!(res.is_ok());
        let out = res.unwrap();
        assert_eq!(out.trim(), "")
    }
}
