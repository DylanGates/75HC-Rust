# 🦀 75 Hard Code Challenge: Rust Projects

This repository contains **21 mini-projects** built in **Rust** as part of the 75-Day Coding Challenge. Covers command-line utilities, web servers, database backends, concurrent programming, and async operations.

## Challenge Focus

- **System Tools:** Fast, efficient CLI applications.
- **Concurrency:** `async`/`await`, threads, channels, `rayon`.
- **Crates Ecosystem:** `clap`, `reqwest`, `serde`, `tokio`, `axum`, `sqlx`, etc.

## Repository Structure

Each project lives in its own self-contained folder:

```bash
75HC-Rust/
├── ProjectName/
│   ├── Cargo.toml
│   └── src/main.rs
└── README.md
```

## Rust Project Tracker

| Day | Week | Project | Folder |
| :-: | :--: | :------ | :----- |
| D1  | Warm-Up | CLI Greeter | [`CLIGreeter`](./CLIGreeter) |
| D5  | Warm-Up | Temperature Converter (C/F/K) | [`temp_converter`](./temp_converter) |
| D9  | Warm-Up | Word/char counter w/ parallelism | [`word_counter`](./word_counter) |
| D13 | Warm-Up | Guessing game | [`guess_game`](./guess_game) |
| D17 | Warm-Up | Mini To-Do CLI | [`todo_cli`](./todo_cli) |
| D21 | Warm-Up | Logging utility w/ web UI | [`logger`](./logger) |
| D25 | Intermediate | Config reader (TOML/YAML/JSON/env) | [`config_reader`](./config_reader) |
| D29 | Intermediate | URL shortener w/ aliases & hit tracking | [`url_shortener`](./url_shortener) |
| D33 | Intermediate | JSON file CRUD database | [`simple_crud`](./simple_crud) |
| D37 | Intermediate | Habit tracker CLI | [`habit_tracker`](./habit_tracker) |
| D41 | Intermediate | Password generator w/ entropy | [`password_manager`](./password_manager) |
| D45 | Intermediate | Async URL fetcher (concurrent) | [`MiniAsyncFetcher`](./MiniAsyncFetcher) |
| D49 | Advanced | Static file HTTP server | [`HTTPServer`](./HTTPServer) |
| D53 | Advanced | Multi-threaded downloader | [`MultiThreadedDownloader`](./MultiThreadedDownloader) |
| D57 | Advanced | Pomodoro timer CLI | [`CLIPomodoro`](./CLIPomodoro) |
| D61 | Advanced | Persistent todo app (JSON) | [`TodoAppWithSaving`](./TodoAppWithSaving) |
| D65 | Advanced | String utility lib + CLI | [`MiniCrate`](./MiniCrate) |
| D69 | Final | Static site generator (Markdown → HTML) | [`SimpleStaticSiteGenerator`](./SimpleStaticSiteGenerator) |
| —  | Backend | Book API (Axum + SQLx + PostgreSQL) | [`backend_db`](./backend_db) |
| —  | Backend | Book API (Axum + MongoDB + Redis + Docker) | [`backend_mongo`](./backend_mongo) |
| D73 | Final | Architecture & testing docs | [`CapstonePolish`](./CapstonePolish) |

All projects compile (tested) and are ready to run with `cargo run`. Build artifacts are gitignored.
