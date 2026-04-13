use std::fs;
use std::path::Path;
use serde_json;
use crate::models::Habit;

pub struct Storage {
    path: String,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn save(&self, habits: &[Habit]) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(habits)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Vec<Habit>, Box<dyn std::error::Error>> {
        if !Path::new(&self.path).exists() {
            return Ok(Vec::new());
        }
        
        let json = fs::read_to_string(&self.path)?;
        let habits = serde_json::from_str(&json)?;
        Ok(habits)
    }

    pub fn export_csv(&self, habits: &[Habit], path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut csv = String::from("id,name,category,current_streak,completion_rate\n");
        
        for h in habits {
            csv.push_str(&format!(
                "{},{},{},{},{:.1}\n",
                h.id, h.name, h.category, h.current_streak, h.completion_rate()
            ));
        }
        
        fs::write(path, csv)?;
        Ok(())
    }
}