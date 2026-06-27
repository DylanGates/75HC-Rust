use std::fs;
use std::fs::File;
use std::io::BufReader;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: u32,
    description: String,
    priority: String,
    completed: bool,
    created_at: DateTime<Utc>,
    due_date: Option<DateTime<Utc>>,
}

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A simple todo CLI application")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task description
        task: String,
        /// Task priority (low, medium, high)
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// List all tasks
    List,
    /// List scheduled tasks
    Scheduled,
    /// Mark a task as complete
    Complete {
        /// Task description to complete
        task: String,
    },
    /// Set due date for a task
    Due {
        /// Task description
        task: String,
        /// Due date (YYYY-MM-DD HH:MM)
        date: String,
    },
    /// Search tasks by keyword
    Search {
        /// Search keyword
        keyword: String,
    },
    /// Delete a task
    Delete {
        /// Task description to delete
        task: String,
    },
    /// Show task statistics
    Stats,
}

fn add_task(task_desc: &str, task_priority: &str) {
    let mut tasks = load_tasks();
    let new_id = tasks.len() as u32 + 1;
    
    let task = Task {
        id: new_id,
        description: task_desc.to_string(),
        priority: task_priority.to_string(),
        completed: false,
        created_at: Utc::now(),
        due_date: None,
    };
    
    tasks.push(task);
    save_tasks(&tasks);
    
    println!(
        "Added task: '{}' with priority: {}",
        task_desc, task_priority
    );
}

fn list_tasks() {
    let tasks = load_tasks();
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }
    
    println!("Listing all tasks:");
    for task in &tasks {
        let status = if task.completed { "✓" } else { "○" };
        println!("{} [{}] {} - {}", status, task.priority, task.description, task.created_at.format("%Y-%m-%d %H:%M"));
    }
}

fn list_scheduled_tasks() {
    let tasks = load_tasks();
    let scheduled_tasks: Vec<&Task> = tasks.iter()
        .filter(|task| task.due_date.is_some())
        .collect();
    
    if scheduled_tasks.is_empty() {
        println!("No scheduled tasks found.");
        return;
    }
    
    println!("Listing scheduled tasks:");
    for task in scheduled_tasks {
        let status = if task.completed { "✓" } else { "○" };
        let due_date = task.due_date.as_ref().unwrap().format("%Y-%m-%d %H:%M");
        println!("{} [{}] {} - Due: {}", status, task.priority, task.description, due_date);
    }
}

fn complete_task(task_desc: &str) {
    let mut tasks = load_tasks();
    
    if let Some(index) = tasks.iter().position(|t| t.description.contains(task_desc)) {
        tasks[index].completed = true;
        save_tasks(&tasks);
        println!("Marked task as complete: '{}'", tasks[index].description);
    } else {
        println!("Task not found: '{}'", task_desc);
    }
}

fn set_due_date(task_desc: &str, date_str: &str) {
    let mut tasks = load_tasks();
    
    // Parse the date string
    let due_date = match DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date_str)) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => match chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S") {
            Ok(ndt) => DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc),
            Err(_) => {
                println!("Invalid date format. Use YYYY-MM-DD or YYYY-MM-DD HH:MM");
                return;
            }
        }
    };
    
    if let Some(index) = tasks.iter().position(|t| t.description.contains(task_desc)) {
        tasks[index].due_date = Some(due_date);
        save_tasks(&tasks);
        println!("Set due date for task '{}': {}", tasks[index].description, due_date.format("%Y-%m-%d %H:%M"));
    } else {
        println!("Task not found: '{}'", task_desc);
    }
}

fn search_tasks(keyword: &str) {
    let tasks = load_tasks();
    let matching_tasks: Vec<&Task> = tasks.iter()
        .filter(|task| task.description.to_lowercase().contains(&keyword.to_lowercase()))
        .collect();
    
    if matching_tasks.is_empty() {
        println!("No tasks found containing '{}'", keyword);
        return;
    }
    
    println!("Tasks containing '{}':", keyword);
    for task in matching_tasks {
        let status = if task.completed { "✓" } else { "○" };
        let due_info = if let Some(due_date) = task.due_date {
            format!(" - Due: {}", due_date.format("%Y-%m-%d %H:%M"))
        } else {
            format!("")
        };
        println!("{} [{}] {}{}", status, task.priority, task.description, due_info);
    }
}

fn delete_task(task_desc: &str) {
    let mut tasks = load_tasks();
    let initial_len = tasks.len();
    
    tasks.retain(|task| !task.description.contains(task_desc));
    
    if tasks.len() < initial_len {
        save_tasks(&tasks);
        println!("Deleted task: '{}'", task_desc);
    } else {
        println!("Task not found: '{}'", task_desc);
    }
}

fn show_stats() {
    let tasks = load_tasks();
    
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }
    
    let total_tasks = tasks.len();
    let completed_tasks = tasks.iter().filter(|t| t.completed).count();
    let pending_tasks = total_tasks - completed_tasks;
    let scheduled_tasks = tasks.iter().filter(|t| t.due_date.is_some()).count();
    
    let priority_counts = tasks.iter().fold(std::collections::HashMap::new(), |mut acc, task| {
        *acc.entry(&task.priority).or_insert(0) += 1;
        acc
    });
    
    println!("📊 Task Statistics");
    println!("==================");
    println!("Total tasks:     {}", total_tasks);
    println!("Completed:       {} ({:.1}%)", completed_tasks, 
             if total_tasks > 0 { (completed_tasks as f64 / total_tasks as f64) * 100.0 } else { 0.0 });
    println!("Pending:         {}", pending_tasks);
    println!("Scheduled:       {}", scheduled_tasks);
    println!("");
    println!("Priority breakdown:");
    for (priority, count) in priority_counts.iter() {
        println!("  {}: {}", priority, count);
    }
}

fn load_tasks() -> Vec<Task> {
    let filename = "todo.json";
    if !fs::metadata(filename).is_ok() {
        return Vec::new();
    }
    
    let file = File::open(filename).expect("Could not open todo.json");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
}

fn save_tasks(tasks: &[Task]) {
    let json = serde_json::to_string_pretty(tasks).expect("Could not serialize tasks");
    fs::write("todo.json", json).expect("Could not write to todo.json");
}

fn main() {
    let cli = Cli::parse();

    let filename = "todo.txt";
    if !fs::metadata(filename).is_ok() {
        fs::File::create(filename).expect("Could not create todo.txt");
    }

    match &cli.command {
        Commands::Add { task, priority } => {
            let task_priority = match priority.as_str() {
                "low" => "Low",
                "medium" => "Medium",
                "high" => "High",
                _ => "Medium",
            };
            add_task(task, task_priority);
        }
        Commands::List => list_tasks(),
        Commands::Scheduled => list_scheduled_tasks(),
        Commands::Complete { task } => complete_task(task),
        Commands::Due { task, date } => set_due_date(task, date),
        Commands::Search { keyword } => search_tasks(keyword),
        Commands::Delete { task } => delete_task(task),
        Commands::Stats => show_stats(),
    }
}
