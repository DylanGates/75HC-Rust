use clap::{Parser, Subcommand};
use crate::models::{HabitCategory, Priority};

#[derive(Parser)]
#[command(name = "habit-tracker")]
#[command(about = "A CLI habit tracker")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new habit
    Add {
        /// Habit name
        name: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Category (sports, study, work, health, mindfulness, creativity, social, finance, other)
        #[arg(short, long, default_value = "other")]
        category: String,
        /// Frequency: daily, weekly, monthly, or custom:N
        #[arg(short, long, default_value = "daily")]
        frequency: String,
        /// Target completions
        #[arg(short, long, default_value_t = 30)]
        target: u32,
        /// Priority: low, medium, high, critical
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// List all habits
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
        /// Show archived habits
        #[arg(long)]
        archived: bool,
        /// Show only due today
        #[arg(long)]
        due: bool,
    },
    /// Complete a habit
    Done {
        /// Habit ID or name
        id: String,
    },
    /// Show habit details
    Show {
        /// Habit ID or name
        id: String,
    },
    /// Edit a habit
    Edit {
        /// Habit ID
        id: String,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Archive a habit
    Archive {
        /// Habit ID
        id: String,
    },
    /// Restore an archived habit
    Restore {
        /// Index in archive list
        index: usize,
    },
    /// Delete a habit permanently
    Delete {
        /// Habit ID
        id: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Show statistics
    Stats,
    /// Show daily dashboard
    Dashboard,
    /// Export habits to CSV
    Export {
        /// Output file path
        #[arg(default_value = "habits_export.csv")]
        path: String,
    },
    /// Interactive mode
    Interactive,
}

impl Commands {
    pub fn parse_category(s: &str) -> Result<HabitCategory, String> {
        match s.to_lowercase().as_str() {
            "sports" => Ok(HabitCategory::Sports),
            "study" => Ok(HabitCategory::Study),
            "work" => Ok(HabitCategory::Work),
            "health" => Ok(HabitCategory::Health),
            "mindfulness" => Ok(HabitCategory::Mindfulness),
            "creativity" => Ok(HabitCategory::Creativity),
            "social" => Ok(HabitCategory::Social),
            "finance" => Ok(HabitCategory::Finance),
            "other" => Ok(HabitCategory::Other),
            _ => Err(format!("Unknown category: {}", s)),
        }
    }

    pub fn parse_priority(s: &str) -> Result<Priority, String> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(format!("Unknown priority: {}", s)),
        }
    }
}

use crate::models::HabitFrequency;
use chrono::Weekday;

pub fn parse_frequency(s: &str) -> Result<HabitFrequency, String> {
    if s == "daily" {
        Ok(HabitFrequency::Daily)
    } else if s.starts_with("weekly:") {
        let days: Result<Vec<Weekday>, _> = s[7..].split(',')
            .map(|d| match d.trim().to_lowercase().as_str() {
                "mon" | "monday" => Ok(Weekday::Mon),
                "tue" | "tuesday" => Ok(Weekday::Tue),
                "wed" | "wednesday" => Ok(Weekday::Wed),
                "thu" | "thursday" => Ok(Weekday::Thu),
                "fri" | "friday" => Ok(Weekday::Fri),
                "sat" | "saturday" => Ok(Weekday::Sat),
                "sun" | "sunday" => Ok(Weekday::Sun),
                _ => Err(format!("Invalid day: {}", d)),
            })
            .collect();
        Ok(HabitFrequency::Weekly { days: days? })
    } else if s == "monthly" {
        Ok(HabitFrequency::Monthly { days: vec![1, 15] })
    } else if s.starts_with("custom:") {
        let days: u32 = s[7..].parse().map_err(|_| "Invalid interval")?;
        Ok(HabitFrequency::Custom { interval_days: days })
    } else {
        Err(format!("Invalid frequency: {}", s))
    }
}