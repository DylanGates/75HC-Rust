use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// ─── Data structures ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: Uuid,
    description: String,
    priority: Priority,
    completed: bool,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
            Priority::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => anyhow::bail!("Invalid priority: {s}. Use: low, medium, high, critical"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoData {
    tasks: HashMap<Uuid, Task>,
    next_id: u64,
}

impl TodoData {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }
}

// ─── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "TodoApp", about = "A persistent todo list CLI app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the data file
    #[arg(short, long, default_value = "data.json", global = true)]
    file: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task description
        description: String,
        /// Priority: low, medium, high, critical
        #[arg(short, long, default_value = "medium")]
        priority: Priority,
    },
    /// List all tasks
    List {
        /// Show completed tasks too
        #[arg(short, long)]
        all: bool,
        /// Filter by priority
        #[arg(short, long)]
        priority: Option<Priority>,
    },
    /// Mark a task as complete
    Complete {
        /// Task ID
        id: Uuid,
    },
    /// Delete a task
    Delete {
        /// Task ID
        id: Uuid,
    },
    /// Clear all tasks (or all completed tasks)
    Clear {
        /// Only clear completed tasks
        #[arg(short, long)]
        completed: bool,
    },
    /// Search tasks by description
    Search {
        /// Search query
        query: String,
    },
    /// Show task statistics
    Stats,
}

// ─── Data file helpers ─────────────────────────────────────────────────────────

