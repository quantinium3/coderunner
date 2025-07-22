use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_perl(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".pl")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let source_path = temp_file.path().to_path_buf();

    let mut cmd = Command::new(which("perl")?)
        .arg(source_path)
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
                    "Perl program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Perl program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod perl_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let perl_code = r#"
print "Hello, World!\n";
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_simple_arithmetic() {
        let perl_code = r#"
my $a = 5;
my $b = 3;
print $a + $b, "\n";
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let perl_code = r#"
my $num = <STDIN>;
chomp($num);
print "You entered: $num\n";
"#;

        let result = compile_perl(perl_code, "42\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: 42");
    }

    #[tokio::test]
    async fn test_string_input() {
        let perl_code = r#"
my $name = <STDIN>;
chomp($name);
print "Hello, $name!\n";
"#;

        let result = compile_perl(perl_code, "Alice\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_multiple_lines_output() {
        let perl_code = r#"
for my $i (1..3) {
    print "Line $i\n";
}
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let perl_code = r#"
exit(1);  # This should cause a runtime error
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_stdin_processing() {
        let perl_code = r#"
my ($a, $b) = split(' ', <STDIN>);
chomp($a, $b);
print "Sum: ", $a + $b, "\n";
print "Product: ", $a * $b, "\n";
"#;

        let result = compile_perl(perl_code, "7 3\n").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Sum: 10"));
        assert!(output.contains("Product: 21"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let perl_code = r#"
# Empty Perl program
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_includes() {
        let perl_code = r#"
use strict;
use warnings;
my $str = "Hello";
print "Length: ", length($str), "\n";
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Length: 5");
    }

    #[tokio::test]
    async fn test_program_with_math_includes() {
        let perl_code = r#"
use strict;
use warnings;
use Math::Trig;
my $x = 16.0;
print "Square root of $x is ", sqrt($x), "\n";
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Square root of 16 is 4");
    }

    #[tokio::test]
    async fn test_program_with_threads() {
        let perl_code = r#"
use strict;
use warnings;
use threads;
my $thread = threads->create(sub { print "Thread running\n"; });
$thread->join();
"#;

        let result = compile_perl(perl_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Thread running");
    }
}
