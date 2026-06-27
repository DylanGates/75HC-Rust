# CLIGreeter

A friendly Rust CLI application that greets users based on their name and asks for their age.

## Features

- **Polite Greeting Detection**: Analyzes name capitalization to determine greeting style
- **Interactive Input**: Prompts for name and age
- **Input Validation**: Validates age input and provides appropriate responses
- **Two Greeting Styles**:
  - Polite: For properly capitalized names (e.g., "John", "Mary Smith")
  - Rude: For names that don't follow proper capitalization

## Usage

Run the application:

```bash
cargo run
```

The application will:

1. Ask for your name
2. Analyze the name's capitalization
3. Provide an appropriate greeting
4. Ask for your age
5. Validate and display the age

### Example Interaction

```
Nice to meet you!
What is your name? (e.g., John or mary)

John

Hello, nice to meet you John!

What is your age?

25

You are 25 years old!
```

### Name Analysis Rules

- **Polite**: Names with proper capitalization (first letter uppercase, rest lowercase)
- **Rude**: Names with incorrect capitalization or containing non-alphabetic characters

## Building

```bash
cargo build --release
```

## Requirements

- Rust 2024 edition or later

## License

This project is open source. Feel free to use and modify as needed.
