
struct Habit {
    name: String,
    description: String,
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
