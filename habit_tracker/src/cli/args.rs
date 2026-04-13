use clap::{Parser, Subcommand, ValueEnum};
use std::str::FromStr;
use crate::models::{HabitCategory, Priority, HabitFrequency};
use chrono::Weekday;

#[derive(Parser)]
#[command(
    name = "habit-tracker",
    about = "A CLI habit tracker with progress visualization",
    version = "0.1.0",
    author = "Your Name",
    long_about = "Track your daily habits with progress bars, streaks, and statistics. \
                  Data is stored in ~/.local/share/habit-tracker/data/habits.json"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Data directory path (default: ~/.local/share/habit-tracker)
    #[arg(long, global = true, env = "HABIT_TRACKER_DATA_DIR")]
    pub data_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new habit with category and frequency
    #[command(alias = "a")]
    Add {
        /// Habit name (unique identifier)
        #[arg(value_parser = validate_name)]
        name: String,
        
        /// Description of the habit
        #[arg(short, long, value_parser = validate_description)]
        description: Option<String>,
        
        /// Category: sports, study, work, health, mindfulness, creativity, social, finance, other
        #[arg(short, long, default_value = "other", value_enum)]
        category: CategoryArg,
        
        /// Frequency: daily, weekly:mon,wed,fri, monthly, custom:3
        #[arg(short, long, default_value = "daily", value_parser = parse_frequency_arg)]
        frequency: FrequencyArg,
        
        /// Target completions before goal is reached
        #[arg(short, long, default_value_t = 30, value_parser = clap::value_parser!(u32).range(1..365))]
        target: u32,
        
        /// Priority: low, medium, high, critical
        #[arg(short, long, default_value = "medium", value_enum)]
        priority: PriorityArg,
    },
    
    /// List all habits with optional filters
    #[command(alias = "ls")]
    List {
        /// Filter by category
        #[arg(short, long, value_enum)]
        category: Option<CategoryArg>,
        
        /// Show archived habits
        #[arg(long)]
        archived: bool,
        
        /// Show only habits due today
        #[arg(long)]
        due: bool,
        
        /// Show only completed today
        #[arg(long)]
        done: bool,
        
        /// Sort by: name, streak, rate, priority
        #[arg(short, long, default_value = "name")]
        sort: SortBy,
    },
    
    /// Mark a habit as complete for today
    #[command(alias = "d")]
    Done {
        /// Habit name or ID prefix
        #[arg(value_parser = validate_name)]
        id: String,
    },
    
    /// Show detailed information about a habit
    #[command(alias = "s")]
    Show {
        /// Habit name or ID
        #[arg(value_parser = validate_name)]
        id: String,
    },
    
    /// Edit habit properties
    #[command(alias = "e")]
    Edit {
        /// Habit ID or name
        #[arg(value_parser = validate_name)]
        id: String,
        
        /// New name
        #[arg(short, long, value_parser = validate_name)]
        name: Option<String>,
        
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        
        /// New priority
        #[arg(short, long, value_enum)]
        priority: Option<PriorityArg>,
    },
    
    /// Archive a habit (soft delete)
    #[command(alias = "ar")]
    Archive {
        /// Habit ID or name
        #[arg(value_parser = validate_name)]
        id: String,
    },
    
    /// Restore an archived habit by index
    #[command(alias = "r")]
    Restore {
        /// Archive list index
        #[arg(value_parser = clap::value_parser!(usize))]
        index: usize,
    },
    
    /// Permanently delete a habit
    #[command(alias = "rm")]
    Delete {
        /// Habit ID or name
        #[arg(value_parser = validate_name)]
        id: String,
        
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show statistics and analytics
    #[command(alias = "st")]
    Stats {
        /// Show detailed breakdown by category
        #[arg(long)]
        by_category: bool,
    },
    
    /// Display daily dashboard with progress
    #[command(alias = "dash", visible_alias = "board")]
    Dashboard,
    
    /// Export habits to CSV or JSON
    #[command(alias = "ex")]
    Export {
        /// Output file path
        #[arg(default_value = "habits_export.csv")]
        path: String,
        
        /// Export format
        #[arg(short, long, default_value = "csv", value_enum)]
        format: ExportFormat,
    },
    
    /// Import habits from JSON
    #[command(alias = "im")]
    Import {
        /// Input JSON file
        path: String,
        
        /// Merge with existing (default: replace)
        #[arg(short, long)]
        merge: bool,
    },
    
    /// Interactive TUI mode
    #[command(alias = "i")]
    Interactive,
    
    /// Reset all data (DANGER)
    #[command(hide = true)]
    Reset {
        #[arg(long)]
        really: bool,
    },
}

