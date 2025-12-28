use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub original_path: PathBuf,
    pub stashed_path: PathBuf,
    pub kind: ItemKind,
    pub size_bytes: u64,
    pub permissions: u32,
    pub modified: DateTime<Utc>,
    pub hash: Option<String>,
}

impl Item {
    pub fn new(
        original_path: PathBuf,
        stashed_path: PathBuf,
        kind: ItemKind,
        size_bytes: u64,
        permissions: u32,
        modified: DateTime<Utc>,
        hash: Option<String>,
    ) -> Self {
        Self {
            original_path,
            stashed_path,
            kind,
            size_bytes,
            permissions,
            modified,
            hash,
        }
    }

    pub fn matches_pattern(&self, pattern: &str) -> bool {
        self.original_path
            .to_string_lossy()
            .to_lowercase()
            .contains(&pattern.to_lowercase())
    }
}
