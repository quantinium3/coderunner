use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn compile_kotlin(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".kt")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let output_dir = tempfile::tempdir()?;
    let output_path = output_dir.path().join("MainKt.class");

    let compile_output = Command::new("kotlinc")
        .arg(&source_path)
        .arg("-d")
        .arg(output_dir.path())
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Kotlin compilation failed:\n{}", stderr).into(),
        ));
    }

    let mut cmd = Command::new("kotlin")
        .arg("-cp")
        .arg(output_dir.path())
        .arg("MainKt")
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
                    "Kotlin program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Kotlin program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod kotlin_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let kotlin_code = r#"
fun main() {
    println("Hello, World!")
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let kotlin_code = r#"
fun main() {
    val a = 5
    val b = 3
    println(a + b)
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let kotlin_code = r#"
fun main() {
    val num = readLine()!!.toInt()
    println("You entered: $num")
}
"#;

        let result = compile_kotlin(kotlin_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let kotlin_code = r#"
fun main() {
    val name = readLine()!!
    println("Hello, $name!")
}
"#;

        let result = compile_kotlin(kotlin_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let kotlin_code = r#"
fun main() {
    for (i in 1..3) {
        println("Line $i")
    }
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_kotlin_code = r#"
fun main() {
    println("Missing semicolon") // This should cause a compilation error
"#;

        let result = compile_kotlin(invalid_kotlin_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let kotlin_code = r#"
fun main() {
    throw Exception("Runtime error")
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let kotlin_code = r#"
fun main() {
    val (a, b) = readLine()!!.split(" ").map { it.toInt() }
    println("Sum: ${a + b}")
    println("Product: ${a * b}")
}
"#;

        let result = compile_kotlin(kotlin_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let kotlin_code = r#"
fun main() {
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let kotlin_code = r#"
fun main() {
    val str = "Hello"
    println("Length: ${str.length}")
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let kotlin_code = r#"
import kotlin.math.sqrt
fun main() {
    val x = 16.0
    println("Square root of $x is ${sqrt(x)}")
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_coroutines() {
        let kotlin_code = r#"
import kotlinx.coroutines.*
fun main() = runBlocking {
    launch { println("Coroutine running") }
    delay(100)
}
"#;

        let result = compile_kotlin(kotlin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Coroutine running");
    }
}
