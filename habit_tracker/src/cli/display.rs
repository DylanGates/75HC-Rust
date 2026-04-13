use colored::*;
use comfy_table::{Table, ContentArrangement};
use crate::models::{Habit, Priority};
use crate::stats::TrackerStats;

pub struct TableFormatter;

impl TableFormatter {
    pub fn habits(habits: &[&Habit]) -> String {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        
        table.set_header(vec![
            "ID".cell(),
            "Name".cell(),
            "Category".cell(),
            "Freq".cell(),
            "Streak".cell(),
            "Rate".cell(),
            "Status".cell(),
        ]);

        for h in habits {
            let status = if h.is_completed_today() {
                "✓ Done".green()
            } else if h.is_due_today() {
                "○ Due".yellow()
            } else {
                "-".dimmed()
            };

            let prio_icon = match h.priority {
                Priority::Critical => "Critical ",
                Priority::High => "High ",
                Priority::Medium => "Medium ",
                Priority::Low => "Low ",
            };

            table.add_row(vec![
                h.id.to_string().dimmed().to_string(),
                format!("{}{}", prio_icon, h.name),
                h.category.to_string(),
                format!("{:?}", h.frequency).dimmed().to_string(),
                format!("{}", h.current_streak).yellow().to_string(),
                format!("{:.1}%", h.completion_rate()),
                status.to_string(),
            ]);
        }

        table.to_string()
    }

    pub fn habit_detail(habit: &Habit) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", "═".repeat(50).cyan()));
        output.push_str(&format!("  {}\n", habit.name.bold().white()));
        output.push_str(&format!("{}\n", "═".repeat(50).cyan()));
        
        output.push_str(&format!("  ID:          {}\n", habit.id));
        output.push_str(&format!("  Description: {}\n", habit.description));
        output.push_str(&format!("  Category:    {}\n", habit.category.to_string().cyan()));
        output.push_str(&format!("  Priority:    {:?}\n", habit.priority));
        output.push_str(&format!("  Frequency:   {:?}\n", habit.frequency));
        output.push_str(&format!("  Created:     {}\n", habit.created_at.format("%Y-%m-%d")));
        
        let (done, target) = habit.weekly_progress();
        output.push_str(&format!("  Weekly:      {}/{} completions\n", done, target));
        output.push_str(&format!("  Streak:      {} (best: {})\n", 
            habit.current_streak.to_string().yellow().bold(),
            habit.longest_streak));
        output.push_str(&format!("  Completion:  {:.1}%\n", habit.completion_rate()));
        output.push_str(&format!("  Active:      {}\n", 
            if habit.is_active { "Yes".green() } else { "No".red() }));
        
        if !habit.completions.is_empty() {
            output.push_str(&format!("\n  Last 5 completions:\n"));
            for (i, c) in habit.completions.iter().rev().take(5).enumerate() {
                output.push_str(&format!("    {}. {}\n", i+1, c.format("%Y-%m-%d %H:%M")));
            }
        }

        output.push_str(&format!("{}\n", "═".repeat(50).cyan()));
        output
    }

    pub fn stats(stats: &TrackerStats) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", "📊 Statistics".bold()));
        output.push_str(&format!("  Total Habits:      {}\n", stats.total_habits));
        output.push_str(&format!("  Active:            {}\n", stats.active_habits.to_string().green()));
        output.push_str(&format!("  Completed Today:   {}\n", stats.completed_today.to_string().cyan()));
        output.push_str(&format!("  This Week:         {}\n", stats.completed_this_week));
        output.push_str(&format!("  On Streak:         {}\n", stats.current_streaks.to_string().yellow()));
        output.push_str(&format!("  Avg Completion:    {:.1}%\n", stats.average_completion_rate));
        output
    }

    pub fn dashboard(habits: &[&Habit]) -> String {
        let mut output = String::new();
        let now = chrono::Local::now();
        
        output.push_str(&format!("\n{}\n", format!("{} Dashboard", now.format("%A, %B %d")).bold()));
        output.push_str(&format!("{}\n\n", "═".repeat(40).cyan()));

        let due: Vec<_> = habits.iter().filter(|h| h.is_due_today() && !h.is_completed_today()).collect();
        let done: Vec<_> = habits.iter().filter(|h| h.is_completed_today()).collect();
        let upcoming: Vec<_> = habits.iter().filter(|h| !h.is_due_today()).collect();

        if !due.is_empty() {
            output.push_str(&format!("{}\n", "Due Today:".yellow().bold()));
            for h in due {
                let icon = match h.priority {
                    Priority::Critical => "Critical",
                    Priority::High => "High",
                    _ => "Medium",
                };
                output.push_str(&format!("  {} {} ({} day streak)\n", icon, h.name, h.current_streak));
            }
            output.push('\n');
        }

        if !done.is_empty() {
            output.push_str(&format!("{}\n", "Completed:".green().bold()));
            for h in done {
                output.push_str(&format!("  ✓ {} ({}% rate)\n", h.name, h.completion_rate()));
            }
            output.push('\n');
        }

        let total = due.len() + done.len();
        let progress = if total > 0 { (done.len() * 100) / total } else { 0 };
        let bar = Self::progress_bar(progress);
        output.push_str(&format!("Progress: {} {}%\n\n", bar, progress));

        output
    }

    fn progress_bar(percent: usize) -> String {
        let filled = percent / 10;
        let empty = 10 - filled;
        format!("{}{}", 
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed())
    }
}