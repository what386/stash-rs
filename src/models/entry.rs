use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::models::item::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub working_directory: PathBuf,
    pub items: Vec<Item>,
    pub total_size_bytes: u64,
    pub was_destructive: bool,
}

impl Entry {
    pub fn new(
        name: Option<String>,
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

    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.short_id())
    }
}
