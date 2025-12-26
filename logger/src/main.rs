use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use chrono::{DateTime, Utc};

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
    let log_entry = LogEntry {
        timestamp: Utc::now(),
        level,
        message: message.to_string(),
    };

    let log_json = serde_json::to_string(&log_entry).expect("Failed to serialize log entry");

    let mut file = File::options()
        .append(true)
        .create(true)
        .open("log.json")
        .expect("Failed to open log file");

    writeln!(file, "{}", log_json).expect("Failed to write log entry");
}

fn read_logs() {
    let mut file = match File::open("log.json") {
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
        println!("[{}] [{:?}] {}", log_entry.timestamp.format("%Y-%m-%d %H:%M:%S"), log_entry.level, log_entry.message);
    }
}

fn main() {
    println!("Please select an option:");

    println!(
        "
    1. Read Logs
    2. Write INFO Log
    3. Write WARN Log
    4. Write ERROR Log
    5. Write DEBUG Log
    6. Exit"
    );

    loop {
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "1" => {
                read_logs();
            }
            "2" => {
                println!("Enter INFO log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::INFO, message.trim());
                println!("INFO log written.");
            }
            "3" => {
                println!("Enter WARN log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::WARN, message.trim());
                println!("WARN log written.");
            }
            "4" => {
                println!("Enter ERROR log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::ERROR, message.trim());
                println!("ERROR log written.");
            }
            "5" => {
                println!("Enter DEBUG log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(LogLevel::DEBUG, message.trim());
                println!("DEBUG log written.");
            }
            "6" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid option. Please try again.");
            }
        }

        println!("\nPlease select an option:");
        println!("1. Read Logs\n2. Write INFO Log\n3. Write WARN Log\n4. Write ERROR Log\n5. Write DEBUG Log\n6. Exit");
    }
}
