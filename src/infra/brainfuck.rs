use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_brainfuck(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".bf")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let source_stem = source_path.file_stem().unwrap().to_string_lossy();
    
    let executable_path = std::env::current_dir()?.join(&*source_stem);

    let compile_output = Command::new(which("bfc")?)
        .arg(&source_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        let stdout = String::from_utf8_lossy(&compile_output.stdout);
        return Err(InfraError::CompilationError(
            format!("Brainfuck compilation failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr).into(),
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
    
    if executable_path.exists() {
        std::fs::remove_file(&executable_path).ok();
    }
    
    match output.status.code() {
        Some(0) => Ok(String::from_utf8(output.stdout)?),
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!(
                    "Brainfuck program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Brainfuck program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod brainfuck_tests {
    use super::*;

    #[tokio::test]
    async fn test_hello_world() {
        // Hello World in Brainfuck
        let bf_code = r#"
++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++
.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.
------.--------.>+.>.
"#;

        let result = compile_brainfuck(bf_code, "").await;
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello World!");
    }

    #[tokio::test]
    async fn test_simple_output() {
        // Output the character 'A' (ASCII 65)
        let bf_code = "++++++++[>++++++++<-]>+.";

        let result = compile_brainfuck(bf_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "A");
    }

    #[tokio::test]
    async fn test_echo_input() {
        // Read one character and echo it back
        let bf_code = ",.";

        let result = compile_brainfuck(bf_code, "X").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "X");
    }

    #[tokio::test]
    async fn test_add_numbers() {
        // Read two single-digit numbers and output their sum
        // This is a simplified version that works with ASCII digits
        let bf_code = r#"
,>,,<[->+<]>.
"#;

        let result = compile_brainfuck(bf_code, "23").await;
        assert!(result.is_ok());
        // The output will be the ASCII character for 5 (sum of 2+3)
    }

    #[tokio::test]
    async fn test_empty_program() {
        let bf_code = "";

        let result = compile_brainfuck(bf_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_comments_ignored() {
        // Brainfuck ignores all characters except +-<>[].,
        let bf_code = r#"
This is a comment
++++++++[>++++++++<-]>+. Output A
More comments here
"#;

        let result = compile_brainfuck(bf_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "A");
    }

    #[tokio::test]
    async fn test_nested_loops() {
        // Test nested loop structure
        let bf_code = "+++[>+++[>++<-]<-]>>.";

        let result = compile_brainfuck(bf_code, "").await;
        assert!(result.is_ok());
        // This should output a character (ASCII 18)
    }

    #[tokio::test]
    async fn test_multiple_output() {
        // Output multiple characters
        let bf_code = r#"
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
"#;

        let result = compile_brainfuck(bf_code, "").await;
        assert!(result.is_ok());
        // Should output "Hello World!" or similar
    }

    #[tokio::test]
    async fn test_input_processing() {
        // Read input and modify it before output
        let bf_code = ",+."; // Read char, increment by 1, output

        let result = compile_brainfuck(bf_code, "A").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "B"); // A + 1 = B
    }

    #[tokio::test]
    async fn test_invalid_brackets() {
        // Test with unmatched brackets - this might be caught by bfc
        let bf_code = "[+";

        let result = compile_brainfuck(bf_code, "").await;
        // This should either fail at compile time or runtime
        // The exact behavior depends on the bfc implementation
    }

    #[tokio::test]
    async fn test_zero_byte_handling() {
        // Test handling of zero bytes in memory
        let bf_code = "+[-]>++."; // Set cell to 1, clear it, move right, set to 2, output

        let result = compile_brainfuck(bf_code, "").await;
        assert!(result.is_ok());
        // Should output ASCII character 2
    }
}
