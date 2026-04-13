use std::collections::HashMap;
use chrono::{DateTime, Local, Duration, NaiveDate, Weekday};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Habit {
    id: Uuid,
    name: String,
    description: String,
    category: HabitCategory,
    frequency: HabitFrequency,
    created_at: DateTime<Local>,
    completions: Vec<DateTime<Local>>,
    current_streak: u32,
    longest_streak: u32,
    target_completions: u32,
    reminder_time: Option<String>,
    is_active: bool,
    priority: Priority,
}

enum Habits {
    Sports,
    Study,
    Work,
    Other,
}

enum HabitFrequency {
    Daily,
    Weekly,
    Monthly,
}

enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

fn create_habit(name: &str, description: &str) -> Habit {
    Habit {
        name: name.to_string(),
        description: description.to_string(),
    }
}

fn frequency(habit: &Habit) -> &Habits {
    match habit.description.as_str() {
        "daily" => &Habits::Daily,
        "weekly" => &Habits::Weekly,
        "monthly" => &Habits::Monthly,
        _ => &Habits::Daily,
    }
}

fn delete_habit(habit: &mut Habit) {
    habit.name.clear();
    habit.description.clear();
}

fn update_habit(habit: &mut Habit, name: &str, description: &str) {
    habit.name = name.to_string();
    habit.description = description.to_string();
}



fn main() {
    
}
