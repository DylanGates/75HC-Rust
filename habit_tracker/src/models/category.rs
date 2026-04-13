use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HabitCategory {
    Sports,
    Study,
    Work,
    Health,
    Mindfulness,
    Creativity,
    Social,
    Finance,
    Other,
}

impl std::fmt::Display for HabitCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HabitCategory::Sports => write!(f, "Sports"),
            HabitCategory::Study => write!(f, "Study"),
            HabitCategory::Work => write!(f, "Work"),
            HabitCategory::Health => write!(f, "Health"),
            HabitCategory::Mindfulness => write!(f, "Mindfulness"),
            HabitCategory::Creativity => write!(f, "Creativity"),
            HabitCategory::Social => write!(f, "Social"),
            HabitCategory::Finance => write!(f, "Finance"),
            HabitCategory::Other => write!(f, "Other"),
        }
    }
}
