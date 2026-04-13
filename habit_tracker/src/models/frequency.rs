use chrono::Weekday;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HabitFrequency {
    Daily,
    Weekly { days: Vec<Weekday> },
    Monthly { days: Vec<u32> },
    Custom { interval_days: u32 },
}

impl HabitFrequency {
    pub fn is_due_today(&self, weekday: Weekday, day_of_month: u32) -> bool {
        match self {
            HabitFrequency::Daily => true,
            HabitFrequency::Weekly { days } => days.contains(&weekday),
            HabitFrequency::Monthly { days } => days.contains(&day_of_month),
            HabitFrequency::Custom { .. } => true, // Simplified
        }
    }

    pub fn expected_per_week(&self) -> u32 {
        match self {
            HabitFrequency::Daily => 7,
            HabitFrequency::Weekly { days } => days.len() as u32,
            HabitFrequency::Monthly { days } => (days.len() as u32 + 3) / 4, // Approximate
            HabitFrequency::Custom { interval_days } => 7 / interval_days,
        }
    }
}
