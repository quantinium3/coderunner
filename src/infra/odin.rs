use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn compile_odin(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".odin")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let mut cmd = Command::new("odin")
        .arg("run")
        .arg(source_path)
        .arg("-file")
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
                    "Odin program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Odin program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod odin_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    fmt.println("Hello, World!")
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    a := 5
    b := 3
    fmt.println(a + b)
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    num: i32
    fmt.scanf("%d", &num)
    fmt.printf("You entered: %d\n", num)
}
"#;

        let result = compile_odin(odin_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    name: string
    fmt.scanf("%s", &name)
    fmt.printf("Hello, %s!\n", name)
}
"#;

        let result = compile_odin(odin_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    for i in 1..=3 {
        fmt.printf("Line %d\n", i)
    }
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    fmt.println("Missing closing brace"  // This should cause a compilation error
}
"#;

        let result = compile_odin(invalid_odin_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let odin_code = r#"
package main
import "core:os"
main :: proc() {
    os.exit(1)  // This should cause a runtime error
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let odin_code = r#"
package main
import "core:fmt"
main :: proc() {
    a, b: i32
    fmt.scanf("%d %d", &a, &b)
    fmt.printf("Sum: %d\n", a + b)
    fmt.printf("Product: %d\n", a * b)
}
"#;

        let result = compile_odin(odin_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let odin_code = r#"
package main
main :: proc() {
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_includes() {
        let odin_code = r#"
package main
import "core:fmt"
import "core:strings"
main :: proc() {
    str := "Hello"
    fmt.printf("Length: %d\n", len(str))
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math_includes() {
        let odin_code = r#"
package main
import "core:fmt"
import "core:math"
main :: proc() {
    x := 16.0
    fmt.printf("Square root of %f is %f\n", x, math.sqrt(x))
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.000000 is 4.000000");
    }

    #[tokio::test]
    async fn test_program_with_threads() {
        let odin_code = r#"
package main
import "core:fmt"
import "core:thread"
thread_func :: proc(t: ^thread.Thread) {
    fmt.println("Thread running")
}
main :: proc() {
    t := thread.create(thread_func)
    thread.start(t)
    thread.join(t)
}
"#;

        let result = compile_odin(odin_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Thread running");
    }
}
