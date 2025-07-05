use std::{io::Write, process::Stdio};

use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, process::Command};

use super::error::InfraError;

pub async fn compile_javascript(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let mut cmd = Command::new("bun")
        .arg(temp_file.path())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
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
            let stderr = String::from_utf8(output.stderr)?;
            Err(InfraError::CompilationError(format!(
                "Failed to compile javascript. Program returned with Error code: {}, stderr: {}",
                code, stderr
            ).into()))
        }
        None => Err(InfraError::CompilationError(
            "Program returned no error code".into(),
        )),
    }
}

#[cfg(test)]
mod js_tests {
    use super::*;

    #[tokio::test]
    async fn test_compile_js_basic_output() {
        let content = r#"console.log('hello world')"#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "hello world");
    }

    #[tokio::test]
    async fn test_compile_js_with_empty_content() {
        let content = "";
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "");
    }

    #[tokio::test]
    async fn test_compile_js_multiple_lines() {
        let content = r#"
            console.log('line 1');
            console.log('line 2');
            console.log('line 3');
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "line 1\nline 2\nline 3");
    }

    #[tokio::test]
    async fn test_compile_js_with_stdin() {
        let content = r#"
            const input = await Bun.stdin.text();
            console.log('Received: ' + input.trim());
        "#;
        let stdin_input = "hello";
        let res = compile_javascript(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Received: hello");
    }

    #[tokio::test]
    async fn test_compile_js_with_empty_stdin() {
        let content = r#"
            const input = await Bun.stdin.text();
            console.log('Input length: ' + input.length);
        "#;
        let stdin_input = "";
        let res = compile_javascript(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Input length: 0");
    }

    #[tokio::test]
    async fn test_compile_js_with_multiline_stdin() {
        let content = r#"
            const input = await Bun.stdin.text();
            const lines = input.trim().split('\n');
            console.log('Lines: ' + lines.length);
        "#;
        let stdin_input = "line1\nline2\nline3";
        let res = compile_javascript(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Lines: 3");
    }

    #[tokio::test]
    async fn test_compile_js_syntax_error() {
        let content = r#"console.log('unclosed string"#;
        let result = compile_javascript(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_js_runtime_error() {
        let content = r#"
            console.log('before error');
            throw new Error('test error');
            console.log('after error');
        "#;
        let result = compile_javascript(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_js_reference_error() {
        let content = r#"console.log(undefinedVariable)"#;
        let result = compile_javascript(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_js_numbers() {
        let content = r#"
            console.log(42);
            console.log(3.14);
            console.log(-10);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "42\n3.14\n-10");
    }

    #[tokio::test]
    async fn test_compile_js_booleans() {
        let content = r#"
            console.log(true);
            console.log(false);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "true\nfalse");
    }

    #[tokio::test]
    async fn test_compile_js_arrays() {
        let content = r#"
            const arr = [1, 2, 3];
            console.log(JSON.stringify(arr));
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "[1,2,3]");
    }

    #[tokio::test]
    async fn test_compile_js_objects() {
        let content = r#"
            const obj = { name: 'test', value: 42 };
            console.log(JSON.stringify(obj));
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), r#"{"name":"test","value":42}"#);
    }

    #[tokio::test]
    async fn test_compile_js_functions() {
        let content = r#"
            function greet(name) {
                return 'Hello, ' + name + '!';
            }
            console.log(greet('World'));
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_compile_js_arrow_functions() {
        let content = r#"
            const add = (a, b) => a + b;
            console.log(add(5, 3));
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "8");
    }

    #[tokio::test]
    async fn test_compile_js_async_functions() {
        let content = r#"
            async function delay() {
                return new Promise(resolve => setTimeout(resolve, 10));
            }
            
            async function main() {
                await delay();
                console.log('async completed');
            }
            
            main();
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "async completed");
    }

    #[tokio::test]
    async fn test_compile_js_if_else() {
        let content = r#"
            const x = 10;
            if (x > 5) {
                console.log('greater');
            } else {
                console.log('lesser');
            }
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "greater");
    }

    #[tokio::test]
    async fn test_compile_js_loops() {
        let content = r#"
            for (let i = 0; i < 3; i++) {
                console.log('iteration ' + i);
            }
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "iteration 0\niteration 1\niteration 2");
    }

    #[tokio::test]
    async fn test_compile_js_destructuring() {
        let content = r#"
            const arr = [1, 2, 3];
            const [first, second] = arr;
            console.log(first + ' ' + second);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "1 2");
    }

    #[tokio::test]
    async fn test_compile_js_template_literals() {
        let content = r#"
            const name = 'JavaScript';
            const version = 2024;
            console.log(`Hello ${name} ${version}!`);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello JavaScript 2024!");
    }

    #[tokio::test]
    async fn test_compile_js_classes() {
        let content = r#"
            class Person {
                constructor(name) {
                    this.name = name;
                }
                
                greet() {
                    return 'Hello, I am ' + this.name;
                }
            }
            
            const person = new Person('Alice');
            console.log(person.greet());
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello, I am Alice");
    }

    #[tokio::test]
    async fn test_compile_js_unicode() {
        let content = r#"console.log('Hello 世界 🌍')"#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello 世界 🌍");
    }

    #[tokio::test]
    async fn test_compile_js_escape_sequences() {
        let content = r#"console.log('Line 1\nLine 2\tTabbed')"#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "Line 1\nLine 2\tTabbed");
    }

    #[tokio::test]
    async fn test_compile_js_json_parsing() {
        let content = r#"
            const json = '{"key": "value", "number": 42}';
            const parsed = JSON.parse(json);
            console.log(parsed.key + ' ' + parsed.number);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "value 42");
    }

    #[tokio::test]
    async fn test_compile_js_large_output() {
        let content = r#"
            for (let i = 0; i < 1000; i++) {
                console.log(i);
            }
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        let lines: Vec<&str> = res.trim().split('\n').collect();
        assert_eq!(lines.len(), 1000);
        assert_eq!(lines[0], "0");
        assert_eq!(lines[999], "999");
    }

    #[tokio::test]
    async fn test_compile_js_no_output() {
        let content = r#"
            const x = 5;
            const y = 10;
            const result = x + y;
            // No console.log
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "");
    }

    #[tokio::test]
    async fn test_compile_js_bun_features() {
        let content = r#"
            console.log(typeof Bun);
            console.log(Bun.version);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        let lines: Vec<&str> = res.trim().split('\n').collect();
        assert_eq!(lines[0], "object");
        assert!(lines[1].contains("1."));
    }

    #[tokio::test]
    async fn test_compile_js_require_builtin() {
        let content = r#"
            const fs = require('fs');
            console.log(typeof fs.readFileSync);
        "#;
        let res = compile_javascript(content, "").await.unwrap();
        assert_eq!(res.trim(), "function");
    }

    #[tokio::test]
    async fn test_compile_js_process_exit() {
        let content = r#"
            console.log('before exit');
            process.exit(2);
            console.log('after exit');
        "#;
        let result = compile_javascript(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_js_stdin_json() {
        let content = r#"
            const input = await Bun.stdin.text();
            const data = JSON.parse(input);
            console.log(data.name + ' is ' + data.age + ' years old');
        "#;
        let stdin_input = r#"{"name": "John", "age": 30}"#;
        let res = compile_javascript(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "John is 30 years old");
    }

    #[tokio::test]
    async fn test_compile_js_stdin_processing() {
        let content = r#"
            const input = await Bun.stdin.text();
            const numbers = input.trim().split('\n').map(Number);
            const sum = numbers.reduce((a, b) => a + b, 0);
            console.log('Sum: ' + sum);
        "#;
        let stdin_input = "10\n20\n30";
        let res = compile_javascript(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Sum: 60");
    }
}
