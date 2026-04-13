use crate::cli::args::SortBy;
use crate::cli::progress::{Color, ProgressBar, StreakBar, WeeklyProgressBar};
use crate::models::{Habit, Priority};
use crate::stats::TrackerStats;
use colored::*;
use comfy_table::{Cell, ContentArrangement, Table};

pub struct TableFormatter;

impl TableFormatter {
    pub fn habits(habits: &[&Habit], sort_by: &SortBy) -> String {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.load_preset(comfy_table::presets::UTF8_FULL);

        table.set_header(vec![
            Cell::new("ID").fg(comfy_table::Color::Cyan),
            Cell::new("Name"),
            Cell::new("Category").fg(comfy_table::Color::Blue),
            Cell::new("Streak"),
            Cell::new("Weekly"),
            Cell::new("Rate"),
            Cell::new("Status"),
        ]);

        let mut sorted: Vec<_> = habits.iter().copied().collect();
        match sort_by {
            SortBy::Name => sorted.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Streak => sorted.sort_by(|a, b| b.current_streak.cmp(&a.current_streak)),
            SortBy::Rate => sorted.sort_by(|a, b| {
                b.completion_rate()
                    .partial_cmp(&a.completion_rate())
                    .unwrap()
            }),
            SortBy::Priority => sorted.sort_by(|a, b| b.priority.cmp(&a.priority)),
            SortBy::Created => sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        for h in sorted {
            let status = if h.is_completed_today() {
                "✓ DONE".green().bold()
            } else if h.is_due_today() {
                "○ DUE".yellow()
            } else {
                "-".dimmed()
            };

            let prio_color = match h.priority {
                Priority::Critical => comfy_table::Color::Red,
                Priority::High => comfy_table::Color::Yellow,
                Priority::Medium => comfy_table::Color::White,
                Priority::Low => comfy_table::Color::Grey,
            };

            let (weekly_done, weekly_target) = h.weekly_progress();

            table.add_row(vec![
                Cell::new(h.id.to_string().split('-').next().unwrap_or("").to_string())
                    .fg(comfy_table::Color::DarkGrey),
                Cell::new(&h.name).fg(prio_color),
                Cell::new(h.category.to_string()).fg(comfy_table::Color::Blue),
                Cell::new(StreakBar::render(h.current_streak, h.longest_streak)),
                Cell::new(WeeklyProgressBar::render(weekly_done, weekly_target)),
                Cell::new(format!("{:.0}%", h.completion_rate())),
                Cell::new(status.to_string()),
            ]);
        }

        table.to_string()
    }

    pub fn habit_detail(habit: &Habit) -> String {
        let mut output = String::new();

        let rate = habit.completion_rate();
        let rate_bar = ProgressBar::new(20).with_chars('█', '░').with_colors(
            if rate >= 80.0 {
                Color::Green
            } else if rate >= 50.0 {
                Color::Yellow
            } else {
                Color::Red
            },
            Color::Dimmed,
        );

        output.push_str(&format!("\n{}\n", "╔".to_string() + &"═".repeat(48) + "╗"));
        output.push_str(&format!("║ {:^46} ║\n", habit.name.white().bold()));
        output.push_str(&format!("{}\n", "╠".to_string() + &"═".repeat(48) + "╣"));

        output.push_str(&format!(
            "║ {} {:>36} ║\n",
            "Progress:".dimmed(),
            rate_bar.render_with_label(rate, "")
        ));

        let (weekly_done, weekly_target) = habit.weekly_progress();
        let weekly_bar = WeeklyProgressBar::render(weekly_done, weekly_target);
        output.push_str(&format!("║ {} {:>36} ║\n", "Weekly:".dimmed(), weekly_bar));

        output.push_str(&format!(
            "║ {} {:>36} ║\n",
            "Streak:".dimmed(),
            StreakBar::render(habit.current_streak, habit.longest_streak)
        ));

        output.push_str(&format!("{}\n", "╠".to_string() + &"═".repeat(48) + "╣"));

        output.push_str(&format!("║  {} {}\n", "ID:".dimmed(), habit.id));
        output.push_str(&format!(
            "║  {} {}\n",
            "Description:".dimmed(),
            habit.description
        ));
        output.push_str(&format!(
            "║  {} {}\n",
            "Category:".dimmed(),
            habit.category.to_string().cyan()
        ));
        output.push_str(&format!(
            "║  {} {:?}\n",
            "Priority:".dimmed(),
            habit.priority
        ));
        output.push_str(&format!(
            "║  {} {:?}\n",
            "Frequency:".dimmed(),
            habit.frequency
        ));
        output.push_str(&format!(
            "║  {} {}\n",
            "Created:".dimmed(),
            habit.created_at.format("%Y-%m-%d")
        ));
        output.push_str(&format!(
            "║  {} {}/{}\n",
            "Target:".dimmed(),
            habit.completions.len(),
            habit.target_completions
        ));

        output.push_str(&format!("{}\n", "╠".to_string() + &"═".repeat(48) + "╣"));

        if !habit.completions.is_empty() {
            output.push_str(&format!("║  {}\n", "Recent completions:".dimmed()));
            for (i, c) in habit.completions.iter().rev().take(5).enumerate() {
                let marker = if i == 0 {
                    "└─►".green()
                } else {
                    "   ".dimmed()
                };
                output.push_str(&format!(
                    "║  {} {} {}\n",
                    marker,
                    c.format("%Y-%m-%d %H:%M").to_string().white(),
                    if i == 0 {
                        "(latest)".dimmed()
                    } else {
                        "".normal()
                    }
                ));
            }
        }

        output.push_str(&format!("{}\n", "╚".to_string() + &"═".repeat(48) + "╝"));
        output
    }

    pub fn stats(stats: &TrackerStats, habits: &[&Habit]) -> String {
        let mut output = String::new();

        let overall = ProgressBar::new(30)
            .with_chars('▓', '░')
            .with_colors(Color::Cyan, Color::Dimmed);

        output.push_str(&format!("\n{}\n", "📊 STATISTICS".bold().underline()));
        output.push_str(&format!(
            "{}\n\n",
            overall.render(stats.average_completion_rate)
        ));

        output.push_str(&format!(
            "  {} {:>20}\n",
            "Total Habits:".dimmed(),
            stats.total_habits
        ));
        output.push_str(&format!(
            "  {} {:>20}\n",
            "Active:".dimmed(),
            stats.active_habits.to_string().green()
        ));
        output.push_str(&format!(
            "  {} {:>20}\n",
            "Completed Today:".dimmed(),
            if stats.completed_today > 0 {
                stats.completed_today.to_string().cyan().bold()
            } else {
                "0".dimmed()
            }
        ));
        output.push_str(&format!(
            "  {} {:>20}\n",
            "This Week:".dimmed(),
            stats.completed_this_week
        ));
        output.push_str(&format!(
            "  {} {:>20}\n",
            "On Fire (streak):".dimmed(),
            stats.current_streaks.to_string().yellow()
        ));
        output.push_str(&format!(
            "  {} {:>20}\n",
            "Avg Completion:".dimmed(),
            format!("{:.1}%", stats.average_completion_rate)
        ));

        use std::collections::HashMap;
        let mut by_cat: HashMap<String, (usize, u32)> = HashMap::new();
        for h in habits {
            let entry = by_cat.entry(h.category.to_string()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += h.current_streak;
        }

        if !by_cat.is_empty() {
            output.push_str(&format!("\n{}\n", "By Category:".dimmed().underline()));
            for (cat, (count, total_streak)) in by_cat {
                let avg = if count > 0 {
                    total_streak / count as u32
                } else {
                    0
                };
                let bar = ProgressBar::new(15)
                    .with_chars('█', '░')
                    .with_colors(Color::Blue, Color::Dimmed);
                let percent = (count as f64 / habits.len() as f64) * 100.0;
                output.push_str(&format!(
                    "  {:12} {} {:>2} habits (avg streak: {})\n",
                    cat,
                    bar.render(percent),
                    count,
                    avg
                ));
            }
        }

        output
    }

    pub fn dashboard(habits: &[&Habit]) -> String {
        let mut output = String::new();
        let now = chrono::Local::now();

        let header_bar = ProgressBar::new(40)
            .with_chars('━', '─')
            .with_colors(Color::Cyan, Color::Dimmed);

        output.push_str(&format!("\n{}\n", header_bar.render(100.0)));
        output.push_str(&format!(
            "{:^50}\n",
            format!("📅 {} {}", now.format("%A"), now.format("%B %d"))
                .bold()
                .white()
        ));
        output.push_str(&format!("{}\n\n", header_bar.render(100.0)));

        let due: Vec<_> = habits
            .iter()
            .filter(|h| h.is_due_today() && !h.is_completed_today())
            .collect();
        let done: Vec<_> = habits.iter().filter(|h| h.is_completed_today()).collect();
        let upcoming: Vec<_> = habits.iter().filter(|h| !h.is_due_today()).collect();

        let total_active = due.len() + done.len();
        let progress_pct = if total_active > 0 {
            (done.len() as f64 / total_active as f64) * 100.0
        } else {
            0.0
        };

        let summary_bar = ProgressBar::new(25).with_chars('█', '░').with_colors(
            if progress_pct >= 80.0 {
                Color::Green
            } else if progress_pct >= 50.0 {
                Color::Yellow
            } else {
                Color::Red
            },
            Color::Dimmed,
        );

        output.push_str(&format!(
            "Daily Progress: {} {:.0}%\n\n",
            summary_bar.render(progress_pct),
            progress_pct
        ));

        if !due.is_empty() {
            output.push_str(&format!("{}\n", "🔥 DUE TODAY".yellow().bold()));
            for h in &due {
                // Fixed: borrow instead of move
                let urgency = match h.priority {
                    Priority::Critical => "🔴 CRITICAL",
                    Priority::High => "🟠 HIGH  ",
                    Priority::Medium => "🟡 MEDIUM",
                    Priority::Low => "🟢 LOW   ",
                };
                let streak_fire = if h.current_streak > 7 {
                    "🔥🔥"
                } else if h.current_streak > 3 {
                    "🔥"
                } else {
                    ""
                };
                output.push_str(&format!(
                    "  {} {} {} ({} day streak {})\n",
                    "○",
                    urgency,
                    h.name.white(),
                    h.current_streak,
                    streak_fire
                ));
            }
            output.push('\n');
        }

        if !done.is_empty() {
            output.push_str(&format!("{}\n", "✅ COMPLETED".green().bold()));
            for h in &done {
                // Fixed: borrow instead of move
                let rate_bar = ProgressBar::new(10)
                    .with_chars('✓', '·')
                    .with_colors(Color::Green, Color::Dimmed);
                output.push_str(&format!(
                    "  {} {} {} {:.0}%\n",
                    "✓".green(),
                    h.name.dimmed(),
                    rate_bar.render(h.completion_rate()),
                    h.completion_rate()
                ));
            }
            output.push('\n');
        }

        if !upcoming.is_empty() && due.is_empty() {
            output.push_str(&format!("{}\n", "📋 UPCOMING".dimmed()));
            for h in upcoming.iter().take(3) {
                output.push_str(&format!(
                    "  - {} ({})\n",
                    h.name.dimmed(),
                    format!("{:?}", h.frequency).dimmed()
                ));
            }
        }

        let footer = if progress_pct >= 100.0 {
            "🎉 All habits complete! Amazing work!".green().bold()
        } else if progress_pct >= 50.0 {
            "💪 More than halfway there! Keep going!".yellow()
        } else if !due.is_empty() {
            "🚀 Let's tackle those habits!".cyan()
        } else {
            "✨ Ready to start your day!".dimmed()
        };
        output.push_str(&format!("\n{}\n", footer));

        output
    }
}
