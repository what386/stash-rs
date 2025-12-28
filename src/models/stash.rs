use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::entry::Entry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stash {
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub entries: Vec<Entry>,
    pub total_size_bytes: u64,
}

impl Stash {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            created: Utc::now(),
            updated: Utc::now(),
            entries: Vec::new(),
            total_size_bytes: 0,
        }
    }

    pub fn push(&mut self, entry: Entry) {
        self.total_size_bytes += entry.total_size_bytes;
        self.entries.push(entry);
        self.touch();
    }

    pub fn pop(&mut self) -> Option<Entry> {
        let entry = self.entries.pop()?;
        self.total_size_bytes -= entry.total_size_bytes;
        self.touch();
        Some(entry)
    }

    pub fn remove(&mut self, uuid: &Uuid) -> Option<Entry> {
        let pos = self.entries.iter().position(|e| &e.uuid == uuid)?;
        let entry = self.entries.remove(pos);
        self.total_size_bytes -= entry.total_size_bytes;
        self.touch();
        Some(entry)
    }

    pub fn get(&self, uuid: &Uuid) -> Option<&Entry> {
        self.entries.iter().find(|e| &e.uuid == uuid)
    }

    pub fn find(&self, identifier: &str) -> Option<&Entry> {
        if let Ok(uuid) = Uuid::parse_str(identifier) {
            if let Some(entry) = self.get(&uuid) {
                return Some(entry);
            }
        }
        self.entries.iter().find(|e| e.name.as_deref() == Some(identifier))
    }

    pub fn search(&self, pattern: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| {
                e.display_name().to_lowercase().contains(&pattern.to_lowercase())
                    || e.items.iter().any(|i| i.matches_pattern(pattern))
            })
            .collect()
    }

    pub fn remove_older_than_hours(&mut self, hours: i64) -> Vec<Entry> {
        let (old, keep): (Vec<_>, Vec<_>) = self.entries
            .drain(..)
            .partition(|e| e.age_hours() > hours);

        self.entries = keep;
        self.total_size_bytes = self.entries.iter().map(|e| e.total_size_bytes).sum();
        if !old.is_empty() {
            self.touch();
        }
        old
    }

    pub fn remove_older_than_days(&mut self, days: i64) -> Vec<Entry> {
        self.remove_older_than_hours(days * 24)
    }


    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn touch(&mut self) {
        self.updated = Utc::now();
    }
}
