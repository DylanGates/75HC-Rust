# Testing Guide — 75HC Rust Projects

This document covers how to build, test, and run each Rust project in the curriculum.

## Prerequisites

- Rust toolchain (install via `rustup`)
- macOS 12+ (for `notify` crate's `macos_kqueue` feature)

## Building All Projects

Each project is a standalone Cargo workspace member. Build individually:

```bash
cd /path/to/75HC-Rust/<project-name>
cargo build           # debug build
cargo build --release # release build
```

## Running Projects

### Level 1–3 (no external dependencies)

```bash
cargo run -- <args>
```

### MiniAsyncFetcher

```bash
cargo run -- --urls https://example.com https://httpbin.org/get
cargo run -- -f urls.txt --format json
```

### HTTPServer

```bash
cargo run -- --port 8080 --dir ./public
cargo run -- --port 8080 --dir ./public --directory-listing
```

### MultiThreadedDownloader

```bash
cargo run -- "https://example.com/file.zip" --threads 8 -o output.zip
```

### CLIPomodoro

```bash
cargo run -- --work 25 --break 5 --cycles 4
# Press Ctrl+C to pause/resume, Ctrl+C twice to quit
```

### TodoAppWithSaving

```bash
cargo run -- add "Buy groceries" --priority high
cargo run -- list
cargo run -- complete <uuid>
cargo run -- stats
```

### MiniCrate

```bash
cargo run -- "hello world" --operations truncate,reverse
cargo run -- "camelCase" --operations snake_case
```

### SimpleStaticSiteGenerator

```bash
cargo run -- --input ./content --output ./dist
cargo run -- --watch  # rebuild on file changes
```

## Running Tests

Projects with test suites:

| Project | Command |
|---------|---------|
| MiniCrate (lib) | `cargo test` |
| SimpleStaticSiteGenerator | `cargo test` (if added) |

For MiniCrate, tests cover all seven string utility functions.

## Performance Testing

For the async/download projects, consider:

- **MiniAsyncFetcher**: Test with varying concurrency levels (`-c 1`, `-c 10`, `-c 50`)
- **MultiThreadedDownloader**: Test with different thread counts (`--threads 2`, `--threads 16`)
- **HTTPServer**: Benchmark with `wrk` or `hey`:

```bash
hey -n 1000 -c 10 http://localhost:8080/
```

## Common Build Issues

### macOS Kqueue for `notify`

The `SimpleStaticSiteGenerator` uses `notify` with `macos_kqueue` feature. If you
encounter build issues on newer macOS versions, try removing the feature flag or
switching to the `fsevent` feature.

### OpenSSL / `reqwest` TLS

On macOS, `reqwest` uses native TLS (`Security.framework`). No additional setup
is needed. On Linux, you may need `libssl-dev`.

## Coverage

For detailed coverage analysis, install `cargo-tarpaulin`:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --ignore-tests
```
