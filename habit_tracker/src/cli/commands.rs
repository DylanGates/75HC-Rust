use colored::*;
use dialoguer::{Select, Input, Confirm};
use crate::cli::args::{Commands, parse_frequency};
use crate::cli::display::TableFormatter;
use crate::models::{Habit, HabitCategory, HabitFrequency, Priority};
use crate::tracker::HabitTracker;
use crate::stats::StatsCalculator;
use crate::storage::Storage;
use uuid::Uuid;

pub struct CommandHandler {
    tracker: HabitTracker,
    storage: Storage,
}

impl CommandHandler {
    pub fn new() -> Self {
        let storage = Storage::new("habits.json");
        let habits = storage.load().unwrap_or_default();
        let mut tracker = HabitTracker::new();
        
        for habit in habits {
            tracker.add(habit);
        }
        
        Self { tracker, storage }
    }

    pub fn execute(&mut self, cmd: Commands) {
        match cmd {
            Commands::Add { name, description, category, frequency, target, priority } => {
                self.cmd_add(name, description, category, frequency, target, priority);
            }
            Commands::List { category, archived, due } => {
                self.cmd_list(category, archived, due);
            }
            Commands::Done { id } => {
                self.cmd_done(id);
            }
            Commands::Show { id } => {
                self.cmd_show(id);
            }
            Commands::Edit { id, name, description } => {
                self.cmd_edit(id, name, description);
            }
            Commands::Archive { id } => {
                self.cmd_archive(id);
            }
            Commands::Restore { index } => {
                self.cmd_restore(index);
            }
            Commands::Delete { id, force } => {
                self.cmd_delete(id, force);
            }
            Commands::Stats => {
                self.cmd_stats();
            }
            Commands::Dashboard => {
                self.cmd_dashboard();
            }
            Commands::Export { path } => {
                self.cmd_export(path);
            }
            Commands::Interactive => {
                self.cmd_interactive();
            }
        }
        
        self.save();
    }

    fn cmd_add(&mut self, name: String, desc: Option<String>, cat: String, freq: String, target: u32, prio: String) {
        let category = Commands::parse_category(&cat).unwrap_or(HabitCategory::Other);
        let priority = Commands::parse_priority(&prio).unwrap_or(Priority::Medium);
        let frequency = parse_frequency(&freq).unwrap_or(HabitFrequency::Daily);
        
        let habit = Habit::new(
            &name,
            &desc.unwrap_or_default(),
            category,
            frequency,
            target,
            priority,
        );
        
        let id = self.tracker.add(habit);
        println!("{} Created habit '{}' with ID {}", "✓".green(), name, id.to_string().dimmed());
    }

    fn cmd_list(&self, category: Option<String>, archived: bool, due: bool) {
        let habits: Vec<&Habit> = if archived {
            self.tracker.archived().iter().collect()
        } else if let Some(cat) = category {
            let cat = Commands::parse_category(&cat).unwrap();
            self.tracker.by_category(cat)
        } else if due {
            self.tracker.due_today()
        } else {
            self.tracker.all()
        };

        if habits.is_empty() {
            println!("{}", "No habits found".dimmed());
            return;
        }

        println!("{}", TableFormatter::habits(&habits));
        println!("{} habits total", habits.len());
    }

    fn cmd_done(&mut self, id: String) {
        let uuid = self.resolve_id(&id);
        match uuid {
            Some(uid) => match self.tracker.complete(uid) {
                Ok(_) => println!("{} Marked habit as complete!", "✓".green()),
                Err(e) => println!("{} {}", "✗".red(), e),
            },
            None => println!("{} Habit not found", "✗".red()),
        }
    }

    fn cmd_show(&self, id: String) {
        let uuid = self.resolve_id(&id);
        match uuid.and_then(|u| self.tracker.get(u)) {
            Some(h) => println!("{}", TableFormatter::habit_detail(h)),
            None => println!("{} Habit not found", "✗".red()),
        }
    }

