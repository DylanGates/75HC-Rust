use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(name = "MultiThreadedDownloader", about = "Download a file in concurrent parts")]
struct Cli {
    /// URL to download
    url: String,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Number of concurrent threads (parts)
    #[arg(short, long, default_value = "4")]
    threads: u32,

    /// Minimum part size in bytes (prevents too many tiny parts)
    #[arg(short = 's', long, default_value = "1048576")]
    min_part_size: u64,
}

struct PartProgress {
    part_id: u32,
    bytes_downloaded: Arc<Mutex<u64>>,
    total_bytes: u64,
}

async fn download_part(
    client: Client,
    url: String,
    start: u64,
    end: u64,
    part_id: u32,
    output_path: PathBuf,
    progress: Arc<PartProgress>,
) -> Result<()> {
    let range = format!("bytes={}-{}", start, end);
    let resp = client
        .get(&url)
        .header("Range", &range)
        .send()
        .await
        .with_context(|| format!("Part {part_id}: request failed"))?;

    if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        anyhow::bail!("Part {part_id}: unexpected status {}", resp.status());
    }

    let data = resp
        .bytes()
        .await
        .with_context(|| format!("Part {part_id}: failed to read body"))?;

    // Write to the output file at the correct offset
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&output_path)
        .await
        .with_context(|| format!("Part {part_id}: failed to open output file"))?;

    file.seek(std::io::SeekFrom::Start(start))
        .await
        .with_context(|| format!("Part {part_id}: seek failed"))?;
    file.write_all(&data)
        .await
        .with_context(|| format!("Part {part_id}: write failed"))?;

    // Update progress
    {
        let mut downloaded = progress.bytes_downloaded.lock().await;
        *downloaded += data.len() as u64;
    }

    Ok(())
}

fn get_output_filename(url: &str) -> String {
    let url_path = url.split('?').next().unwrap_or(url);
    let path = PathBuf::from(url_path);
    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let output_path = cli.output.unwrap_or_else(|| PathBuf::from(get_output_filename(&cli.url)));

    println!("Connecting to: {}", cli.url);

    let client = Client::builder()
        .user_agent("MultiThreadedDownloader/0.1")
        .build()?;

    // Get file size via HEAD request
    let head_resp = client.head(&cli.url).send().await?;
    let file_size: u64 = head_resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .with_context(|| "Could not determine file size. Server may not support HEAD.")?;

    // Check if server accepts Range requests
    let accepts_ranges = head_resp
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("bytes"))
        .unwrap_or(false);

    println!("File size: {} bytes", file_size);

    if !accepts_ranges || file_size < cli.min_part_size {
        println!("Server doesn't support Range or file is too small. Downloading in a single part.");
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
                .progress_chars("##-"),
        );

        let resp = client.get(&cli.url).send().await?;
        let data = resp.bytes().await?;
        pb.inc(data.len() as u64);

        fs::write(&output_path, &data).await?;
        pb.finish_with_message("Download complete");
        println!("\nSaved to: {}", output_path.display());
        return Ok(());
    }

    // Calculate part boundaries
    let num_parts = cli.threads.min(
        (file_size / cli.min_part_size).max(1) as u32,
    );

    let part_size = file_size / num_parts as u64;
    let mut parts = Vec::new();
    let mut start = 0u64;
    for i in 0..num_parts {
        let end = if i == num_parts - 1 {
            file_size - 1
        } else {
            start + part_size - 1
        };
        parts.push((start, end));
        start = end + 1;
    }

    println!("Downloading in {num_parts} parts ({} bytes each)...", part_size);

    // Create empty file of the correct size
    {
        let f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&output_path)
            .await?;
        f.set_len(file_size).await?;
    }

    // Progress bar for overall progress
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("##-"),
    );

    let progress = Arc::new(PartProgress {
        part_id: 0,
        bytes_downloaded: Arc::new(Mutex::new(0)),
        total_bytes: file_size,
    });

    let total_start = Instant::now();

    // Spawn download tasks
    let mut handles = Vec::new();
    let parts_owned: Vec<(u64, u64)> = parts.iter().copied().collect();
    for (i, (part_start, part_end)) in parts_owned.iter().enumerate() {
        let client = client.clone();
        let url = cli.url.clone();
        let out = output_path.clone();
        let prog = Arc::clone(&progress);
        let pb_clone = pb.clone();
        let start = *part_start;
        let end = *part_end;

        handles.push(tokio::spawn(async move {
            let result = download_part(
                client, url, start, end, i as u32, out, prog,
            )
            .await;
            pb_clone.inc(end - start + 1);
            result
        }));
    }

    // Wait for all parts
    for handle in handles {
        if let Err(e) = handle.await? {
            pb.finish_with_message("Download failed");
            anyhow::bail!("Part download failed: {e}");
        }
    }

    pb.finish_with_message("Download complete");
    let elapsed = total_start.elapsed();
    println!("\nSaved to: {}", output_path.display());
    println!("Time: {:.2}s | Size: {} bytes", elapsed.as_secs_f64(), file_size);

    Ok(())
}
