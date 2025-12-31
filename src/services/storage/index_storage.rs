use std::path::{Path, PathBuf};
use std::{fs};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use crate::models::{Index, EntryMetadata};

pub struct IndexStorage {
    stash: Index,
    stash_file: PathBuf,
}

impl IndexStorage {
    pub fn new(stash_file: &Path) -> Result<Self> {
        let mut storage = Self {
            stash: Index::default(),
            stash_file: stash_file.to_path_buf(),
        };
        storage.load_packages()?;
        Ok(storage)
    }

    /// Load stash from the index.json file.
    fn load_packages(&mut self) -> Result<()> {
        if !self.stash_file.exists() {
            self.stash = Index::default();
            return Ok(());
        }
        match fs::read_to_string(&self.stash_file) {
            Ok(json) => {
                self.stash = serde_json::from_str(&json).unwrap_or_default();
                Ok(())
            }
            Err(e) => Err(anyhow!("Warning: Failed to load stash: {}", e)),
        }
    }

    /// Save stash to the index.json file.
    pub fn save_packages(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.stash_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create index directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(&self.stash)
            .map_err(|e| anyhow!("Failed to serialize index: {}", e))?;

        fs::write(&self.stash_file, json)
            .map_err(|e| anyhow!("Failed to write index file: {}", e))?;

        Ok(())
    }

    /// Reload the index from disk
    pub fn reload(&mut self) -> Result<()> {
        self.load_packages()
    }

    /// Get a reference to the entire index
    pub fn index(&self) -> &Index {
        &self.stash
    }

    /// Get a mutable reference to the entire index
    pub fn index_mut(&mut self) -> &mut Index {
        &mut self.stash
    }

    /// Add a new entry to the index and save
    pub fn add_entry(&mut self, uuid: Uuid, name: Option<String>, size: u64, item_count: usize) -> Result<()> {
        self.stash.add_entry(uuid, name, size, item_count);
        self.save_packages()
    }

    /// Remove an entry by UUID and save
    pub fn remove_entry(&mut self, uuid: &Uuid) -> Result<Option<EntryMetadata>> {
        let entry = self.stash.remove_entry(uuid);
        if entry.is_some() {
            self.save_packages()?;
        }
        Ok(entry)
    }

    /// Get metadata for a specific entry
    pub fn get_metadata(&self, uuid: &Uuid) -> Option<&EntryMetadata> {
        self.stash.get_metadata(uuid)
    }

    /// Find entry by name
    pub fn find_by_name(&self, name: &str) -> Option<&EntryMetadata> {
        self.stash.find_by_name(name)
    }

    /// Find entry by identifier (UUID or name)
    pub fn find_by_identifier(&self, identifier: &str) -> Option<&EntryMetadata> {
        self.stash.find_by_identifier(identifier)
    }

    /// Search entries by pattern
    pub fn search(&self, pattern: &str) -> Vec<&EntryMetadata> {
        self.stash.search(pattern)
    }

    /// Remove entries older than specified days and save
    pub fn remove_older_than_days(&mut self, days: i64) -> Result<Vec<Uuid>> {
        let removed = self.stash.remove_older_than_days(days);
        if !removed.is_empty() {
            self.save_packages()?;
        }
        Ok(removed)
    }

    /// Get the most recently created entry
    pub fn most_recent(&self) -> Option<&EntryMetadata> {
        self.stash.most_recent()
    }

    /// List all entries
    pub fn list_all(&self) -> &[EntryMetadata] {
        &self.stash.entries
    }

    /// Get the total number of entries
    pub fn entry_count(&self) -> usize {
        self.stash.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.stash.is_empty()
    }

    /// Get the total size of all entries in bytes
    pub fn total_size(&self) -> u64 {
        self.stash.total_size_bytes
    }

    /// Get the path to the index file
    pub fn index_path(&self) -> &Path {
        &self.stash_file
    }

    /// Clear all entries and save
    pub fn clear(&mut self) -> Result<()> {
        self.stash = Index::new(self.stash.name.clone());
        self.save_packages()
    }

    /// Update the index name and save
    pub fn set_name(&mut self, name: Option<String>) -> Result<()> {
        self.stash.name = name;
        self.stash.touch();
        self.save_packages()
    }

    /// Check if an entry with the given UUID exists
    pub fn contains(&self, uuid: &Uuid) -> bool {
        self.stash.get_metadata(uuid).is_some()
    }

    /// Get entries sorted by creation date (newest first)
    pub fn entries_by_date(&self) -> Vec<&EntryMetadata> {
        let mut entries: Vec<_> = self.stash.entries.iter().collect();
        entries.sort_by(|a, b| b.created.cmp(&a.created));
        entries
    }

    /// Get entries sorted by size (largest first)
    pub fn entries_by_size(&self) -> Vec<&EntryMetadata> {
        let mut entries: Vec<_> = self.stash.entries.iter().collect();
        entries.sort_by(|a, b| b.total_size_bytes.cmp(&a.total_size_bytes));
        entries
    }

    /// Get entries sorted by name
    pub fn entries_by_name(&self) -> Vec<&EntryMetadata> {
        let mut entries: Vec<_> = self.stash.entries.iter().collect();
        entries.sort_by(|a, b| {
            match (&a.name, &b.name) {
                (Some(name_a), Some(name_b)) => name_a.cmp(name_b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        entries
    }

    /// Update an existing entry's metadata and save
    pub fn update_entry_metadata(
        &mut self,
        uuid: &Uuid,
        name: Option<String>,
        size_delta: i64,
        item_count_delta: isize,
    ) -> Result<()> {
        if let Some(entry) = self.stash.entries.iter_mut().find(|e| &e.uuid == uuid) {
            entry.name = name;

            // Update size
            if size_delta >= 0 {
                entry.total_size_bytes += size_delta as u64;
                self.stash.total_size_bytes += size_delta as u64;
            } else {
                let decrease = (-size_delta) as u64;
                entry.total_size_bytes = entry.total_size_bytes.saturating_sub(decrease);
                self.stash.total_size_bytes = self.stash.total_size_bytes.saturating_sub(decrease);
            }

            // Update item count
            if item_count_delta >= 0 {
                entry.item_count += item_count_delta as usize;
            } else {
                entry.item_count = entry.item_count.saturating_sub((-item_count_delta) as usize);
            }

            self.stash.touch();
            self.save_packages()?;
            Ok(())
        } else {
            Err(anyhow!("Entry with UUID {} not found", uuid))
        }
    }
}
