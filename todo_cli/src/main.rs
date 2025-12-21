use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

enum Command {
    Add,
    List,
    Scheduled,
    Complete,
}

enum TaskPriority {
    Low,
    Medium,
    High,
}

fn todo_command(command: Command, task: Option<String>, priority: Option<TaskPriority>) {
    match command {
        Command::Add => {
            if let Some(task_desc) = task {
                let task_priority = match priority {
                    Some(TaskPriority::Low) => "Low",
                    Some(TaskPriority::Medium) => "Medium",
                    Some(TaskPriority::High) => "High",
                    None => "Medium",
                };
                println!(
                    "Added task: '{}' with priority: {}",
                    task_desc, task_priority
                );
            } else {
                println!("No task description provided.");
            }
        }
        Command::List => {
            println!("Listing all tasks...");
            let todos = File::open("todo.txt").expect("Could not open todo.txt");
            let reader = BufReader::new(todos);
            for (index, line) in reader.lines().enumerate() {
                let line = line.expect("Could not read line");
                println!("{}: {}", index + 1, line);
            }
        }
        Command::Scheduled => {
            println!("Listing scheduled tasks...");
            let todos = File::open("todo.txt").expect("Could not open todo.txt");
            let reader = BufReader::new(todos);
            for (index, line) in reader.lines().enumerate() {
                let line = line.expect("Could not read line");
                if line.contains("[Scheduled]") {
                    println!("{}: {}", index + 1, line);
                }
            }
        }
        Command::Complete => {
            if let Some(task_desc) = task {
                println!("Marked task as complete: '{}'", task_desc);
            } else {
                println!("No task description provided to complete.");
            }
        }
    }
}

fn main() {
    println!("Welcome to the Todo CLI!");

    loop {
        println!("Please enter a command (add, list, scheduled, complete) or 'exit' to quit:");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed_input = input.trim();

        if trimmed_input.eq_ignore_ascii_case("exit") {
            break;
        }

        let parts: Vec<&str> = trimmed_input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let command_str = parts[0];
        let command = match command_str.to_lowercase().as_str() {
            "add" => Command::Add,
            "list" => Command::List,
            "scheduled" => Command::Scheduled,
            "complete" => Command::Complete,
            _ => {
                println!("Unknown command: {}", command_str);
                continue;
            }
        };

        let task = if parts.len() > 1 {
            Some(parts[1..].join(" "))
        } else {
            None
        };

        let priority = if let Some(task_desc) = &task {
            if task_desc.contains("[High]") {
                Some(TaskPriority::High)
            } else if task_desc.contains("[Low]") {
                Some(TaskPriority::Low)
            } else {
                Some(TaskPriority::Medium)
            }
        } else {
            None
        };

        todo_command(command, task, priority);
    }
}
