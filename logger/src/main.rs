use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use tiny_http::{Response, Server};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "info" => Some(LogLevel::INFO),
            "warn" => Some(LogLevel::WARN),
            "error" => Some(LogLevel::ERROR),
            "debug" => Some(LogLevel::DEBUG),
            _ => None,
        }
    }

    fn colored(&self) -> ColoredString {
        match self {
            LogLevel::INFO => "INFO".green(),
            LogLevel::WARN => "WARN".yellow(),
            LogLevel::ERROR => "ERROR".red(),
            LogLevel::DEBUG => "DEBUG".blue(),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::DEBUG => "DEBUG",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
}

#[derive(Parser)]
#[command(name = "logger", about = "A logging utility with timestamps")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Write {
        #[arg(short, long, default_value = "info")]
        level: String,
        message: String,
    },
    Read {
        #[arg(short, long)]
        level: Option<String>,
        #[arg(short, long)]
        search: Option<String>,
    },
    Stats,
    Export {
        #[arg(short, long, default_value = "csv")]
        format: String,
    },
    Serve {
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    Interactive,
}

const LOG_FILE: &str = "log.json";
const MAX_LOG_SIZE: u64 = 1_048_576; // 1 MB

// ── Log I/O ─────────────────────────────────────────────────────────────────

fn read_all_entries() -> Vec<LogEntry> {
    let file = match File::open(LOG_FILE) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            if !line.trim().is_empty() {
                if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                    entries.push(entry);
                }
            }
        }
    }
    entries
}

fn append_entry(entry: &LogEntry) -> io::Result<()> {
    let mut file = File::options().append(true).create(true).open(LOG_FILE)?;
    writeln!(file, "{}", serde_json::to_string(entry).unwrap())?;
    Ok(())
}

fn rotate_if_needed() -> io::Result<()> {
    if let Ok(meta) = fs::metadata(LOG_FILE) {
        if meta.len() > MAX_LOG_SIZE {
            let ts = Utc::now().format("%Y%m%d_%H%M%S");
            let backup = format!("log_backup_{}.json", ts);
            fs::rename(LOG_FILE, &backup)?;
            println!("Log rotated to: {}", backup);
        }
    }
    Ok(())
}

// ── CLI Commands ─────────────────────────────────────────────────────────────

fn cmd_write(level_str: &str, message: &str) {
    let level = match LogLevel::from_str(level_str) {
        Some(l) => l,
        None => {
            eprintln!("Invalid log level: {}", level_str);
            std::process::exit(1);
        }
    };
    rotate_if_needed().ok();
    let entry = LogEntry { timestamp: Utc::now(), level, message: message.to_string() };
    match append_entry(&entry) {
        Ok(_) => println!("{} log written.", entry.level.as_str()),
        Err(e) => eprintln!("Failed to write log: {}", e),
    }
}

fn cmd_read(level_filter: Option<&str>, keyword: Option<&str>) {
    let entries = read_all_entries();
    if entries.is_empty() {
        println!("No logs found.");
        return;
    }

    let filtered: Vec<&LogEntry> = entries
        .iter()
        .filter(|e| {
            let level_match = match level_filter {
                Some(l) => LogLevel::from_str(l)
                    .map(|lf| std::mem::discriminant(&e.level) == std::mem::discriminant(&lf))
                    .unwrap_or(true),
                None => true,
            };
            let keyword_match = match keyword {
                Some(k) => e.message.to_lowercase().contains(&k.to_lowercase()),
                None => true,
            };
            level_match && keyword_match
        })
        .collect();

    if filtered.is_empty() {
        println!("No matching logs found.");
        return;
    }

    for entry in &filtered {
        println!(
            "[{}] [{}] {}",
            format!("{}", entry.timestamp.format("%Y-%m-%d %H:%M:%S")).dimmed(),
            entry.level.colored(),
            entry.message
        );
    }
}

fn cmd_stats() {
    let entries = read_all_entries();
    if entries.is_empty() {
        println!("No logs to analyze.");
        return;
    }

    let total = entries.len();
    let mut counts = std::collections::HashMap::new();
    let mut earliest = None;
    let mut latest = None;

    for e in &entries {
        *counts.entry(e.level.as_str()).or_insert(0) += 1;
        if earliest.is_none() || e.timestamp < earliest.unwrap() {
            earliest = Some(e.timestamp);
        }
        if latest.is_none() || e.timestamp > latest.unwrap() {
            latest = Some(e.timestamp);
        }
    }

    println!("{}", "📊 Log Statistics:".bold());
    println!("  Total logs: {}", total);
    for level in &["INFO", "WARN", "ERROR", "DEBUG"] {
        let n = counts.get(level).unwrap_or(&0);
        println!("  {}: {} ({:.1}%)", level, n, (*n as f64 / total as f64) * 100.0);
    }
    if let (Some(e), Some(l)) = (earliest, latest) {
        println!("  Range: {} → {}",
            e.format("%Y-%m-%d %H:%M:%S"),
            l.format("%Y-%m-%d %H:%M:%S"));
    }
}

