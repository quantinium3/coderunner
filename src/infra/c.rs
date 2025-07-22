use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_c(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".c")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let executable_file = NamedTempFile::new()?;
    let executable_path = executable_file.path().to_path_buf();
    drop(executable_file);

    let compile_output = Command::new(which("zig")?)
        .arg("cc")
        .arg(source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("C compilation failed:\n{}", stderr).into(),
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
                    "C program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("C program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod c_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let c_code = r#"
#include <stdio.h>
int main() {
    printf("Hello, World!\n");
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let c_code = r#"
#include <stdio.h>
int main() {
    int a = 5;
    int b = 3;
    printf("%d\n", a + b);
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let c_code = r#"
#include <stdio.h>
int main() {
    int num;
    scanf("%d", &num);
    printf("You entered: %d\n", num);
    return 0;
}
"#;

        let result = compile_c(c_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let c_code = r#"
#include <stdio.h>
int main() {
    char name[100];
    scanf("%s", name);
    printf("Hello, %s!\n", name);
    return 0;
}
"#;

        let result = compile_c(c_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let c_code = r#"
#include <stdio.h>
int main() {
    for (int i = 1; i <= 3; i++) {
        printf("Line %d\n", i);
    }
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_c_code = r#"
#include <stdio.h>
int main() {
    printf("Missing semicolon")  // This should cause a compilation error
    return 0;
}
"#;

        let result = compile_c(invalid_c_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let c_code = r#"
#include <stdio.h>
#include <stdlib.h>
int main() {
    exit(1);  // This should cause a runtime error
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let c_code = r#"
#include <stdio.h>
int main() {
    int a, b;
    scanf("%d %d", &a, &b);
    printf("Sum: %d\n", a + b);
    printf("Product: %d\n", a * b);
    return 0;
}
"#;

        let result = compile_c(c_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let c_code = r#"
int main() {
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_includes() {
        let c_code = r#"
#include <stdio.h>
#include <string.h>
int main() {
    char str[] = "Hello";
    printf("Length: %lu\n", strlen(str));
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math_includes() {
        let c_code = r#"
#include <stdio.h>
#include <math.h>
int main() {
    double x = 16.0;
    printf("Square root of %f is %f\n", x, sqrt(x));
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.000000 is 4.000000");
    }


    #[tokio::test]
    async fn test_program_with_pthread_includes() {
        let c_code = r#"
#include <pthread.h>
#include <stdio.h>
void* thread_func(void* arg) { printf("Thread running\n"); return NULL; }
int main() {
    pthread_t thread;
    pthread_create(&thread, NULL, thread_func, NULL);
    pthread_join(thread, NULL);
    return 0;
}
"#;

        let result = compile_c(c_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Thread running");
    }
}
