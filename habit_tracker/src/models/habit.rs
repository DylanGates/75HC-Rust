use super::{HabitCategory, HabitFrequency};
use chrono::{DateTime, Datelike, Duration, Local}; // Added Datelike
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)] // Added PartialOrd, Ord
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: HabitCategory,
    pub frequency: HabitFrequency,
    pub created_at: DateTime<Local>,
    pub completions: Vec<DateTime<Local>>,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub target_completions: u32,
    pub reminder_time: Option<String>,
    pub is_active: bool,
    pub priority: Priority,
}

impl Habit {
    pub fn new(
        name: &str,
        description: &str,
        category: HabitCategory,
        frequency: HabitFrequency,
        target: u32,
        priority: Priority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            category,
            frequency,
            created_at: Local::now(),
            completions: Vec::new(),
            current_streak: 0,
            longest_streak: 0,
            target_completions: target,
            reminder_time: None,
            is_active: true,
            priority,
        }
    }

    pub fn complete(&mut self) -> Result<(), HabitError> {
        if !self.is_active {
            return Err(HabitError::InactiveHabit);
        }

        if self.is_completed_today() {
            return Err(HabitError::AlreadyCompleted);
        }

        self.completions.push(Local::now());
        self.update_streak();
        Ok(())
    }

    pub fn is_completed_today(&self) -> bool {
        let today = Local::now().date_naive();
        self.completions.iter().any(|c| c.date_naive() == today)
    }

    fn update_streak(&mut self) {
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);

        let completed_yesterday = self.completions.iter().any(|c| c.date_naive() == yesterday);

        if completed_yesterday || self.current_streak == 0 {
            self.current_streak += 1;
            self.longest_streak = self.longest_streak.max(self.current_streak);
        } else {
            self.current_streak = 1;
        }
    }

    pub fn completion_rate(&self) -> f64 {
        let days = (Local::now() - self.created_at).num_days().max(1) as f64;
        let expected = match &self.frequency {
            HabitFrequency::Daily => days,
            HabitFrequency::Weekly { .. } => days / 7.0,
            HabitFrequency::Monthly { .. } => days / 30.0,
            HabitFrequency::Custom { interval_days } => days / *interval_days as f64,
        };

        (self.completions.len() as f64 / expected) * 100.0
    }

    pub fn weekly_progress(&self) -> (u32, u32) {
        let week_start = Local::now() - Duration::days(7);
        let count = self
            .completions
            .iter()
            .filter(|c| **c >= week_start) // Fixed: dereference
            .count() as u32;

        (count, self.frequency.expected_per_week())
    }

    pub fn is_due_today(&self) -> bool {
        let now = Local::now();
        self.is_active
            && !self.is_completed_today()
            && self.frequency.is_due_today(now.weekday(), now.day())
    }
}

#[derive(Debug)]
pub enum HabitError {
    InactiveHabit,
    AlreadyCompleted,
    NotFound,
}

impl std::fmt::Display for HabitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HabitError::InactiveHabit => write!(f, "Habit is inactive"),
            HabitError::AlreadyCompleted => write!(f, "Already completed today"),
            HabitError::NotFound => write!(f, "Habit not found"),
        }
    }
}

impl std::error::Error for HabitError {}
