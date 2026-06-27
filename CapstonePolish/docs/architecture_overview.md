# Architecture Overview — 75HC Rust Projects

This document provides a high-level architectural summary of all 20 Rust projects in the
75HC Learning curriculum. Each project is categorized by theme and complexity to help
navigate the codebase.

## Project Index

### Level 1 — Fundamentals (Projects 1–5)

| # | Name | Area |
|---|------|------|
| 1 | HelloWorld | CLI basics |
| 2 | VariablesMath | Types, control flow |
| 3 | StringParser | Ownership, borrowing |
| 4 | OwnershipDemo | Ownership deep-dive |
| 5 | StructSandbox | Structs, enums, pattern matching |

These establish Rust syntax, ownership model, and basic program structure. Each is a
single `main.rs` with no external dependencies.

### Level 2 — Intermediate (Projects 6–10)

| # | Name | Area |
|---|------|------|
| 6 | ErrorHandling | `Result`, `Option`, `anyhow` |
| 7 | FileReader | File I/O, `BufRead` |
| 8 | CLITool | `clap`, argument parsing |
| 9 | Iterators | Iterator combinators, closures |
| 10 | Collections | `HashMap`, `Vec`, sorting |

Introduction to external crates (`anyhow`, `clap`), file system interaction, and
functional-style iteration.

### Level 3 — Applied (Projects 11–15)

| # | Name | Area |
|---|------|------|
| 11 | MiniGrep | File search tool (like grep) |
| 12 | CSVProcessor | CSV reading/writing |
| 13 | CRUDApp | In-memory CRUD with `HashMap` |
| 14 | Calculator | Expression evaluator |
| 15 | PasswordGenerator | Random generation, entropy |

Projects integrate multiple concepts: error handling, file I/O, CLI, data structures.
MiniGrep is a simplified `ripgrep`-style tool.

### Level 4 — Advanced (Projects 16–20)

| # | Name | Area |
|---|------|------|
| 16 | MiniAsyncFetcher | `tokio`, `reqwest`, concurrent HTTP |
| 17 | HTTPServer | `tiny_http`, static serving |
| 18 | MultiThreadedDownloader | Range requests, concurrent downloads |
| 19 | CLIPomodoro | Timer, ctrlc signal handling |
| 20 | TodoAppWithSaving | Persistent JSON storage, CRUD CLI |

Async programming with `tokio`, network services, and persistent state management.

### Capstone Projects

| # | Name | Area |
|---|------|------|
| C1 | MiniCrate | Library + CLI, string utilities |
| C2 | SimpleStaticSiteGenerator | Markdown → HTML, SSG |
| C3 | CapstonePolish | Documentation, testing, refactoring |

The capstone projects integrate everything: library design, file processing pipelines,
template systems, file watching, and comprehensive documentation.

## Architectural Patterns

### CLI Entry Point

Every project follows the same `clap` derive pattern:

```rust
#[derive(Parser)]
#[command(name = "project-name")]
struct Cli { ... }

fn main() -> Result<()> {
    let cli = Cli::parse();
    // ...
}
```

### Error Handling

All projects use `anyhow::Result` for error propagation, with `anyhow::bail!` and
`Context` for rich error messages.

### Async Pattern

Async projects use `tokio::main` and `tokio::spawn` for concurrency, with `Semaphore`
for rate limiting where needed.

## Build & Test

Each project is standalone. Build with:

```bash
cd <project-dir>
cargo build
cargo test    # where applicable
```

See [testing_guide.md](testing_guide.md) for detailed testing instructions.
