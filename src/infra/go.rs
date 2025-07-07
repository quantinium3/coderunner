use super::error::InfraError;
use std::{fs::File, io::Write, process::Stdio};
use tempfile::{TempDir};
use tokio::{fs::metadata, io::AsyncWriteExt, process::Command};

pub async fn compile_go(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let temp_dir = TempDir::new()?;
    let temp_file_path = temp_dir.path().join("program.go");

    let mut temp_file = File::create(&temp_file_path)?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    if !temp_file_path.exists() {
        return Err(InfraError::CompilationError(
            format!("Temporary file does not exist: {:?}", temp_file_path).into(),
        ));
    }

    let metadata = metadata(&temp_file_path).await?;
    if metadata.len() == 0 {
        return Err(InfraError::CompilationError(
            "Temporary file is empty".into(),
        ));
    }

    eprintln!("Executing go run on file: {:?}", temp_file_path);
    eprintln!("File content: {}", content);

    let mut cmd = Command::new("go")
        .arg("run")
        .arg(&temp_file_path)
        .current_dir(temp_dir.path())
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
                    "Go program execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Go program terminated by signal\nError: {}", stderr).into(),
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
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
"#;
        let result = compile_go(code, "").await;
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_program_with_input() {
        let code = r#"
package main

import "fmt"

func main() {
    var name string
    fmt.Scanln(&name)
    fmt.Printf("Hello, %s!\n", name)
}
"#;
        let result = compile_go(code, "Alice\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_program_with_multiple_inputs() {
        let code = r#"
package main

import "fmt"

func main() {
    var a, b int
    fmt.Scanf("%d %d", &a, &b)
    fmt.Println(a + b)
}
"#;
        let result = compile_go(code, "5 3\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }

    #[tokio::test]
    async fn test_program_with_multiline_input() {
        let code = r#"
package main

import (
    "bufio"
    "fmt"
    "os"
)

func main() {
    scanner := bufio.NewScanner(os.Stdin)
    scanner.Scan()
    line1 := scanner.Text()
    scanner.Scan()
    line2 := scanner.Text()
    fmt.Printf("%s %s\n", line1, line2)
}
"#;
        let result = compile_go(code, "Hello\nWorld\n").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello World");
    }

    #[tokio::test]
    async fn test_compilation_error() {
        let code = r#"
package main

import "fmt"

func main() {
    undefinedFunction()
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_syntax_error() {
        let code = r#"
package main

import "fmt"

func main() {
    fmt.Println("Missing closing quote)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runtime_error() {
        let code = r#"
package main

import (
    "fmt"
    "os"
)

func main() {
    fmt.Println("About to exit with error")
    os.Exit(1)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_program_with_stderr_output() {
        let code = r#"
package main

import (
    "fmt"
    "os"
)

func main() {
    fmt.Println("stdout message")
    fmt.Fprintln(os.Stderr, "stderr message")
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "stdout message");
    }

    #[tokio::test]
    async fn test_program_with_loops() {
        let code = r#"
package main

import "fmt"

func main() {
    for i := 1; i <= 3; i++ {
        fmt.Printf("%d ", i)
    }
    fmt.Println()
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "1 2 3");
    }

    #[tokio::test]
    async fn test_program_with_slices() {
        let code = r#"
package main

import "fmt"

func main() {
    nums := []int{1, 2, 3, 4, 5}
    sum := 0
    for _, num := range nums {
        sum += num
    }
    fmt.Println(sum)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "15");
    }

    #[tokio::test]
    async fn test_empty_program() {
        let code = r#"
package main

func main() {
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[tokio::test]
    async fn test_program_with_switch() {
        let code = r#"
package main

import "fmt"

func main() {
    x := 5
    switch x {
    case 1:
        fmt.Println("one")
    case 2:
        fmt.Println("two")
    default:
        fmt.Println("other")
    }
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "other");
    }

    #[tokio::test]
    async fn test_program_with_struct() {
        let code = r#"
package main

import "fmt"

type Person struct {
    Name string
    Age  int
}

func main() {
    p := Person{Name: "Alice", Age: 30}
    fmt.Printf("Name: %s, Age: %d\n", p.Name, p.Age)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Name: Alice, Age: 30");
    }

    #[tokio::test]
    async fn test_program_with_functions() {
        let code = r#"
package main

import "fmt"

func add(a, b int) int {
    return a + b
}

func multiply(a, b int) int {
    return a * b
}

func main() {
    result := add(multiply(5, 2), 3)
    fmt.Println(result)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "13");
    }

    #[tokio::test]
    async fn test_program_with_map() {
        let code = r#"
package main

import "fmt"

func main() {
    m := make(map[string]int)
    m["hello"] = 5
    m["world"] = 10
    fmt.Println(m["hello"] + m["world"])
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "15");
    }

    #[tokio::test]
    async fn test_program_with_goroutines() {
        let code = r#"
package main

import (
    "fmt"
)

func worker(id int, c chan int) {
    c <- id * 2
}

func main() {
    c := make(chan int)
    go worker(5, c)
    result := <-c
    fmt.Println(result)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "10");
    }

    #[tokio::test]
    async fn test_program_with_error_handling() {
        let code = r#"
package main

import (
    "fmt"
    "strconv"
)

func main() {
    str := "123"
    num, err := strconv.Atoi(str)
    if err != nil {
        fmt.Println("Error:", err)
        return
    }
    fmt.Println(num * 2)
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "246");
    }

    #[tokio::test]
    async fn test_program_with_interfaces() {
        let code = r#"
package main

import "fmt"

type Shape interface {
    Area() float64
}

type Rectangle struct {
    Width, Height float64
}

func (r Rectangle) Area() float64 {
    return r.Width * r.Height
}

func main() {
    var s Shape = Rectangle{Width: 5, Height: 3}
    fmt.Printf("%.0f\n", s.Area())
}
"#;
        let result = compile_go(code, "").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "15");
    }
}