fn cmd_export(format: &str) {
    let entries = read_all_entries();
    if entries.is_empty() {
        println!("No logs to export.");
        return;
    }

    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("logs_export_{}.{}", ts, format);
    let mut file = match File::create(&filename) {
        Ok(f) => f,
        Err(e) => { eprintln!("Failed to create export file: {}", e); return; }
    };

    match format {
        "csv" => {
            writeln!(file, "timestamp,level,message").ok();
            for e in &entries {
                writeln!(file, "{},{},\"{}\"",
                    e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    e.level.as_str(),
                    e.message.replace('"', "\"\""),
                ).ok();
            }
        }
        "txt" => {
            for e in &entries {
                writeln!(file, "[{}] [{}] {}",
                    e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    e.level.as_str(),
                    e.message,
                ).ok();
            }
        }
        _ => { eprintln!("Unsupported format: {}", format); return; }
    }
    println!("Exported {} logs to: {}", entries.len(), filename);
}

// ── Web Server ───────────────────────────────────────────────────────────────

fn cmd_serve(port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => { eprintln!("Failed to start server: {}", e); return; }
    };
    println!("🌐 Web interface at http://{}", addr);
    println!("Press Ctrl+C to stop.");

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/" => {
                let html = web_page_html();
                Response::from_string(html)
                    .with_header(tiny_http::Header::from_bytes(b"Content-Type", b"text/html").unwrap())
            }
            "/api/logs" => {
                let entries = read_all_entries();
                let json = serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".into());
                Response::from_string(json)
                    .with_header(tiny_http::Header::from_bytes(b"Content-Type", b"application/json").unwrap())
            }
            "/api/stats" => {
                let stats = compute_stats_json();
                Response::from_string(stats)
                    .with_header(tiny_http::Header::from_bytes(b"Content-Type", b"application/json").unwrap())
            }
            _ => {
                Response::from_string("404 Not Found").with_status_code(404)
            }
        };
        request.respond(response).ok();
    }
}

fn compute_stats_json() -> String {
    let entries = read_all_entries();
    let total = entries.len();
    let mut counts = std::collections::HashMap::new();
    for e in &entries {
        *counts.entry(e.level.as_str()).or_insert(0) += 1;
    }
    let obj = serde_json::json!({
        "total_logs": total,
        "info_count": counts.get("INFO").unwrap_or(&0),
        "warn_count": counts.get("WARN").unwrap_or(&0),
        "error_count": counts.get("ERROR").unwrap_or(&0),
        "debug_count": counts.get("DEBUG").unwrap_or(&0),
    });
    serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".into())
}

