use crate::models::Habit;
use anyhow::{anyhow, Result};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

pub struct JsonStorage {
    data_dir: PathBuf,
    file_path: PathBuf,
}

impl JsonStorage {
    pub fn new() -> Result<Self> {
        let data_dir = Self::default_data_dir()?;
        Self::with_dir(data_dir)
    }

    pub fn with_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let data_dir = dir.as_ref().to_path_buf();
        let file_path = data_dir.join("habits.json");

        fs::create_dir_all(&data_dir)
            .map_err(|e| anyhow!("Failed to create data directory {:?}: {}", data_dir, e))?;

        Ok(Self {
            data_dir,
            file_path,
        })
    }

    fn default_data_dir() -> Result<PathBuf> {
        let base =
            dirs::data_local_dir().ok_or_else(|| anyhow!("Could not determine data directory"))?;
        Ok(base.join("habit-tracker").join("data"))
    }

    pub fn save(&self, habits: &[Habit]) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(habits)
            .map_err(|e| anyhow!("Failed to serialize habits: {}", e))?;

        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, &json)
            .map_err(|e| anyhow!("Failed to write to temp file {:?}: {}", temp_path, e))?;

        fs::rename(&temp_path, &self.file_path)
            .map_err(|e| anyhow!("Failed to save to {:?}: {}", self.file_path, e))?;

        Ok(())
    }

    pub fn load(&self) -> Result<Vec<Habit>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(&self.file_path)
            .map_err(|e| anyhow!("Failed to read {:?}: {}", self.file_path, e))?;

        let habits: Vec<Habit> = serde_json::from_str(&json)
            .map_err(|e| anyhow!("Failed to parse JSON from {:?}: {}", self.file_path, e))?;

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

        fs::write(path, csv).map_err(|e| anyhow!("Failed to export CSV to {:?}: {}", path, e))?;

        Ok(())
    }

    pub fn export_json(&self, habits: &[Habit], path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(habits)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn import_json(&self, path: &Path) -> Result<Vec<Habit>> {
        let json = fs::read_to_string(path)?;
        let habits: Vec<Habit> = serde_json::from_str(&json)?;
        Ok(habits)
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn backup(&self) -> Result<PathBuf> {
        if !self.file_path.exists() {
            return Err(anyhow!("No data file to backup"));
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("habits_backup_{}.json", timestamp);
        let backup_path = self.data_dir.join(&backup_name);

        fs::copy(&self.file_path, &backup_path)?;
        println!("Backup created: {:?}", backup_path);

        Ok(backup_path)
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("habits_backup_") {
                backups.push(entry.path());
            }
        }

        backups.sort_by(|a, b| b.cmp(a));
        Ok(backups)
    }
}
