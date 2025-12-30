use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub total_size_bytes: u64,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stash {
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub entries: Vec<EntryMetadata>,  // Changed from Vec<Entry>
    pub total_size_bytes: u64,
}

impl Default for Stash {
    fn default() -> Self {
        Self {
            name: None,
            created: Utc::now(),
            updated: Utc::now(),
            entries: Vec::new(),
            total_size_bytes: 0,
        }
    }
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

    pub fn add_entry(&mut self, uuid: Uuid, name: Option<String>, size: u64, item_count: usize) {
        let metadata = EntryMetadata {
            uuid,
            name,
            created: Utc::now(),
            total_size_bytes: size,
            item_count,
        };
        self.total_size_bytes += size;
        self.entries.push(metadata);
        self.touch();
    }

    pub fn remove_entry(&mut self, uuid: &Uuid) -> Option<EntryMetadata> {
        let pos = self.entries.iter().position(|e| &e.uuid == uuid)?;
        let entry = self.entries.remove(pos);
        self.total_size_bytes -= entry.total_size_bytes;
        self.touch();
        Some(entry)
    }

    pub fn get_metadata(&self, uuid: &Uuid) -> Option<&EntryMetadata> {
        self.entries.iter().find(|e| &e.uuid == uuid)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&EntryMetadata> {
        self.entries.iter().find(|e| e.name.as_deref() == Some(name))
    }

    pub fn find_by_identifier(&self, identifier: &str) -> Option<&EntryMetadata> {
        // Try UUID first
        if let Ok(uuid) = Uuid::parse_str(identifier) {
            if let Some(entry) = self.get_metadata(&uuid) {
                return Some(entry);
            }
        }
        // Fall back to name
        self.find_by_name(identifier)
    }

    pub fn search(&self, pattern: &str) -> Vec<&EntryMetadata> {
        let pattern_lower = pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&pattern_lower))
                    .unwrap_or(false)
                    || e.uuid.to_string().starts_with(&pattern_lower)
            })
            .collect()
    }

    pub fn remove_older_than_days(&mut self, days: i64) -> Vec<Uuid> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let (old, keep): (Vec<_>, Vec<_>) = self
            .entries
            .drain(..)
            .partition(|e| e.created < cutoff);

        self.entries = keep;
        self.total_size_bytes = self.entries.iter().map(|e| e.total_size_bytes).sum();

        if !old.is_empty() {
            self.touch();
        }

        old.into_iter().map(|e| e.uuid).collect()
    }

    pub fn most_recent(&self) -> Option<&EntryMetadata> {
        self.entries.last()
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
