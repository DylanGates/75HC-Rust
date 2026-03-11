use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub created: String,
}

pub struct JsonDb {
    path: PathBuf,
}

impl JsonDb {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        JsonDb { path: path.into() }
    }

    fn load_all(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let s = fs::read_to_string(&self.path)?;
        let items: Vec<Item> = serde_json::from_str(&s)?;
        Ok(items)
    }

    fn save_all(&self, items: &[Item]) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = self.path.with_extension("tmp");
        let mut f = File::create(&tmp)?;
        let s = serde_json::to_string_pretty(items)?;
        f.write_all(s.as_bytes())?;
        f.sync_all()?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        self.load_all()
    }

    pub fn create(&self, name: String, notes: Option<String>) -> Result<Item, Box<dyn std::error::Error>> {
        let mut items = self.load_all()?;
        let item = Item {
            id: Uuid::new_v4().to_string(),
            name,
            notes,
            created: Utc::now().to_rfc3339(),
        };
        items.insert(0, item.clone());
        self.save_all(&items)?;
        Ok(item)
    }

    pub fn read(&self, id: &str) -> Result<Option<Item>, Box<dyn std::error::Error>> {
        let items = self.load_all()?;
        Ok(items.into_iter().find(|i| i.id == id))
    }

    pub fn update(&self, id: &str, name: Option<String>, notes: Option<String>) -> Result<Option<Item>, Box<dyn std::error::Error>> {
        let mut items = self.load_all()?;
        if let Some(pos) = items.iter().position(|i| i.id == id) {
            if let Some(n) = name {
                items[pos].name = n;
            }
            if notes.is_some() {
                items[pos].notes = notes;
            }
            let updated = items[pos].clone();
            self.save_all(&items)?;
            return Ok(Some(updated));
        }
        Ok(None)
    }

    pub fn delete(&self, id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut items = self.load_all()?;
        let orig = items.len();
        items.retain(|i| i.id != id);
        if items.len() == orig {
            return Ok(false);
        }
        self.save_all(&items)?;
        Ok(true)
    }
}
