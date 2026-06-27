# Refactoring Report — 75HC Rust Projects

This report summarizes cross-cutting improvement opportunities, code quality
observations, and suggested refactoring priorities for the 75HC Rust codebase.

## 1. Cross-Cutting Observations

### 1.1 Consistency

- **CLI naming**: Most projects use `clap` derive with `#[command(name = ...)]`, but
  a few are inconsistent in their `about` descriptions. Standardize to short,
  imperative descriptions.
- **Error messages**: Some projects use `eprintln!` for errors while others return
  `anyhow::Result`. Prefer returning `Result` for library errors and `eprintln!`
  only for fatal startup checks.
- **File naming**: All source files follow `src/main.rs` (or `src/lib.rs`). This is
  consistent and correct.

### 1.2 Shared Dependencies

Several projects share the same dependency set (`anyhow`, `clap`, `serde`).
Consider creating a workspace-level Cargo.toml with shared dependency declarations:

```toml
[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
```

This ensures all projects use the same versions and simplifies updates.

### 1.3 Code Duplication

- **MiniAsyncFetcher** and **MultiThreadedDownloader** both implement HTTP client
  setup with `reqwest`. Extract a shared utility crate for HTTP helpers.
- **HTTPServer** MIME detection (`mime_guess`) could be a shared utility.
- **TodoAppWithSaving** JSON persistence pattern is reusable across projects.
  Extract a `persist` module or standalone crate.

## 2. Project-Specific Recommendations

### MiniAsyncFetcher

- Add timeout configuration per URL to prevent hanging on slow servers.
- Consider streaming response bodies instead of buffering all bytes in memory
  for large payloads.
- The `detect_content_type` function is unused in the `FetchResult` — either
  include it in the output or remove it.

### HTTPServer

- The `sanitize_path` function prevents directory traversal for simple cases, but
  should canonicalize the path and verify it's under the serve directory.
- Add logging via a crate like `log` + `env_logger` instead of raw `println!`.
- Consider adding ETag / If-Modified-Since support for caching.

### MultiThreadedDownloader

- The `PartProgress` struct tracks progress but the overall progress bar update
  is done separately. Merge the two for a single source of truth.
- Add resume support (download partial file and continue).
- Handle HTTP redirects explicitly (set `reqwest::Client` to follow up to N redirects).

### CLIPomodoro

- The `wait_for_resume` busy-waits with 100ms polling. Use a `Condvar` or channel
  for a more efficient wake-up.
- The double-Ctrl-C quit pattern is fragile. Consider prompting "Press Ctrl+C again
  to quit" with a 2-second window instead.

### TodoAppWithSaving

- Add data validation before saving (e.g., reject empty descriptions).
- Consider atomic writes (write to temp file, then rename) to prevent corruption.
- The `load_data` function silently creates empty data if the file doesn't exist
  but errors on invalid JSON — this is the correct behavior.
- Add pagination for large task lists.

### MiniCrate

- The `to_snake_case` and `to_camel_case` functions handle basic cases but may
  not cover Unicode or edge cases (e.g., consecutive underscores). Add more tests.
- Unicode support: `truncate` counts chars, not grapheme clusters. For emoji-heavy
  strings, consider using the `unicode-segmentation` crate.

### SimpleStaticSiteGenerator

- The frontmatter parser supports both JSON and simple key-value formats. This is
  pragmatic but fragile — standardize on one format (YAML or TOML would be better
  but adds dependencies).
- The `notify` watcher rebuilds the entire site on any change. For large sites,
  implement incremental builds (only rebuild changed files).
- Add sitemap generation.

## 3. Priority Matrix

| Priority | Change | Effort | Impact |
|----------|--------|--------|--------|
| P0 | Path traversal fix in HTTPServer | Low | Security |
| P0 | Atomic writes in TodoAppWithSaving | Low | Data integrity |
| P1 | Shared dep versions via workspace | Medium | Maintainability |
| P1 | Timeout in MiniAsyncFetcher | Low | Robustness |
| P2 | Incremental builds in SSG | High | Performance |
| P2 | Unicode support in MiniCrate | Medium | Correctness |
| P3 | HTTP helper crate extraction | High | Code reuse |

## 4. Proposed Workspace Structure

```text
75HC-Rust/
├── Cargo.toml              # workspace root
├── shared-http/            # shared HTTP utilities
├── shared-persist/         # shared persistence utilities
├── MiniAsyncFetcher/
├── HTTPServer/
├── MultiThreadedDownloader/
├── CLIPomodoro/
├── TodoAppWithSaving/
├── MiniCrate/
├── SimpleStaticSiteGenerator/
└── CapstonePolish/
```

A workspace root `Cargo.toml` would declare all members and shared dependencies,
making `cargo build --workspace` build everything at once.
