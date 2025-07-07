use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn compile_groovy(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".groovy")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let output_dir = tempfile::tempdir()?;
    let output_path = output_dir.path();

    let compile_output = Command::new("groovyc")
        .arg(&source_path)
        .arg("--classpath")
        .arg(output_path)
        .arg("-d")
        .arg(output_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Groovy compilation failed:\n{}", stderr).into(),
        ));
    }

    let mut cmd = Command::new("groovy")
        .arg("-cp")
        .arg(output_path)
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
                    "Groovy program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Groovy program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod groovy_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let groovy_code = r#"
println "Hello, World!"
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let groovy_code = r#"
def a = 5
def b = 3
println a + b
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let groovy_code = r#"
def num = System.in.newReader().readLine().toInteger()
println "You entered: $num"
"#;

        let result = compile_groovy(groovy_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let groovy_code = r#"
def name = System.in.newReader().readLine()
println "Hello, $name!"
"#;

        let result = compile_groovy(groovy_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let groovy_code = r#"
(1..3).each { i ->
    println "Line $i"
}
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_groovy_code = r#"
println "Missing closing brace" // This should cause a compilation error
def x = {
"#;

        let result = compile_groovy(invalid_groovy_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let groovy_code = r#"
throw new Exception("Runtime error")
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let groovy_code = r#"
def (a, b) = System.in.newReader().readLine().split().collect { it.toInteger() }
println "Sum: ${a + b}"
println "Product: ${a * b}"
"#;

        let result = compile_groovy(groovy_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let groovy_code = r#"
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let groovy_code = r#"
def str = "Hello"
println "Length: ${str.length()}"
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let groovy_code = r#"
def x = 16.0
println "Square root of $x is ${Math.sqrt(x)}"
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_threads() {
        let groovy_code = r#"
def thread = new Thread({
    println "Thread running"
})
thread.start()
thread.join()
"#;

        let result = compile_groovy(groovy_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Thread running");
    }
}