    fn cmd_edit(&mut self, id: String, name: Option<String>, desc: Option<String>) {
        let uuid = self.resolve_id(&id);
        match uuid {
            Some(uid) => match self.tracker.update(uid, name.as_deref(), desc.as_deref()) {
                Ok(_) => println!("{} Updated habit", "✓".green()),
                Err(e) => println!("{} {}", "✗".red(), e),
            },
            None => println!("{} Habit not found", "✗".red()),
        }
    }

    fn cmd_archive(&mut self, id: String) {
        let uuid = self.resolve_id(&id);
        match uuid {
            Some(uid) => match self.tracker.archive(uid) {
                Ok(_) => println!("{} Archived habit", "✓".green()),
                Err(e) => println!("{} {}", "✗".red(), e),
            },
            None => println!("{} Habit not found", "✗".red()),
        }
    }

    fn cmd_restore(&mut self, index: usize) {
        match self.tracker.restore(index) {
            Ok(id) => println!("{} Restored habit {}", "✓".green(), id),
            Err(e) => println!("{} {}", "✗".red(), e),
        }
    }

    fn cmd_delete(&mut self, id: String, force: bool) {
        if !force {
            let confirm = Confirm::new()
                .with_prompt("Are you sure? This cannot be undone")
                .default(false)
                .interact()
                .unwrap();
            if !confirm { return; }
        }
        
        let uuid = self.resolve_id(&id);
        match uuid {
            Some(uid) => match self.tracker.delete_permanently(uid) {
                Ok(_) => println!("{} Deleted habit", "✓".green()),
                Err(e) => println!("{} {}", "✗".red(), e),
            },
            None => println!("{} Habit not found", "✗".red()),
        }
    }

    fn cmd_stats(&self) {
        let stats = StatsCalculator::calculate(&self.tracker.all());
        println!("{}", TableFormatter::stats(&stats));
    }

    fn cmd_dashboard(&self) {
        println!("{}", TableFormatter::dashboard(&self.tracker.all()));
    }

    fn cmd_export(&self, path: String) {
        let habits: Vec<Habit> = self.tracker.all().iter().map(|&h| h.clone()).collect();
        match self.storage.export_csv(&habits, &path) {
            Ok(_) => println!("{} Exported to {}", "✓".green(), path),
            Err(e) => println!("{} {}", "✗".red(), e),
        }
    }

    fn cmd_interactive(&mut self) {
        loop {
            let choices = vec![
                "List habits",
                "Add habit",
                "Complete habit",
                "Dashboard",
                "Stats",
                "Exit",
            ];
            
            let selection = Select::new()
                .with_prompt("What would you like to do?")
                .items(&choices)
                .default(0)
                .interact()
                .unwrap();

            match selection {
                0 => self.cmd_list(None, false, false),
                1 => {
                    let name: String = Input::new().with_prompt("Name").interact_text().unwrap();
                    let desc: String = Input::new().with_prompt("Description").allow_empty(true).interact_text().unwrap();
                    self.cmd_add(name, Some(desc), "other".to_string(), "daily".to_string(), 30, "medium".to_string());
                }
                2 => {
                    let id: String = Input::new().with_prompt("Habit ID").interact_text().unwrap();
                    self.cmd_done(id);
                }
                3 => self.cmd_dashboard(),
                4 => self.cmd_stats(),
                _ => break,
            }
            
            println!();
        }
    }

    fn resolve_id(&self, id: &str) -> Option<Uuid> {
        // Try direct UUID parse
        if let Ok(uuid) = Uuid::parse_str(id) {
            return Some(uuid);
        }
        
        // Try find by name (case insensitive)
        let id_lower = id.to_lowercase();
        self.tracker.all().iter()
            .find(|h| h.name.to_lowercase() == id_lower)
            .map(|h| h.id)
    }

    fn save(&self) {
        let habits: Vec<Habit> = self.tracker.all().iter().map(|&h| h.clone()).collect();
        let _ = self.storage.save(&habits);
    }
}