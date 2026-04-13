use habit_tracker::{Habit, HabitCategory, HabitFrequency, HabitTracker, Priority, TrackerStats, StatsCalculator, Storage};
use chrono::Weekday;

fn main() {
    let mut tracker = HabitTracker::new();

    // Create habits
    let exercise = Habit::new(
        "Morning Run",
        "5km every morning",
        HabitCategory::Sports,
        HabitFrequency::Daily,
        30,
        Priority::High,
    );

    let reading = Habit::new(
        "Read Books",
        "20 pages",
        HabitCategory::Study,
        HabitFrequency::Daily,
        20,
        Priority::Medium,
    );

    let gym = Habit::new(
        "Gym",
        "Weight training",
        HabitCategory::Sports,
        HabitFrequency::Weekly { 
            days: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri] 
        },
        12,
        Priority::High,
    );

    // Add to tracker
    let ex_id = tracker.add(exercise);
    let read_id = tracker.add(reading);
    let gym_id = tracker.add(gym);

    // Complete some habits
    tracker.complete(ex_id).unwrap();
    tracker.complete(read_id).unwrap();

    // Print stats
    let stats = StatsCalculator::calculate(&tracker.all());
    println!("Stats: {:#?}", stats);

    // Print due today
    println!("\nDue Today:");
    for h in tracker.due_today() {
        println!("  - {} [{}]", h.name, h.priority_string());
    }

    // Print by category
    println!("\n🏃 Sports:");
    for h in tracker.by_category(HabitCategory::Sports) {
        let (done, target) = h.weekly_progress();
        println!("  - {}: {}/{} this week", h.name, done, target);
    }

    // Save to file
    let storage = Storage::new("habits.json");
    let all_habits: Vec<Habit> = tracker.all().iter().map(|&h| h.clone()).collect();
    storage.save(&all_habits).unwrap();
    println!("\nSaved to habits.json");

    // Export CSV
    storage.export_csv(&all_habits, "habits.csv").unwrap();
    println!("Exported to habits.csv");
}

// Extension trait for display
trait PriorityDisplay {
    fn priority_string(&self) -> &'static str;
}

impl PriorityDisplay for Habit {
    fn priority_string(&self) -> &'static str {
        match self.priority {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}