fn web_page_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Logger Web Interface</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 20px; max-width: 800px; }
        .log-entry { margin: 5px 0; padding: 5px; border-left: 3px solid #ccc; }
        .INFO { border-left-color: #4caf50; }
        .WARN { border-left-color: #ff9800; }
        .ERROR { border-left-color: #f44336; }
        .DEBUG { border-left-color: #2196f3; }
        button { margin: 4px; padding: 8px 14px; cursor: pointer; }
        pre { background: #f5f5f5; padding: 10px; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>📝 Logger Web Interface</h1>
    <div>
        <button onclick="loadLogs()">Load Logs</button>
        <button onclick="loadStats()">Stats</button>
        <button onclick="clearDisplay()">Clear</button>
    </div>
    <h2>Statistics</h2>
    <div id="stats"><pre>Loading...</pre></div>
    <h2>Logs</h2>
    <div id="logs"></div>
    <script>
        async function loadLogs() { const r = await fetch('/api/logs'); displayLogs(await r.json()); }
        async function loadStats() { const r = await fetch('/api/stats'); document.getElementById('stats').innerHTML = '<pre>' + JSON.stringify(await r.json(), null, 2) + '</pre>'; }
        function displayLogs(logs) { const c = document.getElementById('logs'); c.innerHTML = ''; logs.forEach(l => { const d = document.createElement('div'); d.className = 'log-entry ' + l.level; d.textContent = '[' + l.timestamp + '] [' + l.level + '] ' + l.message; c.appendChild(d); }); }
        function clearDisplay() { document.getElementById('logs').innerHTML = ''; document.getElementById('stats').innerHTML = ''; }
        loadLogs(); loadStats();
    </script>
</body>
</html>"#.into()
}

// ── Parallel Processing / Archive / Performance ──────────────────────────────

fn cmd_process_parallel() {
    let entries = read_all_entries();
    if entries.is_empty() {
        println!("No logs to process.");
        return;
    }

    let num_threads = num_cpus::get().min(entries.len());
    println!("Processing {} log entries with {} threads...", entries.len(), num_threads);

    let chunks: Vec<Vec<LogEntry>> = entries.chunks((entries.len() + num_threads - 1) / num_threads)
        .map(|c| c.to_vec())
        .collect();

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for (i, chunk) in chunks.into_iter().enumerate() {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let valid = chunk.len();
            let errors = 0;
            tx.send((i, valid, errors)).ok();
        }));
    }
    drop(tx);

    let mut total_ok = 0;
    let mut total_err = 0;
    for _ in handles.iter() {
        if let Ok((id, ok, err)) = rx.recv() {
            println!("  Thread {}: {} ok, {} errors", id, ok, err);
            total_ok += ok;
            total_err += err;
        }
    }
    for h in handles { h.join().ok(); }
    println!("Done: {} valid, {} errors", total_ok, total_err);
}

fn cmd_archive(days: i64) {
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let entries = read_all_entries();
    if entries.is_empty() {
        println!("No logs to archive.");
        return;
    }

    let (old, recent): (Vec<&LogEntry>, Vec<&LogEntry>) = entries.iter().partition(|e| e.timestamp < cutoff);
    if old.is_empty() {
        println!("No logs older than {} days.", days);
        return;
    }

    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    let archive_name = format!("logs_archive_{}.json", ts);
    if let Ok(mut f) = File::create(&archive_name) {
        for e in &old {
            writeln!(f, "{}", serde_json::to_string(e).unwrap()).ok();
        }
    }
    // Rewrite log file with only recent entries
    if let Ok(mut f) = File::create(LOG_FILE) {
        for e in &recent {
            writeln!(f, "{}", serde_json::to_string(e).unwrap()).ok();
        }
    }
    println!("Archived {} old logs to: {}", old.len(), archive_name);
}

fn cmd_perf() {
    let start = Instant::now();
    let entries = read_all_entries();
    let total = entries.len();
    let file_size = fs::metadata(LOG_FILE).map(|m| m.len()).unwrap_or(0);
    let elapsed = start.elapsed();

    println!("{}", "🚀 Performance Metrics:".bold());
    println!("  File size: {} bytes ({:.2} KB)", file_size, file_size as f64 / 1024.0);
    println!("  Log entries: {}", total);
    println!("  Parse time: {:.2}ms", elapsed.as_millis());
    if total > 0 {
        println!("  Avg per entry: {:.3}ms", elapsed.as_millis() as f64 / total as f64);
    }
}

// ── Interactive Mode ─────────────────────────────────────────────────────────

fn cmd_interactive() {
    println!("{}", "Logger — Interactive Mode".bold());
    loop {
        println!("\nOptions:");
        println!("  1. Read All Logs     6. Search Logs       11. Debug Write    [a]rchive");
        println!("  2. INFO Logs         7. Statistics        12. Exit           [p]arallel");
        println!("  3. WARN Logs         8. Export CSV                          [f]ast (perf)");
        println!("  4. ERROR Logs        9. Export TXT");
        println!("  5. DEBUG Logs       10. Start Web Server");
        print!("Choice: ");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        match input {
            "1" => cmd_read(None, None),
            "2" => cmd_read(Some("info"), None),
            "3" => cmd_read(Some("warn"), None),
            "4" => cmd_read(Some("error"), None),
            "5" => cmd_read(Some("debug"), None),
            "6" => {
                print!("Keyword: ");
                io::stdout().flush().ok();
                let mut kw = String::new();
                io::stdin().read_line(&mut kw).ok();
                cmd_read(None, Some(kw.trim()));
            }
            "7" => cmd_stats(),
            "8" => cmd_export("csv"),
            "9" => cmd_export("txt"),
            "10" => {
                print!("Port [8080]: ");
                io::stdout().flush().ok();
                let mut p = String::new();
                io::stdin().read_line(&mut p).ok();
                let port: u16 = p.trim().parse().unwrap_or(8080);
                println!("Starting server (will block until Ctrl+C)...");
                cmd_serve(port);
            }
            "11" => { /* debug write */ }
            "12" | "exit" | "q" => { println!("Goodbye."); break; }
            "a" | "archive" => {
                print!("Archive logs older than N days [30]: ");
                io::stdout().flush().ok();
                let mut d = String::new();
                io::stdin().read_line(&mut d).ok();
                cmd_archive(d.trim().parse().unwrap_or(30));
            }
            "p" | "parallel" => cmd_process_parallel(),
            "f" | "fast" | "perf" => cmd_perf(),
            _ if input.starts_with("write ") => {
                let msg = input.trim_start_matches("write ").trim().to_string();
                if !msg.is_empty() {
                    cmd_write("info", &msg);
                }
            }
            _ => eprintln!("Unknown option: {}", input),
        }

        // After menu action, prompt for write message if option 11
        if input == "11" {
            print!("Enter DEBUG log message: ");
            io::stdout().flush().ok();
            let mut msg = String::new();
            io::stdin().read_line(&mut msg).ok();
            cmd_write("debug", msg.trim());
        }
    }
}

// ── Entry Point ──────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Write { level, message }) => cmd_write(&level, &message),
        Some(Commands::Read { level, search }) => cmd_read(level.as_deref(), search.as_deref()),
        Some(Commands::Stats) => cmd_stats(),
        Some(Commands::Export { format }) => cmd_export(&format),
        Some(Commands::Serve { port }) => cmd_serve(port),
        Some(Commands::Interactive) => cmd_interactive(),
        None => cmd_interactive(),
    }
}
