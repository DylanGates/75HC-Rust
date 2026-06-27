use anyhow::{Context, Result};
use clap::Parser;
use chrono::Local;
use reqwest::Client;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tokio::sync::Semaphore;

#[derive(Parser)]
#[command(name = "MiniAsyncFetcher", about = "Fetch multiple URLs concurrently")]
struct Cli {
    /// URLs to fetch directly
    #[arg(short = 'u', long)]
    urls: Vec<String>,

    /// File containing URLs (one per line)
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    /// Output format: text or json
    #[arg(short = 'F', long, default_value = "text")]
    format: String,

    /// Max concurrent fetches
    #[arg(short = 'c', long, default_value = "5")]
    concurrency: usize,
}

#[derive(Serialize)]
struct FetchResult {
    url: String,
    status: u16,
    size_bytes: usize,
    duration_ms: u64,
    fetched_at: String,
    error: Option<String>,
}

#[derive(Serialize)]
struct Summary {
    total_urls: usize,
    successes: usize,
    failures: usize,
    total_duration_ms: u64,
    total_bytes: usize,
    results: Vec<FetchResult>,
}

fn detect_content_type(headers: &reqwest::header::HeaderMap) -> String {
    headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

async fn fetch_url(client: &Client, url: &str) -> FetchResult {
    let start = Instant::now();
    let fetched_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let _content_type = detect_content_type(resp.headers());
            match resp.bytes().await {
                Ok(body) => {
                    let duration = start.elapsed().as_millis() as u64;
                    FetchResult {
                        url: url.to_string(),
                        status,
                        size_bytes: body.len(),
                        duration_ms: duration,
                        fetched_at,
                        error: None,
                    }
                }
                Err(e) => {
                    let duration = start.elapsed().as_millis() as u64;
                    FetchResult {
                        url: url.to_string(),
                        status,
                        size_bytes: 0,
                        duration_ms: duration,
                        fetched_at,
                        error: Some(format!("body read error: {e}")),
                    }
                }
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            FetchResult {
                url: url.to_string(),
                status: 0,
                size_bytes: 0,
                duration_ms: duration,
                fetched_at,
                error: Some(format!("request error: {e}")),
            }
        }
    }
}

fn print_text(results: &[FetchResult], total_start: Instant) {
    println!("=== MiniAsyncFetcher Results ===\n");
    for r in results {
        let status_icon = if r.status >= 200 && r.status < 300 {
            "✓"
        } else {
            "✗"
        };
        println!("{status_icon} {} ", r.url);
        println!("   Status: {} | Size: {} bytes | Time: {}ms | At: {}",
            r.status, r.size_bytes, r.duration_ms, r.fetched_at);
        if let Some(ref err) = r.error {
            println!("   Error: {err}");
        }
        println!();
    }

    let successes = results.iter().filter(|r| r.error.is_none() && r.status >= 200 && r.status < 300).count();
    let failures = results.len() - successes;
    let total_bytes: usize = results.iter().map(|r| r.size_bytes).sum();
    let total_ms = total_start.elapsed().as_millis();
    println!("--- Summary ---");
    println!("Total: {} | Success: {successes} | Failures: {failures}", results.len());
    println!("Total bytes: {total_bytes} | Total time: {total_ms}ms");
}

async fn read_urls_from_file(path: &PathBuf) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).await
        .with_context(|| format!("Failed to read URL file: {}", path.display()))?;
    Ok(content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut all_urls = cli.urls;
    if let Some(ref file) = cli.file {
        let file_urls = read_urls_from_file(file).await?;
        all_urls.extend(file_urls);
    }

    if all_urls.is_empty() {
        anyhow::bail!("No URLs provided. Use --urls or --file.");
    }

    println!("Fetching {} URLs (concurrency: {})...\n", all_urls.len(), cli.concurrency);

    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(cli.concurrency));
    let total_start = Instant::now();

    let mut handles = Vec::new();
    for url in all_urls {
        let client = client.clone();
        let sem = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            fetch_url(&client, &url).await
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }

    match cli.format.as_str() {
        "json" => {
            let summary = Summary {
                total_urls: results.len(),
                successes: results.iter().filter(|r| r.error.is_none()).count(),
                failures: results.iter().filter(|r| r.error.is_some()).count(),
                total_duration_ms: total_start.elapsed().as_millis() as u64,
                total_bytes: results.iter().map(|r| r.size_bytes).sum(),
                results,
            };
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        _ => print_text(&results, total_start),
    }

    Ok(())
}
