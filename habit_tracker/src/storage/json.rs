use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde_json;
use crate::models::Habit;

pub struct JsonStorage {
    data_dir: PathBuf,
    file_path: PathBuf,
}

impl JsonStorage {
    /// Create storage with default data directory (~/.local/share/habit-tracker/data)
    pub fn new() -> Result<Self> {
        let data_dir = Self::default_data_dir()?;
        Self::with_dir(data_dir)
    }

    /// Create storage with custom directory
    pub fn with_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let data_dir = dir.as_ref().to_path_buf();
        let file_path = data_dir.join("habits.json");
        
        // Ensure data directory exists
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;
        
        Ok(Self { data_dir, file_path })
    }

    fn default_data_dir() -> Result<PathBuf> {
        let base = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        Ok(base.join("habit-tracker").join("data"))
    }

    pub fn save(&self, habits: &[Habit]) -> Result<()> {
        // Create parent directories if needed
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(habits)
            .context("Failed to serialize habits")?;
        
        // Write to temp file first, then rename for atomicity
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, json)
            .with_context(|| format!("Failed to write to temp file: {:?}", temp_path))?;
        
        fs::rename(&temp_path, &self.file_path)
            .with_context(|| format!("Failed to save to {:?}", self.file_path))?;
        
        Ok(())
    }

    pub fn load(&self) -> Result<Vec<Habit>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read {:?}", self.file_path))?;
        
        let habits: Vec<Habit> = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse JSON from {:?}", self.file_path))?;
        
        Ok(habits)
    }

    pub fn export_csv(&self, habits: &[Habit], path: &Path) -> Result<()> {
        let mut csv = String::from("id,name,description,category,frequency,current_streak,longest_streak,completion_rate,total_completions,created_at,is_active\n");
        
        for h in habits {
            let freq_str = format!("{:?}", h.frequency).replace(',', ";");
            csv.push_str(&format!(
                "{},{},\"{}\",{},{},{},{},{:.2},{},{},{}\n",
                h.id,
                h.name.replace(',', ";"),
                h.description.replace(',', ";"),
                h.category,
                freq_str,
                h.current_streak,
                h.longest_streak,
                h.completion_rate(),
                h.completions.len(),
                h.created_at.to_rfc3339(),
                h.is_active
            ));
        }
        
        fs::write(path, csv)
            .with_context(|| format!("Failed to export CSV to {:?}", path))?;
        
        Ok(())
    }

    pub fn export_json(&self, habits: &[Habit], path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(habits)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn import_json(&self, path: &Path) -> Result<Vec<Habit>> {
        let json = fs::read_to_string(path)?;
        let habits = serde_json::from_str(&json)?;
        Ok(habits)
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Create backup with timestamp
    pub fn backup(&self) -> Result<PathBuf> {
        if !self.file_path.exists() {
            return Err(anyhow::anyhow!("No data file to backup"));
        }
        
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("habits_backup_{}.json", timestamp);
        let backup_path = self.data_dir.join(&backup_name);
        
        fs::copy(&self.file_path, &backup_path)?;
        println!("Backup created: {:?}", backup_path);
        
        Ok(backup_path)
    }

    /// List all backup files
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("habits_backup_") {
                backups.push(entry.path());
            }
        }
        
        backups.sort_by(|a, b| b.cmp(a)); // Newest first
        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_load() {
        let temp = TempDir::new().unwrap();
        let storage = JsonStorage::with_dir(temp.path()).unwrap();
        
        let habits = vec![];
        storage.save(&habits).unwrap();
        
        let loaded = storage.load().unwrap();
        assert!(loaded.is_empty());
    }
}