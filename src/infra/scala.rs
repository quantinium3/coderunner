use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_scala(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".scala")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let output_dir = tempfile::tempdir()?;
    let output_path = output_dir.path();

    let compile_output = Command::new(which("scalac")?)
        .arg(&source_path)
        .arg("-d")
        .arg(output_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Scala compilation failed:\n{}", stderr).into(),
        ));
    }

    let mut cmd = Command::new("scala")
        .arg("-cp")
        .arg(output_path)
        .arg("Main")
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
                    "Scala program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Scala program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod scala_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    println("Hello, World!")
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    val a = 5
    val b = 3
    println(a + b)
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    val num = scala.io.StdIn.readLine().toInt
    println(s"You entered: $num")
  }
}
"#;

        let result = compile_scala(scala_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    val name = scala.io.StdIn.readLine()
    println(s"Hello, $name!")
  }
}
"#;

        let result = compile_scala(scala_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    for (i <- 1 to 3) {
      println(s"Line $i")
    }
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    println("Missing closing brace" // This should cause a compilation error
"#;

        let result = compile_scala(invalid_scala_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    throw new Exception("Runtime error")
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    val Array(a, b) = scala.io.StdIn.readLine().split(" ").map(_.toInt)
    println(s"Sum: ${a + b}")
    println(s"Product: ${a * b}")
  }
}
"#;

        let result = compile_scala(scala_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {}
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    val str = "Hello"
    println(s"Length: ${str.length}")
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let scala_code = r#"
object Main {
  def main(args: Array[String]): Unit = {
    val x = 16.0
    println(s"Square root of $x is ${math.sqrt(x)}")
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_actors() {
        let scala_code = r#"
import scala.concurrent.Future
import scala.concurrent.ExecutionContext.Implicits.global
object Main {
  def main(args: Array[String]): Unit = {
    val future = Future { println("Future running") }
    Thread.sleep(100)
  }
}
"#;

        let result = compile_scala(scala_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Future running");
    }
}
