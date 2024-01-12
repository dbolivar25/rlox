# rlox

rlox is a Rust-based interpreter for the Lox programming language. It offers a
comprehensive implementation covering lexical analysis, parsing, environment
management, and interpretation.

## Overview of Modules

### 'ast.rs'

- Function: Defines the abstract syntax tree (AST) for the interpreter.
- Details: Uses macros for flexible AST structure definitions, crucial for
  handling Lox language expressions and statements.

### 'environment.rs'

- Function: Manages the execution environment for variables and scopes.
- Details: Implements scope management using hash maps, essential for variable
  handling in the interpreter.

### 'interpreter.rs'

- Function: The core interpreter logic.
- Details: Integrates lexer, parser, and environment modules. Handles expression
  evaluation and statement execution.

### 'lexer.rs'

- Function: Tokenizes Lox source code.
- Details: Detailed lexical analysis logic for handling various token types.

### 'main.rs'

- Function: Entry point of the application.
- Details: Initializes and starts the interpreter by integrating various
  modules.

### 'parser.rs'

- Function: Parses tokens into an AST.
- Details: Robust parsing logic, capable of handling complex syntactical
  structures.

### 'token.rs'

- Function: Defines token structure and types.
- Details: Fundamental to the lexer and parser modules, outlines syntax elements
  of Lox.

### 'value.rs'

- Function: Defines value types in Lox.
- Details: Handles value evaluation and is key for expression and statement
  operations.

### 'visitor.rs'

- Function: Implements the visitor pattern for AST.
- Details: Separates tree structure from operations, allowing modular and
  flexible design.

## Installation

```
git clone https://github.com/dbolivar25/rlox.git
cd rlox
```

## Usage

### REPL Interpreter

```
cargo run
```

### File Interpreter

```
cargo run -- -f <your_file_name>
```
