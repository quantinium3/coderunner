use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_d(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".d")?;
    let modified_content = format!("module temp;\n{}", content);
    temp_file.write_all(modified_content.as_bytes())?;
    temp_file.flush()?;
    let source_path = temp_file.path().to_path_buf();

    let executable_file = NamedTempFile::new()?;
    drop(executable_file);

    let mut cmd = Command::new(which("dmd")?)
        .arg("-run")
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
                    "D program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("D program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InfraError, compile_d};
    use tokio;

    #[tokio::test]
    async fn test_compile_d_success() {
        let d_code = r#"
import std.stdio;
void main() {
    writeln("Hello, D!");
}
"#;
        let result = compile_d(d_code, "").await;
        assert!(
            result.is_ok(),
            "Expected successful execution, got {:?}",
            result
        );
        assert_eq!(result.unwrap(), "Hello, D!\n", "Unexpected output");
    }

    #[tokio::test]
    async fn test_compile_d_compilation_error() {
        let invalid_d_code = r#"
import std.stdio;
void main() {
    writeln("Hello, D!"; // Missing closing parenthesis
}
"#;
        let result = compile_d(invalid_d_code, "").await;
        assert!(
            matches!(result, Err(InfraError::CompilationError(_))),
            "Expected compilation error, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_compile_d_runtime_error() {
        let d_code = r#"
import std.stdio;
void main() {
    int[] arr;
    writeln(arr[0]); // Access out of bounds
}
"#;
        let result = compile_d(d_code, "").await;
        assert!(
            matches!(result, Err(InfraError::CompilationError(_))),
            "Expected runtime error, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_compile_d_with_stdin() {
        let d_code = r#"
import std.stdio;
void main() {
    char[] input;
    stdin.readln(input);
    writeln("Received: ", input);
}
"#;
        let input = "Test Input\n";
        let result = compile_d(d_code, input).await;
        assert!(
            result.is_ok(),
            "Expected successful execution with stdin, got {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            "Received: Test Input\n\n",
            "Unexpected output with stdin"
        );
    }
}
