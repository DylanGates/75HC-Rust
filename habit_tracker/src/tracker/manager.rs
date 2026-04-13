use crate::models::{Habit, HabitCategory, HabitError, Priority};
use std::collections::HashMap;
use uuid::Uuid;

pub struct HabitTracker {
    habits: HashMap<Uuid, Habit>,
    archived: Vec<Habit>,
}

impl HabitTracker {
    pub fn new() -> Self {
        Self {
            habits: HashMap::new(),
            archived: Vec::new(),
        }
    }

    pub fn add(&mut self, habit: Habit) -> Uuid {
        let id = habit.id;
        self.habits.insert(id, habit);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Habit> {
        self.habits.get(&id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Habit> {
        self.habits.get_mut(&id)
    }

    pub fn complete(&mut self, id: Uuid) -> Result<(), HabitError> {
        self.get_mut(id).ok_or(HabitError::NotFound)?.complete()
    }

    pub fn update(
        &mut self,
        id: Uuid,
        name: Option<&str>,
        desc: Option<&str>,
    ) -> Result<(), HabitError> {
        let habit = self.get_mut(id).ok_or(HabitError::NotFound)?;
        if let Some(n) = name {
            habit.name = n.to_string();
        }
        if let Some(d) = desc {
            habit.description = d.to_string();
        }
        Ok(())
    }

    pub fn archive(&mut self, id: Uuid) -> Result<(), HabitError> {
        let habit = self.habits.remove(&id).ok_or(HabitError::NotFound)?;
        self.archived.push(habit);
        Ok(())
    }

    pub fn restore(&mut self, index: usize) -> Result<Uuid, String> {
        if index >= self.archived.len() {
            return Err("Invalid index".to_string());
        }
        let mut habit = self.archived.remove(index);
        habit.is_active = true;
        let id = habit.id;
        self.habits.insert(id, habit);
        Ok(id)
    }

    pub fn delete_permanently(&mut self, id: Uuid) -> Result<(), HabitError> {
        self.habits.remove(&id).ok_or(HabitError::NotFound)?;
        Ok(())
    }

    pub fn all(&self) -> Vec<&Habit> {
        self.habits.values().collect()
    }

    pub fn by_category(&self, cat: HabitCategory) -> Vec<&Habit> {
        self.habits.values().filter(|h| h.category == cat).collect()
    }

    pub fn by_priority(&self, p: Priority) -> Vec<&Habit> {
        self.habits.values().filter(|h| h.priority == p).collect()
    }

    pub fn due_today(&self) -> Vec<&Habit> {
        self.habits.values().filter(|h| h.is_due_today()).collect()
    }

    pub fn active(&self) -> Vec<&Habit> {
        self.habits.values().filter(|h| h.is_active).collect()
    }

    pub fn archived(&self) -> &[Habit] {
        &self.archived
    }

    pub fn count(&self) -> usize {
        self.habits.len()
    }
}
