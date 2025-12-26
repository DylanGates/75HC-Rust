use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{self, File};
use std::io;
use std::io::{Read, Write};
use chrono::{DateTime, Utc};
use colored::*;

const LOG_FILE_PATH: &str = "log.json";
const MAX_LOG_SIZE: u64 = 1024 * 1024; // 1MB

fn rotate_log_if_needed() -> io::Result<()> {
    if let Ok(metadata) = fs::metadata(LOG_FILE_PATH) {
        if metadata.len() > MAX_LOG_SIZE {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let backup_path = format!("log_backup_{}.json", timestamp);
            fs::rename(LOG_FILE_PATH, backup_path)?;
            println!("Log file rotated to: {}", backup_path);
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
}

fn log_message(level: LogLevel, message: &str) {
    if let Err(e) = rotate_log_if_needed() {
        eprintln!("Failed to rotate log: {}", e);
    }

    let log_entry = LogEntry {
        timestamp: Utc::now(),
        level,
        message: message.to_string(),
    };

    let log_json = serde_json::to_string(&log_entry).expect("Failed to serialize log entry");

    let mut file = File::options()
        .append(true)
        .create(true)
        .open(LOG_FILE_PATH)
        .expect("Failed to open log file");

    writeln!(file, "{}", log_json).expect("Failed to write log entry");
}

fn search_logs(keyword: &str) {
    let mut file = match File::open(LOG_FILE_PATH) {
        Ok(file) => file,
        Err(_) => {
            println!("No log file found. No logs to search.");
            return;
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read log file");

    if contents.trim().is_empty() {
        println!("Log file is empty.");
        return;
    }

    let mut found = false;
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let log_entry: LogEntry =
            serde_json::from_str(line).expect("Failed to deserialize log entry");
        
        if log_entry.message.to_lowercase().contains(&keyword.to_lowercase()) {
            let level_str = match log_entry.level {
                LogLevel::INFO => "INFO".green(),
                LogLevel::WARN => "WARN".yellow(),
                LogLevel::ERROR => "ERROR".red(),
                LogLevel::DEBUG => "DEBUG".blue(),
            };
            
            println!("[{}] [{}] {}", 
                log_entry.timestamp.format("%Y-%m-%d %H:%M:%S").dimmed(),
                level_str,
                log_entry.message
            );
            found = true;
        }
    }
    
    if !found {
        println!("No logs found containing: {}", keyword);
    }
}
    let mut file = match File::open(LOG_FILE_PATH) {
        Ok(file) => file,
        Err(_) => {
            println!("No log file found. No logs to display.");
            return;
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read log file");

    if contents.trim().is_empty() {
        println!("Log file is empty.");
        return;
    }

    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let log_entry: LogEntry =
            serde_json::from_str(line).expect("Failed to deserialize log entry");
        
        // Filter by level if specified
        if let Some(filter_level) = level_filter {
            if std::mem::discriminant(&log_entry.level) != std::mem::discriminant(&filter_level) {
                continue;
            }
        }
        
        let level_str = match log_entry.level {
            LogLevel::INFO => "INFO".green(),
            LogLevel::WARN => "WARN".yellow(),
            LogLevel::ERROR => "ERROR".red(),
            LogLevel::DEBUG => "DEBUG".blue(),
        };
        
        println!("[{}] [{}] {}", 
            log_entry.timestamp.format("%Y-%m-%d %H:%M:%S").dimmed(),
            level_str,
            log_entry.message
        );
    }
}

fn main() {
    println!("Please select an option:");

    println!(
        "
    1. Read All Logs
    2. Read INFO Logs
    3. Read WARN Logs
    4. Read ERROR Logs
    5. Read DEBUG Logs
    6. Search Logs
    7. Write INFO Log
    8. Write WARN Log
    9. Write ERROR Log
    10. Write DEBUG Log
    11. Exit"
    );

    loop {
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "1" => {
                read_logs_filtered(None);
            }
            "2" => {
                read_logs_filtered(Some(LogLevel::INFO));
            }
            "3" => {
                read_logs_filtered(Some(LogLevel::WARN));
            }
            "4" => {
                read_logs_filtered(Some(LogLevel::ERROR));
            }
            "5" => {
                read_logs_filtered(Some(LogLevel::DEBUG));
            }
            "6" => {
                println!("Enter search keyword:");
                let mut keyword = String::new();
                io::stdin()
                    .read_line(&mut keyword)
                    .expect("Failed to read line");
                search_logs(keyword.trim());
            }
            "7" => {
                println!("Enter INFO log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::INFO, message.trim());
                println!("INFO log written.");
            }
            "8" => {
                println!("Enter WARN log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::WARN, message.trim());
                println!("WARN log written.");
            }
            "9" => {
                println!("Enter ERROR log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::ERROR, message.trim());
                println!("ERROR log written.");
            }
            "10" => {
                println!("Enter DEBUG log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::DEBUG, message.trim());
                println!("DEBUG log written.");
            }
            "11" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid option. Please try again.");
            }
        }

        println!("\nPlease select an option:");
        println!("1. Read All Logs\n2. Read INFO Logs\n3. Read WARN Logs\n4. Read ERROR Logs\n5. Read DEBUG Logs\n6. Search Logs\n7. Write INFO Log\n8. Write WARN Log\n9. Write ERROR Log\n10. Write DEBUG Log\n11. Exit");
    }
}
