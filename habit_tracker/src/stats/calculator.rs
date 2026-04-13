use crate::models::Habit;
use chrono::{Duration, Local};

#[derive(Debug, Default)]
pub struct TrackerStats {
    pub total_habits: u32,
    pub active_habits: u32,
    pub completed_today: u32,
    pub completed_this_week: u32,
    pub current_streaks: u32,
    pub average_completion_rate: f64,
}

pub struct StatsCalculator;

impl StatsCalculator {
    pub fn calculate(habits: &[&Habit]) -> TrackerStats {
        let today = Local::now().date_naive();
        let week_start = Local::now() - Duration::days(7);

        let mut stats = TrackerStats {
            total_habits: habits.len() as u32,
            ..Default::default()
        };

        let mut total_rate = 0.0;

        for h in habits {
            if h.is_active {
                stats.active_habits += 1;
            }

            if h.current_streak > 0 {
                stats.current_streaks += 1;
            }

            let today_count = h
                .completions
                .iter()
                .filter(|c| c.date_naive() == today)
                .count() as u32;
            stats.completed_today += today_count;

            let week_count = h
                .completions
                .iter()
                .filter(|c| **c >= week_start) // Fixed: dereference
                .count() as u32;
            stats.completed_this_week += week_count;

            total_rate += h.completion_rate();
        }

        if !habits.is_empty() {
            stats.average_completion_rate = total_rate / habits.len() as f64;
        }

        stats
    }
}
