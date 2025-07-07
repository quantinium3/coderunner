use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn compile_dart(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".dart")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let executable_file = NamedTempFile::new()?;
    let executable_path = executable_file.path().to_path_buf();
    drop(executable_file);

    let compile_output = Command::new("dart")
        .arg("compile")
        .arg("exe")
        .arg(&source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Dart compilation failed:\n{}", stderr).into(),
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
                    "Dart program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Dart program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod dart_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let dart_code = r#"
void main() {
  print('Hello, World!');
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let dart_code = r#"
void main() {
  int a = 5;
  int b = 3;
  print(a + b);
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let dart_code = r#"
import 'dart:io';
void main() {
  int num = int.parse(stdin.readLineSync()!);
  print('You entered: $num');
}
"#;

        let result = compile_dart(dart_code, "42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let dart_code = r#"
import 'dart:io';
void main() {
  String name = stdin.readLineSync()!;
  print('Hello, $name!');
}
"#;

        let result = compile_dart(dart_code, "Alice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let dart_code = r#"
void main() {
  for (int i = 1; i <= 3; i++) {
    print('Line $i');
  }
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_dart_code = r#"
void main() {
  print('Missing semicolon') // This should cause a compilation error
}
"#;

        let result = compile_dart(invalid_dart_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let dart_code = r#"
void main() {
  throw Exception('Runtime error');
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let dart_code = r#"
import 'dart:io';
void main() {
  List<int> numbers = stdin.readLineSync()!.split(' ').map(int.parse).toList();
  int a = numbers[0];
  int b = numbers[1];
  print('Sum: ${a + b}');
  print('Product: ${a * b}');
}
"#;

        let result = compile_dart(dart_code, "7 3").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let dart_code = r#"
void main() {}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_stdlib() {
        let dart_code = r#"
void main() {
  String str = 'Hello';
  print('Length: ${str.length}');
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math() {
        let dart_code = r#"
import 'dart:math';
void main() {
  double x = 16.0;
  print('Square root of $x is ${sqrt(x)}');
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0");
    }

    #[tokio::test]
    async fn test_program_with_async() {
        let dart_code = r#"
import 'dart:async';
void main() async {
  await Future(() => print('Future running'));
}
"#;

        let result = compile_dart(dart_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Future running");
    }
}