fn load_data(path: &PathBuf) -> Result<TodoData> {
    if !path.exists() {
        return Ok(TodoData::new());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read data file: {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(TodoData::new());
    }
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse data file: {}", path.display()))
}

fn save_data(path: &PathBuf, data: &TodoData) -> Result<()> {
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write data file: {}", path.display()))?;
    Ok(())
}

// ─── Display helpers ───────────────────────────────────────────────────────────

fn priority_color(p: &Priority) -> &'static str {
    match p {
        Priority::Low => "\x1b[32m",     // green
        Priority::Medium => "\x1b[33m",  // yellow
        Priority::High => "\x1b[38;5;208m", // orange
        Priority::Critical => "\x1b[31m", // red
    }
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const STRIKE: &str = "\x1b[9m";

fn format_task(task: &Task, show_id: bool) -> String {
    let status = if task.completed { "✓" } else { " " };
    let color = priority_color(&task.priority);
    let desc = if task.completed {
        format!("{STRIKE}{}{RESET}", task.description)
    } else {
        task.description.clone()
    };
    let created = task.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M");
    let id_part = if show_id {
        format!(" {DIM}[{}]{RESET}", task.id)
    } else {
        String::new()
    };

    format!(
        "  [{status}] {color}{:<8}{RESET} {desc}{id_part} {DIM}({created}){RESET}",
        task.priority.to_string()
    )
}

// ─── Commands ──────────────────────────────────────────────────────────────────

fn cmd_add(data: &mut TodoData, description: String, priority: Priority) -> Result<()> {
    let task = Task {
        id: Uuid::new_v4(),
        description,
        priority,
        completed: false,
        created_at: Utc::now(),
        completed_at: None,
    };
    println!("Added task: {}", task.description);
    println!("{}", format_task(&task, true));
    data.tasks.insert(task.id, task);
    Ok(())
}

fn cmd_list(data: &TodoData, show_all: bool, filter_priority: Option<Priority>) {
    let mut tasks: Vec<&Task> = data.tasks.values().collect();
    tasks.sort_by_key(|t| t.created_at);

    let total = tasks.len();
    let filtered: Vec<&&Task> = tasks
        .iter()
        .filter(|t| {
            let completed_ok = show_all || !t.completed;
            let priority_ok = filter_priority.as_ref().map_or(true, |p| t.priority == *p);
            completed_ok && priority_ok
        })
        .collect();

    if filtered.is_empty() {
        println!("No tasks found.");
        return;
    }

    println!("{}", BOLD);
    println!("Tasks ({}/{total} shown):", filtered.len());
    println!("{}", RESET);
    for t in &filtered {
        println!("{}", format_task(t, false));
    }
}

fn cmd_complete(data: &mut TodoData, id: Uuid) -> Result<()> {
    let task = data
        .tasks
        .get_mut(&id)
        .with_context(|| format!("Task not found: {id}"))?;

    if task.completed {
        println!("Task is already completed: {}", task.description);
        println!("{}", format_task(task, true));
        return Ok(());
    }

    task.completed = true;
    task.completed_at = Some(Utc::now());
    println!("Completed task: {}", task.description);
    println!("{}", format_task(task, true));
    Ok(())
}

fn cmd_delete(data: &mut TodoData, id: Uuid) -> Result<()> {
    let task = data
        .tasks
        .remove(&id)
        .with_context(|| format!("Task not found: {id}"))?;
    println!("Deleted task: {} [{}]", task.description, task.id);
    Ok(())
}

fn cmd_clear(data: &mut TodoData, only_completed: bool) {
    if only_completed {
        let count = data.tasks.len();
        data.tasks.retain(|_, t| !t.completed);
        let removed = count - data.tasks.len();
        println!("Cleared {removed} completed task(s).");
    } else {
        let count = data.tasks.len();
        data.tasks.clear();
        println!("Cleared all {count} task(s).");
    }
}

fn cmd_search(data: &TodoData, query: &str) {
    let query_lower = query.to_lowercase();
    let mut results: Vec<&Task> = data
        .tasks
        .values()
        .filter(|t| t.description.to_lowercase().contains(&query_lower))
        .collect();

    results.sort_by_key(|t| t.created_at);

    if results.is_empty() {
        println!("No tasks match query: {query}");
        return;
    }

    println!("Found {} task(s) matching '{query}':", results.len());
    for t in &results {
        println!("{}", format_task(t, true));
    }
}

fn cmd_stats(data: &TodoData) {
    let total = data.tasks.len();
    let completed = data.tasks.values().filter(|t| t.completed).count();
    let pending = total - completed;

    println!("═══ Task Statistics ═══");
    println!("  Total:     {total}");
    println!("  Completed: {completed}");
    println!("  Pending:   {pending}");

    if total > 0 {
        let pct = (completed as f64 / total as f64) * 100.0;
        // Simple progress bar
        let bar_width = 20;
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let empty = bar_width - filled;
        let bar: String = std::iter::repeat('█')
            .take(filled)
            .chain(std::iter::repeat('░').take(empty))
            .collect();
        println!("  Progress:  [{bar}] {pct:.1}%");
    }

    // Priority breakdown
    println!();
    for p in [Priority::Critical, Priority::High, Priority::Medium, Priority::Low] {
        let count = data.tasks.values().filter(|t| t.priority == p).count();
        let done = data.tasks.values().filter(|t| t.priority == p && t.completed).count();
        if count > 0 {
            println!("  {}: {count} ({done} done)", p);
        }
    }
}

// ─── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut data = load_data(&cli.file)?;

    match cli.command {
        Commands::Add { description, priority } => {
            cmd_add(&mut data, description, priority)?;
        }
        Commands::List { all, priority } => {
            cmd_list(&data, all, priority);
            return Ok(()); // no save needed
        }
        Commands::Complete { id } => {
            cmd_complete(&mut data, id)?;
        }
        Commands::Delete { id } => {
            cmd_delete(&mut data, id)?;
        }
        Commands::Clear { completed } => {
            cmd_clear(&mut data, completed);
        }
        Commands::Search { query } => {
            cmd_search(&data, &query);
            return Ok(());
        }
        Commands::Stats => {
            cmd_stats(&data);
            return Ok(());
        }
    }

    save_data(&cli.file, &data)?;
    Ok(())
}
