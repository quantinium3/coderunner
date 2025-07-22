use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_haskell(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".hs")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();
    let executable_file = NamedTempFile::new()?;
    let executable_path = executable_file.path().to_path_buf();
    drop(executable_file);

    let compile_output = Command::new(which("ghc")?)
        .arg("-o")
        .arg(&executable_path)
        .arg(&source_path)
        .output()
        .await?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(InfraError::CompilationError(
            format!("Haskell compilation failed:\n{}", stderr).into(),
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
                    "Haskell program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Haskell program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod haskell_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let haskell_code = r#"
main :: IO ()
main = putStrLn "Hello, World!"
"#;
        let result = compile_haskell(haskell_code, "").await;
        println!("{:?}", result);
        assert!(result.is_ok(), "Failed to compile or execute simple hello world program");
        assert_eq!(result.unwrap().trim(), "Hello, World!", "Expected output 'Hello, World!' but got different output");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let haskell_code = r#"
main :: IO ()
main = do
    let a = 5
    let b = 3
    print (a + b)
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_ok(), "Failed to compile or execute simple arithmetic program");
        assert_eq!(result.unwrap().trim(), "8", "Expected output '8' but got different output");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let haskell_code = r#"
main :: IO ()
main = do
    num <- readLn :: IO Int
    putStrLn $ "You entered: " ++ show num
"#;

        let result = compile_haskell(haskell_code, "42\n").await;
        assert!(result.is_ok(), "Failed to compile or execute program with stdin input");
        assert_eq!(result.unwrap().trim(), "You entered: 42", "Expected output 'You entered: 42' but got different output");
    }

    #[tokio::test]
    async fn test_string_input() {
        let haskell_code = r#"
main :: IO ()
main = do
    name <- getLine
    putStrLn $ "Hello, " ++ name ++ "!"
"#;

        let result = compile_haskell(haskell_code, "Alice\n").await;
        assert!(result.is_ok(), "Failed to compile or execute program with string input");
        assert_eq!(result.unwrap().trim(), "Hello, Alice!", "Expected output 'Hello, Alice!' but got different output");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let haskell_code = r#"
main :: IO ()
main = mapM_ (\i -> putStrLn $ "Line " ++ show i) [1..3]
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_ok(), "Failed to compile or execute program with multiple lines output");
        let output = result.unwrap();
        assert!(output.contains("Line 1"), "Output does not contain 'Line 1'");
        assert!(output.contains("Line 2"), "Output does not contain 'Line 2'");
        assert!(output.contains("Line 3"), "Output does not contain 'Line 3'");
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let haskell_code = r#"
import System.Exit
main :: IO ()
main = exitWith (ExitFailure 1)  -- This should cause a runtime error
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_err(), "Expected runtime error due to exitWith (ExitFailure 1) but program executed successfully");
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let haskell_code = r#"
main :: IO ()
main = do
    input <- getLine
    let [a, b] = map read (words input) :: [Int]
    putStrLn $ "Sum: " ++ show (a + b)
    putStrLn $ "Product: " ++ show (a * b)
"#;

        let result = compile_haskell(haskell_code, "7 3\n").await;
        assert!(result.is_ok(), "Failed to compile or execute program with complex stdin processing");
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"), "Output does not contain 'Sum: 10'");
        assert!(output.contains("Product: 21"), "Output does not contain 'Product: 21'");
    }

    #[tokio::test]
    async fn test_empty_program() {
        let haskell_code = r#"
main :: IO ()
main = return ()
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_ok(), "Failed to compile or execute empty program");
        assert_eq!(result.unwrap().trim(), "", "Expected empty output but got different output");
    }

    #[tokio::test]
    async fn test_program_with_includes() {
        let haskell_code = r#"
import Data.List
main :: IO ()
main = do
    let str = "Hello"
    putStrLn $ "Length: " ++ show (length str)
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_ok(), "Failed to compile or execute program with Data.List import");
        assert_eq!(result.unwrap().trim(), "Length: 5", "Expected output 'Length: 5' but got different output");
    }

    #[tokio::test]
    async fn test_program_with_math_includes() {
        let haskell_code = r#"
import Prelude hiding (sqrt)
import qualified Prelude as P
main :: IO ()
main = do
    let x = 16.0 :: Double
    putStrLn $ "Square root of " ++ show x ++ " is " ++ show (P.sqrt x)
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_ok(), "Failed to compile or execute program with math operations");
        assert_eq!(result.unwrap().trim(), "Square root of 16.0 is 4.0", "Expected output 'Square root of 16.0 is 4.0' but got different output");
    }

    #[tokio::test]
    async fn test_program_with_threads() {
        let haskell_code = r#"
import Control.Concurrent
main :: IO ()
main = do
    forkIO $ putStrLn "Thread running"
    threadDelay 100000  -- Allow thread to execute
"#;

        let result = compile_haskell(haskell_code, "").await;
        assert!(result.is_ok(), "Failed to compile or execute program with Control.Concurrent");
        assert_eq!(result.unwrap().trim(), "Thread running", "Expected output 'Thread running' but got different output");
    }
}
