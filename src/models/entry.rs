use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::models::item::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub uuid: Uuid,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub working_directory: PathBuf,
    pub items: Vec<Item>,
    pub total_size_bytes: u64,
    pub was_destructive: bool,
}

impl Entry {
    pub fn new(
        name: String,
        items: Vec<Item>,
        working_directory: PathBuf,
        was_destructive: bool,
    ) -> Self {
        let total_size_bytes = items.iter().map(|e| e.size_bytes).sum();
        Self {
            uuid: Uuid::new_v4(),
            name,
            created: Utc::now(),
            updated: Utc::now(),
            working_directory,
            items,
            total_size_bytes,
            was_destructive,
        }
    }

    pub fn touch(&mut self) {
        self.updated = Utc::now();
    }

    pub fn age_hours(&self) -> i64 {
        (Utc::now() - self.created).num_hours()
    }

    pub fn age_days(&self) -> i64 {
        (Utc::now() - self.created).num_days()
    }

    pub fn short_id(&self) -> String {
        self.uuid.to_string()[..6].to_string()
    }

    pub fn contains_path(&self, path: &str) -> bool {
        self.items.iter().any(|item| {
            item.original_path.to_string_lossy().contains(path)
        })
    }

    pub fn get_item(&self, original_path: &Path) -> Option<&Item> {
        self.items.iter().find(|item| item.original_path == original_path)
    }

    pub fn file_count(&self) -> usize {
        self.items.len()
    }

    pub fn recalculate_size(&mut self) {
        self.total_size_bytes = self.items.iter().map(|i| i.size_bytes).sum();
        self.touch();
    }
}