// Value Enums for clap
#[derive(Clone, ValueEnum)]
pub enum CategoryArg {
    Sports,
    Study,
    Work,
    Health,
    Mindfulness,
    Creativity,
    Social,
    Finance,
    Other,
}

impl From<CategoryArg> for HabitCategory {
    fn from(arg: CategoryArg) -> Self {
        match arg {
            CategoryArg::Sports => HabitCategory::Sports,
            CategoryArg::Study => HabitCategory::Study,
            CategoryArg::Work => HabitCategory::Work,
            CategoryArg::Health => HabitCategory::Health,
            CategoryArg::Mindfulness => HabitCategory::Mindfulness,
            CategoryArg::Creativity => HabitCategory::Creativity,
            CategoryArg::Social => HabitCategory::Social,
            CategoryArg::Finance => HabitCategory::Finance,
            CategoryArg::Other => HabitCategory::Other,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum PriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

impl From<PriorityArg> for Priority {
    fn from(arg: PriorityArg) -> Self {
        match arg {
            PriorityArg::Low => Priority::Low,
            PriorityArg::Medium => Priority::Medium,
            PriorityArg::High => Priority::High,
            PriorityArg::Critical => Priority::Critical,
        }
    }
}

#[derive(Clone)]
pub struct FrequencyArg(pub HabitFrequency);

impl FromStr for FrequencyArg {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_frequency_arg(s).map_err(|e| e.to_string())
    }
}

#[derive(Clone, ValueEnum)]
pub enum SortBy {
    Name,
    Streak,
    Rate,
    Priority,
    Created,
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Csv,
    Json,
}

// Validators
fn validate_name(s: &str) -> Result<String, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if trimmed.len() > 100 {
        return Err("Name too long (max 100 chars)".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("Name cannot contain / or \\".to_string());
    }
    Ok(trimmed.to_string())
}

fn validate_description(s: &str) -> Result<String, String> {
    if s.len() > 500 {
        return Err("Description too long (max 500 chars)".to_string());
    }
    Ok(s.to_string())
}

pub fn parse_frequency_arg(s: &str) -> Result<FrequencyArg, String> {
    let freq = if s == "daily" {
        HabitFrequency::Daily
    } else if s.starts_with("weekly:") {
        let days_str = &s[7..];
        let days: Result<Vec<Weekday>, _> = days_str.split(',')
            .map(|d| parse_weekday(d.trim()))
            .collect();
        HabitFrequency::Weekly { days: days? }
    } else if s == "monthly" {
        HabitFrequency::Monthly { days: vec![1, 15] }
    } else if s.starts_with("monthly:") {
        let days: Result<Vec<u32>, _> = s[8..].split(',')
            .map(|d| d.trim().parse().map_err(|_| format!("Invalid day: {}", d)))
            .collect();
        let days = days?;
        if days.iter().any(|&d| d == 0 || d > 31) {
            return Err("Month days must be 1-31".to_string());
        }
        HabitFrequency::Monthly { days }
    } else if s.starts_with("custom:") {
        let days: u32 = s[7..].parse().map_err(|_| "Invalid interval")?;
        if days == 0 || days > 365 {
            return Err("Interval must be 1-365 days".to_string());
        }
        HabitFrequency::Custom { interval_days: days }
    } else {
        return Err(format!(
            "Invalid frequency '{}'. Use: daily, weekly:mon,wed,fri, monthly, custom:N",
            s
        ));
    };
    
    Ok(FrequencyArg(freq))
}

fn parse_weekday(s: &str) -> Result<Weekday, String> {
    match s.to_lowercase().as_str() {
        "mon" | "monday" => Ok(Weekday::Mon),
        "tue" | "tuesday" => Ok(Weekday::Tue),
        "wed" | "wednesday" => Ok(Weekday::Wed),
        "thu" | "thursday" => Ok(Weekday::Thu),
        "fri" | "friday" => Ok(Weekday::Fri),
        "sat" | "saturday" => Ok(Weekday::Sat),
        "sun" | "sunday" => Ok(Weekday::Sun),
        _ => Err(format!("Invalid day: {}. Use: mon, tue, wed, thu, fri, sat, sun", s)),
    }
}