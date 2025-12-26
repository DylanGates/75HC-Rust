use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

enum Logger {
    Write,
}

#[derive(Serialize, Deserialize)]
struct LogEntry {
    mode: String,
    message: String,
}

fn log_message(mode: Logger, message: &str) {
    let log_entry = LogEntry {
        mode: match mode {
            Logger::Write => "WRITE".to_string(),
        },
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
        println!("[{}] {}", log_entry.mode, log_entry.message);
    }
}

fn main() {
    println!("Please select an option:");

    println!(
        "
    1. Read Logs
    2. Write Log
    3. Exit"
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
                println!("Enter log message:");
                let mut message = String::new();
                io::stdin()
                    .read_line(&mut message)
                    .expect("Failed to read line");
                log_message(Logger::Write, message.trim());
                println!("Log written.");
            }
            "3" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid option. Please try again.");
            }
        }

        println!("\nPlease select an option:");
        println!("1. Read Logs\n2. Write Log\n3. Exit");
    }
}
