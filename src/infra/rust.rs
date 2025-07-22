use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_rust(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".rs")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let executable_file = NamedTempFile::new()?;
    let executable_path = executable_file.path().to_path_buf();
    drop(executable_file);

    let compile_output = Command::new(which("rustc")?)
        .arg(source_path)
        .arg("--crate-name")
        .arg("temp")
        .arg("-o")
        .arg(&executable_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Rust compilation failed:\n{}", stderr).into(),
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
                    "Rust program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Rust program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let code = r#"
fn main() {
    println!("Hello, World!");
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_program_with_input() {
        let code = r#"
use std::io;

fn main() {
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim();
    println!("Hello, {}!", name);
}
"#;
        let result = compile_rust(code, "Alice\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_program_with_multiple_inputs() {
        let code = r#"
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let numbers: Vec<i32> = input
        .trim()
        .split_whitespace()
        .map(|s| s.parse().expect("Parse error"))
        .collect();
    
    let sum = numbers[0] + numbers[1];
    println!("{}", sum);
}
"#;
        let result = compile_rust(code, "5 3\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_program_with_multiline_input() {
        let code = r#"
use std::io;

fn main() {
    let mut line1 = String::new();
    let mut line2 = String::new();
    
    io::stdin().read_line(&mut line1).expect("Failed to read line");
    io::stdin().read_line(&mut line2).expect("Failed to read line");
    
    println!("{} {}", line1.trim(), line2.trim());
}
"#;
        let result = compile_rust(code, "Hello\nWorld\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello World");
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let code = r#"
fn main() {
    undefined_function();
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_syntax_error() {
        let code = r#"
fn main() {
    println!("Missing semicolon")
    println!("Missing semicolon")
}
"#;
        let result = compile_rust(code, "").await;
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let code = r#"
use std::process;

fn main() {
    println!("About to exit with error");
    process::exit(1);
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_program_with_stderr_output() {
        let code = r#"
fn main() {
    println!("stdout message");
    eprintln!("stderr message");
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "stdout message");
    }

    #[tokio::test]
    async fn test_program_with_loops() {
        let code = r#"
fn main() {
    for i in 1..=3 {
        print!("{} ", i);
    }
    println!();
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "1 2 3");
    }

    #[tokio::test]
    async fn test_program_with_vectors() {
        let code = r#"
fn main() {
    let nums = vec![1, 2, 3, 4, 5];
    let sum: i32 = nums.iter().sum();
    println!("{}", sum);
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "15");
    }

    #[tokio::test]
    async fn test_empty_program() {
        let code = r#"
fn main() {
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_match() {
        let code = r#"
fn main() {
    let x = 5;
    match x {
        1 => println!("one"),
        2 => println!("two"),
        _ => println!("other"),
    }
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "other");
    }

    #[tokio::test]
    async fn test_program_with_ownership() {
        let code = r#"
fn main() {
    let s = String::from("hello");
    let len = s.len();
    println!("Length: {}", len);
}
"#;
        let result = compile_rust(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }
}
