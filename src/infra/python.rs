use super::error::InfraError;
use std::env;
use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn compile_python(content: &str, stdin_input: &str) -> Result<String, InfraError> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;

    let mut cmd = Command::new("python3")
        .arg(temp_file.path())
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
                    "Python execution failed with status code: {}\nError: {}",
                    code, stderr
                )
                .into(),
            ))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InfraError::CompilationError(
                format!("Python process terminated by signal\nError: {}", stderr).into(),
            ))
        }
    }
}

#[cfg(test)]
mod python_tests {
    use super::*;

    #[tokio::test]
    async fn test_compile_python_basic_output() {
        let content = r#"print("hello world")"#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "hello world");
    }

    #[tokio::test]
    async fn test_compile_python_with_empty_content() {
        let content = "";
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "");
    }

    #[tokio::test]
    async fn test_compile_python_multiple_lines() {
        let content = r#"
print("line 1")
print("line 2")
print("line 3")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "line 1\nline 2\nline 3");
    }

    #[tokio::test]
    async fn test_compile_python_with_stdin() {
        let content = r#"
import sys
input_data = sys.stdin.read().strip()
print(f"Received: {input_data}")
        "#;
        let stdin_input = "hello from stdin";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Received: hello from stdin");
    }

    #[tokio::test]
    async fn test_compile_python_with_input_function() {
        let content = r#"
name = input("Enter your name: ")
print(f"Hello, {name}!")
        "#;
        let stdin_input = "Alice";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Enter your name: Hello, Alice!");
    }

    #[tokio::test]
    async fn test_compile_python_with_empty_stdin() {
        let content = r#"
import sys
input_data = sys.stdin.read()
print(f"Input length: {len(input_data)}")
        "#;
        let stdin_input = "";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Input length: 0");
    }

    #[tokio::test]
    async fn test_compile_python_with_multiline_stdin() {
        let content = r#"
import sys
lines = sys.stdin.read().strip().split('\n')
print(f"Lines: {len(lines)}")
        "#;
        let stdin_input = "line1\nline2\nline3";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Lines: 3");
    }

    #[tokio::test]
    async fn test_compile_python_multiple_inputs() {
        let content = r#"
name = input()
age = input()
print(f"{name} is {age} years old")
        "#;
        let stdin_input = "John\n25";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "John is 25 years old");
    }

    #[tokio::test]
    async fn test_compile_python_syntax_error() {
        let content = r#"
print("unclosed string
        "#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_indentation_error() {
        let content = r#"
if True:
print("bad indentation")
        "#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_runtime_error() {
        let content = r#"
print("before error")
raise ValueError("test error")
print("after error")
        "#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_name_error() {
        let content = r#"print(undefined_variable)"#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_import_error() {
        let content = r#"import nonexistent_module"#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_zero_division_error() {
        let content = r#"
print("before division")
result = 10 / 0
print("after division")
        "#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_numbers() {
        let content = r#"
print(42)
print(3.14)
print(-10)
print(2**10)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "42\n3.14\n-10\n1024");
    }

    #[tokio::test]
    async fn test_compile_python_strings() {
        let content = r#"
print("double quotes")
print('single quotes')
print("""triple quotes""")
print(f"f-string: {2 + 3}")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "double quotes\nsingle quotes\ntriple quotes\nf-string: 5");
    }

    #[tokio::test]
    async fn test_compile_python_booleans() {
        let content = r#"
print(True)
print(False)
print(not True)
print(True and False)
print(True or False)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "True\nFalse\nFalse\nFalse\nTrue");
    }

    #[tokio::test]
    async fn test_compile_python_lists() {
        let content = r#"
arr = [1, 2, 3, 4, 5]
print(arr)
print(len(arr))
print(arr[0])
print(arr[-1])
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "[1, 2, 3, 4, 5]\n5\n1\n5");
    }

    #[tokio::test]
    async fn test_compile_python_dictionaries() {
        let content = r#"
person = {"name": "Alice", "age": 30}
print(person)
print(person["name"])
print(len(person))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "{'name': 'Alice', 'age': 30}\nAlice\n2");
    }

    #[tokio::test]
    async fn test_compile_python_tuples() {
        let content = r#"
coords = (10, 20)
print(coords)
print(coords[0])
print(len(coords))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "(10, 20)\n10\n2");
    }

    #[tokio::test]
    async fn test_compile_python_sets() {
        let content = r#"
numbers = {1, 2, 3, 3, 4, 5}
print(len(numbers))
print(3 in numbers)
print(sorted(numbers))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "5\nTrue\n[1, 2, 3, 4, 5]");
    }

    #[tokio::test]
    async fn test_compile_python_functions() {
        let content = r#"
def greet(name):
    return f"Hello, {name}!"

print(greet("World"))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_compile_python_lambda() {
        let content = r#"
add = lambda x, y: x + y
print(add(5, 3))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "8");
    }

    #[tokio::test]
    async fn test_compile_python_default_parameters() {
        let content = r#"
def greet(name, greeting="Hello"):
    return f"{greeting}, {name}!"

print(greet("Alice"))
print(greet("Bob", "Hi"))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello, Alice!\nHi, Bob!");
    }

    #[tokio::test]
    async fn test_compile_python_args_kwargs() {
        let content = r#"
def sum_all(*args):
    return sum(args)

def print_info(**kwargs):
    for key, value in kwargs.items():
        print(f"{key}: {value}")

print(sum_all(1, 2, 3, 4, 5))
print_info(name="Alice", age=30)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert!(res.contains("15"));
        assert!(res.contains("name: Alice"));
        assert!(res.contains("age: 30"));
    }

    #[tokio::test]
    async fn test_compile_python_if_else() {
        let content = r#"
x = 10
if x > 5:
    print("greater")
else:
    print("lesser")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "greater");
    }

    #[tokio::test]
    async fn test_compile_python_elif() {
        let content = r#"
score = 85
if score >= 90:
    print("A")
elif score >= 80:
    print("B")
elif score >= 70:
    print("C")
else:
    print("F")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "B");
    }

    #[tokio::test]
    async fn test_compile_python_for_loop() {
        let content = r#"
for i in range(3):
    print(f"iteration {i}")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "iteration 0\niteration 1\niteration 2");
    }

    #[tokio::test]
    async fn test_compile_python_while_loop() {
        let content = r#"
i = 0
while i < 3:
    print(f"count {i}")
    i += 1
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "count 0\ncount 1\ncount 2");
    }

    #[tokio::test]
    async fn test_compile_python_list_comprehension() {
        let content = r#"
squares = [x**2 for x in range(5)]
print(squares)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "[0, 1, 4, 9, 16]");
    }

    #[tokio::test]
    async fn test_compile_python_enumerate() {
        let content = r#"
items = ["apple", "banana", "cherry"]
for i, item in enumerate(items):
    print(f"{i}: {item}")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "0: apple\n1: banana\n2: cherry");
    }

    #[tokio::test]
    async fn test_compile_python_classes() {
        let content = r#"
class Person:
    def __init__(self, name, age):
        self.name = name
        self.age = age
    
    def greet(self):
        return f"Hello, I'm {self.name}, {self.age} years old"

person = Person("Alice", 30)
print(person.greet())
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello, I'm Alice, 30 years old");
    }

    #[tokio::test]
    async fn test_compile_python_inheritance() {
        let content = r#"
class Animal:
    def __init__(self, name):
        self.name = name
    
    def speak(self):
        return f"{self.name} makes a sound"

class Dog(Animal):
    def speak(self):
        return f"{self.name} barks"

dog = Dog("Rex")
print(dog.speak())
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Rex barks");
    }

    #[tokio::test]
    async fn test_compile_python_math_module() {
        let content = r#"
import math
print(math.pi)
print(math.sqrt(16))
print(math.factorial(5))
        "#;
        let res = compile_python(content, "").await.unwrap();
        let lines: Vec<&str> = res.trim().split('\n').collect();
        assert!(lines[0].starts_with("3.14"));
        assert_eq!(lines[1], "4.0");
        assert_eq!(lines[2], "120");
    }

    #[tokio::test]
    async fn test_compile_python_random_module() {
        let content = r#"
import random
random.seed(42)
print(random.randint(1, 10))
print(random.choice(['a', 'b', 'c']))
        "#;
        let res = compile_python(content, "").await.unwrap();
        // Results should be deterministic with seed
        let lines: Vec<&str> = res.trim().split('\n').collect();
        assert!(lines[0].parse::<i32>().is_ok());
        assert!(["a", "b", "c"].contains(&lines[1]));
    }

    #[tokio::test]
    async fn test_compile_python_datetime() {
        let content = r#"
from datetime import datetime, timedelta
now = datetime(2024, 1, 1, 12, 0, 0)
print(now.year)
print(now.strftime("%Y-%m-%d"))
future = now + timedelta(days=1)
print(future.day)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "2024\n2024-01-01\n2");
    }

    #[tokio::test]
    async fn test_compile_python_json_module() {
        let content = r#"
import json
data = {"name": "Alice", "age": 30}
json_str = json.dumps(data)
print(json_str)
parsed = json.loads(json_str)
print(parsed["name"])
        "#;
        let res = compile_python(content, "").await.unwrap();
        let lines: Vec<&str> = res.trim().split('\n').collect();
        assert!(lines[0].contains("Alice"));
        assert!(lines[0].contains("30"));
        assert_eq!(lines[1], "Alice");
    }

    #[tokio::test]
    async fn test_compile_python_try_except() {
        let content = r#"
try:
    result = 10 / 0
except ZeroDivisionError:
    print("Cannot divide by zero")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Cannot divide by zero");
    }

    #[tokio::test]
    async fn test_compile_python_try_except_finally() {
        let content = r#"
try:
    print("trying")
    raise ValueError("test error")
except ValueError as e:
    print(f"caught: {e}")
finally:
    print("finally executed")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "trying\ncaught: test error\nfinally executed");
    }

    #[tokio::test]
    async fn test_compile_python_unicode() {
        let content = r#"print("Hello 世界 🐍")"#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello 世界 🐍");
    }

    #[tokio::test]
    async fn test_compile_python_escape_sequences() {
        let content = r#"print("Line 1\nLine 2\tTabbed")"#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Line 1\nLine 2\tTabbed");
    }

    #[tokio::test]
    async fn test_compile_python_raw_strings() {
        let content = r#"print(r"C:\path\to\file")"#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), r"C:\path\to\file");
    }

    #[tokio::test]
    async fn test_compile_python_multiline_strings() {
        let content = r#"
text = """This is a
multiline
string"""
print(text)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "This is a\nmultiline\nstring");
    }

    #[tokio::test]
    async fn test_compile_python_generators() {
        let content = r#"
def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i += 1

for num in count_up_to(3):
    print(num)
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "0\n1\n2");
    }

    #[tokio::test]
    async fn test_compile_python_generator_expression() {
        let content = r#"
squares = (x**2 for x in range(5))
print(list(squares))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "[0, 1, 4, 9, 16]");
    }

    #[tokio::test]
    async fn test_compile_python_decorators() {
        let content = r#"
def uppercase_decorator(func):
    def wrapper(*args, **kwargs):
        result = func(*args, **kwargs)
        return result.upper()
    return wrapper

@uppercase_decorator
def greet(name):
    return f"hello, {name}"

print(greet("alice"))
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "HELLO, ALICE");
    }

    #[tokio::test]
    async fn test_compile_python_context_manager() {
        let content = r#"
class MyContext:
    def __enter__(self):
        print("entering context")
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        print("exiting context")

with MyContext() as ctx:
    print("inside context")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "entering context\ninside context\nexiting context");
    }

    #[tokio::test]
    async fn test_compile_python_large_output() {
        let content = r#"
for i in range(1000):
    print(i)
        "#;
        let res = compile_python(content, "").await.unwrap();
        let lines: Vec<&str> = res.trim().split('\n').collect();
        assert_eq!(lines.len(), 1000);
        assert_eq!(lines[0], "0");
        assert_eq!(lines[999], "999");
    }

    #[tokio::test]
    async fn test_compile_python_no_output() {
        let content = r#"
x = 5
y = 10
result = x + y
# No print statements
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "");
    }

    #[tokio::test]
    async fn test_compile_python_stdin_json() {
        let content = r#"
import json
import sys
data = json.loads(sys.stdin.read())
print(f"{data['name']} is {data['age']} years old")
        "#;
        let stdin_input = r#"{"name": "John", "age": 30}"#;
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "John is 30 years old");
    }

    #[tokio::test]
    async fn test_compile_python_stdin_csv_processing() {
        let content = r#"
import sys
lines = sys.stdin.read().strip().split('\n')
total = 0
for line in lines:
    total += int(line)
print(f"Sum: {total}")
        "#;
        let stdin_input = "10\n20\n30";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Sum: 60");
    }

    #[tokio::test]
    async fn test_compile_python_stdin_word_count() {
        let content = r#"
import sys
text = sys.stdin.read()
words = text.split()
print(f"Word count: {len(words)}")
        "#;
        let stdin_input = "hello world python programming";
        let res = compile_python(content, stdin_input).await.unwrap();
        assert_eq!(res.trim(), "Word count: 4");
    }

    #[tokio::test]
    async fn test_compile_python_sys_exit() {
        let content = r#"
import sys
print("before exit")
sys.exit(2)
print("after exit")
        "#;
        let result = compile_python(content, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_python_type_hints() {
        let content = r#"
from typing import List, Dict

def process_data(items: List[int]) -> Dict[str, int]:
    return {"count": len(items), "sum": sum(items)}

result = process_data([1, 2, 3, 4, 5])
print(f"Count: {result['count']}, Sum: {result['sum']}")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Count: 5, Sum: 15");
    }

    #[tokio::test]
    async fn test_compile_python_async_await() {
        let content = r#"
import asyncio

async def async_greet(name):
    await asyncio.sleep(0.01)  # Small delay
    return f"Hello, {name}!"

async def main():
    result = await async_greet("Alice")
    print(result)

asyncio.run(main())
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_compile_python_walrus_operator() {
        let content = r#"
numbers = [1, 2, 3, 4, 5]
if (n := len(numbers)) > 3:
    print(f"List has {n} items")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "List has 5 items");
    }

    #[tokio::test]
    async fn test_compile_python_fstring_expressions() {
        let content = r#"
name = "Alice"
age = 30
print(f"{name.upper()} is {age * 12} months old")
print(f"Next year: {age + 1}")
        "#;
        let res = compile_python(content, "").await.unwrap();
        assert_eq!(res.trim(), "ALICE is 360 months old\nNext year: 31");
    }
}
