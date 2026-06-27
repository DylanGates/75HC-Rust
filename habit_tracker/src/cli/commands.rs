use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Input, Select}; // Removed MultiSelect
use std::io::Write; // Added for flush
use std::path::Path;

use crate::cli::args::{CategoryArg, Commands, ExportFormat, FrequencyArg, PriorityArg, SortBy};
use crate::cli::display::TableFormatter;
use crate::cli::progress::{Color, ProgressBar};
use crate::models::{Habit, HabitCategory}; // Removed Priority
use crate::stats::StatsCalculator;
use crate::storage::JsonStorage;
use crate::tracker::HabitTracker;
use uuid::Uuid;

pub struct CommandHandler {
    tracker: HabitTracker,
    storage: JsonStorage,
    verbose: bool,
}

impl CommandHandler {
    pub fn new(data_dir: Option<String>, verbose: bool) -> Result<Self> {
        let storage = if let Some(dir) = data_dir {
            JsonStorage::with_dir(dir)?
        } else {
            JsonStorage::new()?
        };

        if verbose {
            eprintln!("Using data directory: {:?}", storage.data_dir());
        }

        let habits = storage.load().unwrap_or_default();
        let mut tracker = HabitTracker::new();

        for habit in habits {
            tracker.add(habit);
        }

        Ok(Self {
            tracker,
            storage,
            verbose,
        })
    }

    pub fn execute(&mut self, cmd: Commands) -> Result<()> {
        match cmd {
            Commands::Add {
                name,
                description,
                category,
                frequency,
                target,
                priority,
            } => {
                self.cmd_add(name, description, category, frequency, target, priority)?;
            }
            Commands::List {
                category,
                archived,
                due,
                done,
                sort,
            } => {
                self.cmd_list(category, archived, due, done, sort)?;
            }
            Commands::Done { id } => {
                self.cmd_done(id)?;
            }
            Commands::Show { id } => {
                self.cmd_show(id)?;
            }
            Commands::Edit {
                id,
                name,
                description,
                priority,
            } => {
                self.cmd_edit(id, name, description, priority)?;
            }
            Commands::Archive { id } => {
                self.cmd_archive(id)?;
            }
            Commands::Restore { index } => {
                self.cmd_restore(index)?;
            }
            Commands::Delete { id, force } => {
                self.cmd_delete(id, force)?;
            }
            Commands::Stats { by_category } => {
                self.cmd_stats(by_category)?;
            }
            Commands::Dashboard => {
                self.cmd_dashboard()?;
            }
            Commands::Export { path, format } => {
                self.cmd_export(path, format)?;
            }
            Commands::Import { path, merge } => {
                self.cmd_import(path, merge)?;
            }
            Commands::Interactive => {
                self.cmd_interactive()?;
            }
            Commands::Reset { really } => {
                if really {
                    self.cmd_reset()?;
                } else {
                    println!("Use --really to confirm data reset");
                }
            }
        }

        self.save()?;
        Ok(())
    }

    fn cmd_add(
        &mut self,
        name: String,
        desc: Option<String>,
        cat: CategoryArg,
        freq: FrequencyArg,
        target: u32,
        prio: PriorityArg,
    ) -> Result<()> {
        let habit = Habit::new(
            &name,
            &desc.unwrap_or_default(),
            cat.into(),
            freq.0,
            target,
            prio.into(),
        );

        let id = self.tracker.add(habit);

        let bar = ProgressBar::new(20)
            .with_chars('▓', '░')
            .with_colors(Color::Green, Color::Dimmed);
        println!("Creating habit...");
        for i in 1..=5 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            print!("\r{}", bar.render((i as f64 / 5.0) * 100.0));
            std::io::stdout().flush().unwrap();
        }
        println!();

