use super::error::InfraError;
use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};
use which::which;

pub async fn compile_zig(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::with_suffix(".zig")?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;
    let source_path = temp_file.path().to_path_buf();

    let executable_file = NamedTempFile::new()?;
    drop(executable_file);

    let mut cmd = Command::new(which("zig")?)
        .arg("run")
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
                    "Zig program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Zig program terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_simple_hello_world() {
        let zig_code = r#"
const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Hello, World!\n", .{});
}
"#;

        let result = compile_zig(zig_code, "").await;
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_stdin_input() {
        let zig_code = r#"
const std = @import("std");

pub fn main() !void {
    const stdin = std.io.getStdIn().reader();
    const stdout = std.io.getStdOut().writer();
    
    var buffer: [100]u8 = undefined;
    if (try stdin.readUntilDelimiterOrEof(buffer[0..], '\n')) |input| {
        try stdout.print("You entered: {s}\n", .{input});
    }
}
"#;

        let result = compile_zig(zig_code, "Hello Zig").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "You entered: Hello Zig");
    }

    #[tokio::test]
    async fn test_arithmetic_operations() {
        let zig_code = r#"
const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    const a: i32 = 10;
    const b: i32 = 5;
    
    try stdout.print("Addition: {}\n", .{a + b});
    try stdout.print("Subtraction: {}\n", .{a - b});
    try stdout.print("Multiplication: {}\n", .{a * b});
    try stdout.print("Division: {}\n", .{@divTrunc(a, b)});
}
"#;

        let result = compile_zig(zig_code, "").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Addition: 15"));
        assert!(output.contains("Subtraction: 5"));
        assert!(output.contains("Multiplication: 50"));
        assert!(output.contains("Division: 2"));
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let invalid_zig_code = r#"
const std = @import("std");

pub fn main() !void {
    // This should cause a compilation error - undefined variable
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Value: {}\n", .{undefined_variable});
}
"#;

        let result = compile_zig(invalid_zig_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let zig_code = r#"
const std = @import("std");

pub fn main() !void {
    std.process.exit(1);
}
"#;

        let result = compile_zig(zig_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_stdin_lines() {
        let zig_code = r#"
const std = @import("std");

pub fn main() !void {
    const stdin = std.io.getStdIn().reader();
    const stdout = std.io.getStdOut().writer();
    
    var buffer: [100]u8 = undefined;
    var line_count: u32 = 0;
    
    while (try stdin.readUntilDelimiterOrEof(buffer[0..], '\n')) |input| {
        line_count += 1;
        try stdout.print("Line {}: {s}\n", .{ line_count, input });
    }
}
"#;

        let input = "First line\nSecond line\nThird line";
        let result = compile_zig(zig_code, input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Line 1: First line"));
        assert!(output.contains("Line 2: Second line"));
        assert!(output.contains("Line 3: Third line"));
    }

    #[tokio::test]
    async fn test_empty_program() {
        let zig_code = r#"
pub fn main() !void {
    // Empty main function
}
"#;

        let result = compile_zig(zig_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_complex_data_structures() {
        let zig_code = r#"
const std = @import("std");

const Person = struct {
    name: []const u8,
    age: u32,
};

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    
    const person = Person{
        .name = "Alice",
        .age = 30,
    };
    
    try stdout.print("Name: {s}, Age: {}\n", .{ person.name, person.age });
}
"#;

        let result = compile_zig(zig_code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Name: Alice, Age: 30");
    }

    #[tokio::test]
    async fn test_syntax_error() {
        let invalid_zig_code = r#"
const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    // Missing semicolon and incorrect syntax
    try stdout.print("Hello"
}
"#;

        let result = compile_zig(invalid_zig_code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_large_output() {
        let zig_code = r#"
const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    
    var i: u32 = 0;
    while (i < 10) : (i += 1) {
        try stdout.print("Line {}: This is a test line\n", .{i});
    }
}
"#;

        let result = compile_zig(zig_code, "").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let lines: Vec<&str> = output.trim().split('\n').collect();
        assert_eq!(lines.len(), 10);
        assert!(lines[0].contains("Line 0: This is a test line"));
        assert!(lines[9].contains("Line 9: This is a test line"));
    }
}
