use std::{fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::Operation;

pub struct JournalStorage {
    journal: Vec<Operation>,
    log_file: PathBuf,
}

impl JournalStorage {
    pub fn new(log_file: &Path) -> Result<Self> {
        let mut storage = Self {
            journal: Vec::new(),
            log_file: log_file.to_path_buf(),
        };

        storage.load_operations()?;
        Ok(storage)
    }

    /// Load all journal from the journal.json file.
    fn load_operations(&mut self) -> Result<()> {
        if !self.log_file.exists() {
            self.journal.clear();
            return Ok(());
        }

        let json = fs::read_to_string(&self.log_file)
            .with_context(|| format!("Failed to read journal file {:?}", self.log_file))?;

        self.journal = serde_json::from_str(&json)
            .with_context(|| "Failed to deserialize journal")?;

        Ok(())
    }

    /// Save all journal to the journal.json file.
    pub fn save_operations(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.journal)
            .context("Failed to serialize journal")?;

        fs::write(&self.log_file, json)
            .with_context(|| format!("Failed to write journal file {:?}", self.log_file))?;

        Ok(())
    }

    /// Append an operation to the journal
    pub fn append(&mut self, operation: Operation) -> Result<()> {
        self.journal.push(operation);
        self.save_operations()
    }

    /// Get the most recent operation
    pub fn last(&self) -> Result<Option<Operation>> {
        Ok(self.journal.last().cloned())
    }

    /// Get journal since a specific time
    pub fn since(&self, since: DateTime<Utc>) -> Result<Vec<Operation>> {
        Ok(self.journal
            .iter()
            .filter(|op| op.timestamp > since)
            .cloned()
            .collect())
    }

    /// Get journal for a specific entry
    pub fn for_entry(&self, entry_id: &Uuid) -> Result<Vec<Operation>> {
        Ok(self.journal
            .iter()
            .filter(|op| op.involves_entry(entry_id))
            .cloned()
            .collect())
    }

    /// Get recent N journal
    pub fn recent(&self, n: usize) -> Result<Vec<Operation>> {
        let start = self.journal.len().saturating_sub(n);
        Ok(self.journal[start..].to_vec())
    }

    /// Clear the journal (use with caution!)
    pub fn clear(&mut self) -> Result<()> {
        self.journal.clear();
        self.save_operations()
    }

    /// Compact the journal (remove journal for deleted entries)
    pub fn compact(&mut self, existing_entry_ids: &[Uuid]) -> Result<()> {
        self.journal.retain(|op| {
            match op.entry_id() {
                Some(id) => existing_entry_ids.contains(&id),
                None => true, // Keep journal without entry_id (e.g. global ops)
            }
        });

        self.save_operations()
    }
}