        println!(
            "{} Created habit '{}' (ID: {})\n",
            "✓".green(),
            name.bold(),
            id.to_string().chars().take(8).collect::<String>().dimmed()
        );
        Ok(())
    }

    fn cmd_list(
        &self,
        category: Option<CategoryArg>,
        archived: bool,
        due: bool,
        done: bool,
        sort: SortBy,
    ) -> Result<()> {
        let habits: Vec<&Habit> = if archived {
            self.tracker.archived().iter().collect()
        } else {
            let mut filtered: Vec<_> = self.tracker.all();

            if let Some(cat) = category {
                let cat: HabitCategory = cat.into();
                filtered.retain(|h| h.category == cat);
            }

            if due {
                filtered.retain(|h| h.is_due_today() && !h.is_completed_today());
            }

            if done {
                filtered.retain(|h| h.is_completed_today());
            }

            filtered
        };

        if habits.is_empty() {
            println!(
                "{}",
                "No habits found. Create one with: habit-tracker add <name>".dimmed()
            );
            return Ok(());
        }

        println!("{}", TableFormatter::habits(&habits, &sort));

        let total = habits.len();
        let completed_today = habits.iter().filter(|h| h.is_completed_today()).count();
        let pct = if total > 0 {
            (completed_today as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let bar = ProgressBar::new(15).with_chars('█', '░').with_colors(
            if pct >= 80.0 {
                Color::Green
            } else {
                Color::Yellow
            },
            Color::Dimmed,
        );

        println!(
            "\n{} habits | {} done today | {}",
            total,
            format!("{}/{}", completed_today, total).cyan(),
            bar.render(pct)
        );

        Ok(())
    }

    fn cmd_done(&mut self, id: String) -> Result<()> {
        let uuid = self
            .resolve_id(&id)
            .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", id))?;

        let habit = self.tracker.get(uuid).unwrap();
        let name = habit.name.clone();

        match self.tracker.complete(uuid) {
            Ok(_) => {
                let new_streak = self.tracker.get(uuid).unwrap().current_streak;
                if new_streak % 7 == 0 {
                    println!("{}", "🎉 WEEKLY STREAK MILESTONE! 🎉".green().bold());
                }

                println!(
                    "{} Completed '{}'! Streak: {} {}",
                    "✓".green().bold(),
                    name,
                    new_streak.to_string().yellow().bold(),
                    "🔥".repeat((new_streak / 7).min(5) as usize)
                );
                Ok(())
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e);
                Ok(())
            }
        }
    }

    fn cmd_show(&self, id: String) -> Result<()> {
        let uuid = self
            .resolve_id(&id)
            .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", id))?;

        let habit = self
            .tracker
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Habit not found"))?;

        println!("{}", TableFormatter::habit_detail(habit));
        Ok(())
    }

    fn cmd_edit(
        &mut self,
        id: String,
        name: Option<String>,
        desc: Option<String>,
        priority: Option<PriorityArg>,
    ) -> Result<()> {
        let uuid = self
            .resolve_id(&id)
            .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", id))?;

        let mut updates = Vec::new();
        if let Some(n) = &name {
            updates.push(format!("name -> {}", n));
        }
        if let Some(_d) = &desc {
            updates.push("description updated".to_string());
        }
        if let Some(p) = &priority {
            updates.push(format!("priority -> {:?}", p));
        }

        self.tracker
            .update(uuid, name.as_deref(), desc.as_deref())?;

        if let Some(p) = priority {
            if let Some(h) = self.tracker.get_mut(uuid) {
                h.priority = p.into();
            }
        }

        println!("{} Updated: {}", "✓".green(), updates.join(", "));
        Ok(())
    }

    fn cmd_archive(&mut self, id: String) -> Result<()> {
        let uuid = self
            .resolve_id(&id)
            .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", id))?;

        self.tracker.archive(uuid)?;
        println!(
            "{} Archived habit (can restore with: restore <index>)",
            "📦".yellow()
        );
        Ok(())
    }

    fn cmd_restore(&mut self, index: usize) -> Result<()> {
        let id = self
            .tracker
            .restore(index)
            .map_err(|e| anyhow::anyhow!(e))?;
        println!("{} Restored habit {}", "✓".green(), id);
        Ok(())
    }

    fn cmd_delete(&mut self, id: String, force: bool) -> Result<()> {
        if !force {
            let confirm = Confirm::new()
                .with_prompt("⚠️  Permanently delete? This cannot be undone")
                .default(false)
                .interact()?;
            if !confirm {
                return Ok(());
            }
        }

        let uuid = self
            .resolve_id(&id)
            .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", id))?;

        self.storage.backup()?;

        self.tracker.delete_permanently(uuid)?;
        println!("{} Deleted permanently", "🗑️".red());
        Ok(())
    }

    fn cmd_stats(&self, _by_category: bool) -> Result<()> {
        let stats = StatsCalculator::calculate(&self.tracker.all());
        println!("{}", TableFormatter::stats(&stats, &self.tracker.all()));
        Ok(())
    }

    fn cmd_dashboard(&self) -> Result<()> {
        println!("{}", TableFormatter::dashboard(&self.tracker.all()));
        Ok(())
    }

    fn cmd_export(&self, path: String, format: ExportFormat) -> Result<()> {
        let habits: Vec<Habit> = self.tracker.all().iter().map(|&h| h.clone()).collect();
        let path = Path::new(&path);

        match format {
            ExportFormat::Csv => {
                self.storage.export_csv(&habits, path)?;
                println!(
                    "{} Exported {} habits to CSV: {:?}",
                    "✓".green(),
                    habits.len(),
                    path
                );
            }
            ExportFormat::Json => {
                self.storage.export_json(&habits, path)?;
                println!(
                    "{} Exported {} habits to JSON: {:?}",
                    "✓".green(),
                    habits.len(),
                    path
                );
            }
        }
        Ok(())
    }

    fn cmd_import(&mut self, path: String, merge: bool) -> Result<()> {
        let path = Path::new(&path);
        let imported = self.storage.import_json(path)?;

        if !merge {
            self.storage.backup()?;
            self.tracker = HabitTracker::new();
        }

        let mut count = 0;
        for habit in imported {
            self.tracker.add(habit);
            count += 1;
        }

        println!("{} Imported {} habits", "✓".green(), count);
        Ok(())
    }

    fn cmd_interactive(&mut self) -> Result<()> {
        loop {
            let choices = vec![
                "📋 List habits",
                "➕ Add new habit",
                "✅ Complete habit",
                "📊 Dashboard",
                "📈 Statistics",
                "🔍 Show habit details",
                "📦 Archive habit",
                "❌ Exit",
            ];

            let selection = Select::new()
                .with_prompt("What would you like to do?")
                .items(&choices)
                .default(0)
                .interact()?;

            match selection {
                0 => self.cmd_list(None, false, false, false, SortBy::Name)?,
                1 => {
                    let name: String = Input::new().with_prompt("Habit name").interact_text()?;
                    let desc: String = Input::new()
                        .with_prompt("Description (optional)")
                        .allow_empty(true)
                        .interact_text()?;

                    let cats = vec!["Sports", "Study", "Work", "Health", "Mindfulness", "Other"];
                    let cat_idx = Select::new()
                        .with_prompt("Category")
                        .items(&cats)
                        .default(0)
                        .interact()?;

                    let freqs = vec!["Daily", "Weekly (Mon/Wed/Fri)", "Custom (every 3 days)"];
                    let freq_idx = Select::new()
                        .with_prompt("Frequency")
                        .items(&freqs)
                        .default(0)
                        .interact()?;

                    let freq = match freq_idx {
                        1 => FrequencyArg(crate::models::HabitFrequency::Weekly {
                            days: vec![
                                chrono::Weekday::Mon,
                                chrono::Weekday::Wed,
                                chrono::Weekday::Fri,
                            ],
                        }),
                        2 => {
                            FrequencyArg(crate::models::HabitFrequency::Custom { interval_days: 3 })
                        }
                        _ => FrequencyArg(crate::models::HabitFrequency::Daily),
                    };

                    let cat = match cat_idx {
                        0 => CategoryArg::Sports,
                        1 => CategoryArg::Study,
                        2 => CategoryArg::Work,
                        3 => CategoryArg::Health,
                        4 => CategoryArg::Mindfulness,
                        _ => CategoryArg::Other,
                    };

                    self.cmd_add(name, Some(desc), cat, freq, 30, PriorityArg::Medium)?;
                }
                2 => {
                    let due = self.tracker.due_today();
                    if due.is_empty() {
                        println!("No habits due!");
                    } else {
                        let names: Vec<_> = due.iter().map(|h| h.name.as_str()).collect();
                        let idx = Select::new()
                            .with_prompt("Complete which?")
                            .items(&names)
                            .interact()?;
                        let id = due[idx].id.to_string();
                        self.cmd_done(id)?;
                    }
                }
                3 => self.cmd_dashboard()?,
                4 => self.cmd_stats(false)?,
                5 => {
                    let all = self.tracker.all();
                    let names: Vec<_> = all.iter().map(|h| h.name.as_str()).collect();
                    let idx = Select::new()
                        .with_prompt("Show which?")
                        .items(&names)
                        .interact()?;
                    let id = all[idx].id.to_string();
                    self.cmd_show(id)?;
                }
                6 => {
                    let all = self.tracker.all();
                    let names: Vec<_> = all.iter().map(|h| h.name.as_str()).collect();
                    let idx = Select::new()
                        .with_prompt("Archive which?")
                        .items(&names)
                        .interact()?;
                    let id = all[idx].id.to_string();
                    self.cmd_archive(id)?;
                }
                _ => break,
            }

            println!();
        }
        Ok(())
    }

    fn cmd_reset(&mut self) -> Result<()> {
        self.storage.backup()?;
        self.tracker = HabitTracker::new();
        println!("{} All data reset. Backup created.", "⚠️".yellow());
        Ok(())
    }

    fn resolve_id(&self, id: &str) -> Option<Uuid> {
        if let Ok(uuid) = Uuid::parse_str(id) {
            return Some(uuid);
        }

        let all = self.tracker.all();
        if id.len() >= 4 {
            for h in &all {
                if h.id.to_string().starts_with(id) {
                    return Some(h.id);
                }
            }
        }

        let id_lower = id.to_lowercase();
        all.iter()
            .find(|h| h.name.to_lowercase().contains(&id_lower))
            .map(|h| h.id)
    }

    fn save(&self) -> Result<()> {
        let habits: Vec<Habit> = self.tracker.all().iter().map(|&h| h.clone()).collect();
        self.storage.save(&habits)?;
        if self.verbose {
            eprintln!(
                "Saved {} habits to {:?}",
                habits.len(),
                self.storage.file_path()
            );
        }
        Ok(())
    }
}
