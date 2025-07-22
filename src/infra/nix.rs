use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_nix(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".nix")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let eval_output = Command::new(which("nix")?)
        .arg("eval")
        .arg("--file")
        .arg(&source_path)
        .arg("--raw")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut cmd = eval_output;
    
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
                    "Nix evaluation failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Nix evaluation terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_simple_string() {
        let code = r#""Hello, World!""#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_arithmetic() {
        let code = r#"toString (5 + 3)"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_list_operations() {
        let code = r#"
let
  nums = [1 2 3 4 5];
  sum = builtins.foldl' (acc: x: acc + x) 0 nums;
in
  toString sum
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "15");
    }

    #[tokio::test]
    async fn test_attribute_set() {
        let code = r#"
let
  person = { name = "Alice"; age = 30; };
in
  "Hello, ${person.name}!"
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_function_definition() {
        let code = r#"
let
  greet = name: "Hello, ${name}!";
in
  greet "World"
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_conditional_expression() {
        let code = r#"
let
  x = 5;
in
  if x > 3 then "greater" else "smaller"
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "greater");
    }

    #[tokio::test]
    async fn test_string_interpolation() {
        let code = r#"
let
  name = "Nix";
  version = "2.0";
in
  "Using ${name} version ${version}"
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Using Nix version 2.0");
    }

    #[tokio::test]
    async fn test_map_function() {
        let code = r#"
let
  nums = [1 2 3];
  doubled = map (x: x * 2) nums;
in
  toString (builtins.foldl' (acc: x: acc + x) 0 doubled)
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "12");
    }

    #[tokio::test]
    async fn test_with_expression() {
        let code = r#"
let
  attrs = { x = 10; y = 20; };
in
  with attrs; toString (x + y)
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "30");
    }

    #[tokio::test]
    async fn test_builtin_functions() {
        let code = r#"
let
  str = "hello world";
  words = builtins.split " " str;
in
  toString (builtins.length words)
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "3");
    }

    #[tokio::test]
    async fn test_syntax_error() {
        let code = r#"
let
  x = 1
  y = 2;
in
  x + y
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_evaluation_error() {
        let code = r#"
let
  x = undefined_variable;
in
  x
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_type_error() {
        let code = r#"1 + "string""#;
        let result = compile_nix(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_expression() {
        let code = r#""""#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_nested_functions() {
        let code = r#"
let
  add = x: y: x + y;
  multiply = x: y: x * y;
  compute = x: add (multiply x 2) 3;
in
  toString (compute 5)
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "13");
    }

    #[tokio::test]
    async fn test_path_operations() {
        let code = r#"
let
  path = /tmp/test.txt;
in
  toString path
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "/tmp/test.txt");
    }

    #[tokio::test]
    async fn test_boolean_operations() {
        let code = r#"
let
  a = true;
  b = false;
in
  toString (a && !b)
"#;
        let result = compile_nix(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "1");
    }
}
