use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_cpp(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".cpp")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let executable_file = NamedTempFile::new()?;
    let executable_path = executable_file.path().to_path_buf();
    drop(executable_file);

    let compile_output = Command::new(which("clang++")?)
        .arg(source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("C++ compilation failed:\n{}", stderr).into(),
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
                    "C++ program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("C++ program terminated by signal\nError: {}", stderr).into(),
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
#include <iostream>
int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_program_with_input() {
        let code = r#"
#include <iostream>
#include <string>
int main() {
    std::string name;
    std::cin >> name;
    std::cout << "Hello, " << name << "!" << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_program_with_multiple_inputs() {
        let code = r#"
#include <iostream>
int main() {
    int a, b;
    std::cin >> a >> b;
    std::cout << a + b << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "5 3").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_program_with_multiline_input() {
        let code = r#"
#include <iostream>
#include <string>
int main() {
    std::string line1, line2;
    std::getline(std::cin, line1);
    std::getline(std::cin, line2);
    std::cout << line1 << " " << line2 << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "Hello\nWorld").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello World");
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let code = r#"
#include <iostream>
int main() {
    undefined_function();
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_syntax_error() {
        let code = r#"
#include <iostream>
int main() {
    std::cout << "Missing semicolon"
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let code = r#"
#include <iostream>
int main() {
    std::cout << "About to exit with error" << std::endl;
    return 1;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_program_with_stderr_output() {
        let code = r#"
#include <iostream>
int main() {
    std::cout << "stdout message" << std::endl;
    std::cerr << "stderr message" << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "stdout message");
    }

    #[tokio::test]
    async fn test_program_with_loops() {
        let code = r#"
#include <iostream>
int main() {
    for (int i = 1; i <= 3; i++) {
        std::cout << i << " ";
    }
    std::cout << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "1 2 3");
    }

    #[tokio::test]
    async fn test_program_with_vectors() {
        let code = r#"
#include <iostream>
#include <vector>
int main() {
    std::vector<int> nums = {1, 2, 3, 4, 5};
    int sum = 0;
    for (int num : nums) {
        sum += num;
    }
    std::cout << sum << std::endl;
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "15");
    }

    #[tokio::test]
    async fn test_empty_program() {
        let code = r#"
int main() {
    return 0;
}
"#;
        let result = compile_cpp(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }
}
