import { loadLanguage } from "@uiw/codemirror-extensions-langs";
import { elixir } from "codemirror-lang-elixir";
import { StreamLanguage } from "@codemirror/language"
import { fSharp, oCaml } from "@codemirror/legacy-modes/mode/mllike";
import { objectiveC } from "@codemirror/legacy-modes/mode/clike";

export const languages = [
    {
        name: 'C',
        value: 'c',
        defaultCode: '#include <stdio.h>\n\nint main() {\n    printf("Hello, World!\\n");\n    return 0;\n}',
        extension: loadLanguage('c')
    },
    {
        name: 'C++',
        value: 'cpp',
        defaultCode: '#include <iostream>\nusing namespace std;\n\nint main() {\n    cout << "Hello, World!" << endl;\n    return 0;\n}',
        extension: loadLanguage('cpp')
    },
    {
        name: 'Python',
        value: 'python',
        defaultCode: 'print("Hello, World!")',
        extension: loadLanguage('python')
    },
    {
        name: 'Java',
        value: 'java',
        defaultCode: 'public class Main {\n    public static void main(String[] args) {\n        System.out.println("Hello, World!");\n    }\n}',
        extension: loadLanguage('java')
    },
    {
        name: 'JavaScript',
        value: 'javascript',
        defaultCode: 'console.log("Hello, World!");',
        extension: loadLanguage('javascript')
    },
    {
        name: 'Ruby',
        value: 'ruby',
        defaultCode: 'puts "Hello, World!"',
        extension: loadLanguage('ruby')
    },
    {
        name: 'Go',
        value: 'go',
        defaultCode: 'package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello, World!")\n}',
        extension: loadLanguage('go')
    },
    {
        name: 'Rust',
        value: 'rust',
        defaultCode: 'fn main() {\n    println!("Hello, World!");\n}',
        extension: loadLanguage('rust')
    },
    {
        name: 'Swift',
        value: 'swift',
        defaultCode: 'print("Hello, World!")',
        extension: loadLanguage('swift')
    },
    {
        name: 'Kotlin',
        value: 'kotlin',
        defaultCode: 'fun main() {\n    println("Hello, World!")\n}',
        extension: loadLanguage('kotlin')
    },
    {
        name: 'PHP',
        value: 'php',
        defaultCode: '<?php\n\necho "Hello, World!\\n";\n',
        extension: loadLanguage('php')
    },
    {
        name: 'Perl',
        value: 'perl',
        defaultCode: 'print "Hello, World!\\n";',
        extension: loadLanguage('perl')
    },
    {
        name: 'Lua',
        value: 'lua',
        defaultCode: 'print("Hello, World!")',
        extension: loadLanguage('lua')
    },
    {
        name: 'R',
        value: 'r',
        defaultCode: 'cat("Hello, World!\\n")',
        extension: loadLanguage('r')
    },
    {
        name: 'MATLAB',
        value: 'matlab',
        defaultCode: 'disp("Hello, World!")',
        extension: loadLanguage('matlab')
    },
    {
        name: 'Fortran',
        value: 'fortran',
        defaultCode: 'program hello\n    print *, "Hello, World!"\nend program hello',
        extension: loadLanguage('fortran')
    },
    {
        name: 'Ada',
        value: 'ada',
        defaultCode: 'with Ada.Text_IO; use Ada.Text_IO;\n\nprocedure Hello is\nbegin\n    Put_Line("Hello, World!");\nend Hello;',
        extension: loadLanguage('c')
    },
    {
        name: 'Pascal',
        value: 'pascal',
        defaultCode: 'program Hello;\nbegin\n    writeln("Hello, World!");\nend.',
        extension: loadLanguage('pascal')
    },
    {
        name: 'Delphi',
        value: 'delphi',
        defaultCode: 'program Hello;\nuses SysUtils;\nbegin\n    Writeln("Hello, World!");\nend.',
        extension: loadLanguage('tsx')
    },
    {
        name: 'C#',
        value: 'csharp',
        defaultCode: 'using System;\n\nclass Program {\n    static void Main() {\n        Console.WriteLine("Hello, World!");\n    }\n}',
        extension: loadLanguage('csharp')
    },
    {
        name: 'Visual Basic',
        value: 'vb',
        defaultCode: 'Module Program\n    Sub Main()\n        Console.WriteLine("Hello, World!")\n    End Sub\nEnd Module',
        extension: loadLanguage('vb')
    },
    {
        name: 'Dart',
        value: 'dart',
        defaultCode: 'void main() {\n    print("Hello, World!");\n}',
        extension: loadLanguage('dart')
    },
    {
        name: 'Elixir',
        value: 'elixir',
        defaultCode: 'IO.puts "Hello, World!"',
        extension: elixir()
    },
    {
        name: 'Erlang',
        value: 'erlang',
        defaultCode: '-module(hello).\n-export([main/0]).\n\nmain() ->\n    io:format("Hello, World!~n").',
        extension: loadLanguage('erlang')
    },
    {
        name: 'Haskell',
        value: 'haskell',
        defaultCode: 'main :: IO ()\nmain = putStrLn "Hello, World!"',
        extension: loadLanguage('haskell')
    },
    {
        name: 'Scala',
        value: 'scala',
        defaultCode: 'object Hello {\n    def main(args: Array[String]): Unit = {\n        println("Hello, World!")\n    }\n}',
        extension: loadLanguage('scala')
    },
    {
        name: 'Clojure',
        value: 'clojure',
        defaultCode: '(println "Hello, World!")',
        extension: loadLanguage('clojure')
    },
    {
        name: 'Groovy',
        value: 'groovy',
        defaultCode: 'println "Hello, World!"',
        extension: loadLanguage('groovy')
    },
    {
        name: 'F#',
        value: 'fsharp',
        defaultCode: 'printfn "Hello, World!"',
        extension: StreamLanguage.define(fSharp)
    },
    {
        name: 'Objective-C',
        value: 'objc',
        defaultCode: '#import <Foundation/Foundation.h>\n\nint main() {\n    NSLog(@"Hello, World!");\n    return 0;\n}',
        extension: StreamLanguage.define(objectiveC)
    },
    {
        name: 'D',
        value: 'd',
        defaultCode: 'import std.stdio;\n\nvoid main() {\n    writeln("Hello, World!");\n}',
        extension: loadLanguage('d')
    },
    {
        name: 'Crystal',
        value: 'crystal',
        defaultCode: 'puts "Hello, World!"',
        extension: loadLanguage('crystal')
    },
    {
        name: 'Julia',
        value: 'julia',
        defaultCode: 'println("Hello, World!")',
        extension: loadLanguage('julia')
    },
    {
        name: 'Scheme',
        value: 'scheme',
        defaultCode: '(display "Hello, World!\\n")',
        extension: loadLanguage('scheme')
    },
    {
        name: 'Commong Lisp',
        value: 'common lisp',
        defaultCode: '(format t "Hello, World!~%")',
        extension: loadLanguage('commonLisp')
    },
    {
        name: 'OCaml',
        value: 'ocaml',
        defaultCode: 'print_endline "Hello, World!"',
        extension: StreamLanguage.define(oCaml)
    },
    {
        name: 'Eiffel',
        value: 'eiffel',
        defaultCode: 'class HELLO\ncreate\n    make\nfeature\n    make\n        do\n            io.put_string ("Hello, World!%N")\n        end\nend',
        extension: loadLanguage('eiffel')
    },
    {
        name: 'Nim',
        value: 'nim',
        defaultCode: 'echo "Hello, World!"',
        extension: null
    },
    {
        name: 'Zig',
        value: 'zig',
        defaultCode: 'const std = @import("std");\n\npub fn main() !void {\n    std.debug.print("Hello, World!\\n", .{});\n}',
        extension: loadLanguage('c')
    },
    {
        name: 'V',
        value: 'v',
        defaultCode: 'fn main() {\n    println("Hello, World!")\n}',
        extension: loadLanguage('rust')
    },
    {
        name: 'Haxe',
        value: 'haxe',
        defaultCode: 'class Main {\n    static function main() {\n        trace("Hello, World!");\n    }\n}',
        extension: loadLanguage('haxe')
    },
    {
        name: 'Tcl',
        value: 'tcl',
        defaultCode: 'puts "Hello, World!"',
        extension: loadLanguage('tcl')
    },
    {
        name: 'Forth',
        value: 'forth',
        defaultCode: '.( Hello, World!) CR',
        extension: loadLanguage('forth')
    },
    {
        name: 'Smalltalk',
        value: 'smalltalk',
        defaultCode: 'Transcript show: \'Hello, World!\'; cr.',
        extension: loadLanguage('smalltalk')
    },
    {
        name: 'Prolog',
        value: 'prolog',
        defaultCode: ':- write(\'Hello, World!\'), nl.',
        extension: loadLanguage('c')
    },
    {
        name: 'Bash',
        value: 'bash',
        defaultCode: 'echo "Hello, World!"',
        extension: loadLanguage('shell')
    },
    {
        name: 'PowerShell',
        value: 'powershell',
        defaultCode: 'Write-Output "Hello, World!"',
        extension: loadLanguage('powershell')
    },
    {
        name: 'AWK',
        value: 'awk',
        defaultCode: 'BEGIN { print "Hello, World!" }',
        extension: loadLanguage('shell')
    },
    {
        name: 'COBOL',
        value: 'cobol',
        defaultCode: '       IDENTIFICATION DIVISION.\n       PROGRAM-ID. Hello.\n       PROCEDURE DIVISION.\n           DISPLAY "Hello, World!".\n           STOP RUN.',
        extension: loadLanguage('cobol')
    },
    {
        name: 'Assembly',
        value: 'asm',
        defaultCode: 'section .text\n    global _start\n\n_start:\n    mov rax, 1\n    mov rdi, 1\n    mov rsi, message\n    mov rdx, 13\n    syscall\n    mov rax, 60\n    xor rdi, rdi\n    syscall\n\nsection .data\nmessage: db "Hello, World!", 10',
        extension: loadLanguage('c')
    },
    {
        name: 'TypeScript',
        value: 'typescript',
        defaultCode: 'const message: string = "Hello, World!";\nconsole.log(message);',
        extension: loadLanguage('typescript')
    },
    {
        name: 'CoffeeScript',
        value: 'coffeescript',
        defaultCode: 'console.log "Hello, World!"',
        extension: loadLanguage('coffeescript')
    },
    {
        name: 'Elm',
        value: 'elm',
        defaultCode: 'module Main exposing (..)\nimport Debug\n\nmain =\n    Debug.log "Hello, World!" ()',
        extension: loadLanguage('elm')
    },
    {
        name: 'PureScript',
        value: 'purescript',
        defaultCode: 'module Main where\n\nimport Effect.Console (log)\n\nmain = log "Hello, World!"',
        extension: loadLanguage('tsx')
    },
    {
        name: 'BASIC',
        value: 'basic',
        defaultCode: '10 PRINT "Hello, World!"\n20 END',
        extension: loadLanguage('vb')
    },
    {
        name: 'Vala',
        value: 'vala',
        defaultCode: 'void main() {\n    stdout.printf("Hello, World!\\n");\n}',
        extension: loadLanguage('c')
    },
    {
        name: 'COOL',
        value: 'cool',
        defaultCode: 'class Main inherits IO {\n    main() : Object {\n        out_string("Hello, World!\\n")\n    };\n};',
        extension: loadLanguage('cpp')
    },
    {
        name: 'Hack',
        value: 'hack',
        defaultCode: '<?hh\n\necho "Hello, World!\\n";',
        extension: loadLanguage('c')
    },
    {
        name: 'APL',
        value: 'apl',
        defaultCode: '⎕←\'Hello, World!\'',
        extension: loadLanguage('apl')
    },
    {
        name: 'Nix',
        value: 'nix',
        defaultCode: 'builtins.trace "Hello, World!" 42',
        extension: loadLanguage('nix')
    },
    {
        name: 'ALGOL 68',
        value: 'algol68',
        defaultCode: 'main: (\n    print("Hello, World!\\n")\n)',
        extension: loadLanguage('c')
    },
    {
        name: 'Factor',
        value: 'factor',
        defaultCode: 'USE: io\n"Hello, World!" print',
        extension: loadLanguage('factor')
    }
];
