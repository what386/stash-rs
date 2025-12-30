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

    pub fn from_path(
        original_path: PathBuf,
        stashed_path: PathBuf,
        calculate_hash: bool,
    ) -> std::io::Result<Self> {
        use std::fs;

        let metadata = fs::metadata(&original_path)?;
        let kind = if metadata.is_dir() {
            ItemKind::Directory
        } else if metadata.is_symlink() {
            ItemKind::Symlink
        } else {
            ItemKind::File
        };

        let size_bytes = metadata.len();
        let permissions = metadata.permissions().mode();
        let modified = metadata.modified()?.into();

        let hash = if calculate_hash && kind == ItemKind::File {
            Some(calculate_file_hash(&original_path)?)
        } else {
            None
        };

        Ok(Self {
            original_path,
            stashed_path,
            kind,
            size_bytes,
            permissions,
            modified,
            hash,
        })
    }
}

fn calculate_file_hash(path: &Path) -> std::io::Result<String> {
    use sha2::{Sha256, Digest};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("sha256:{:x}", hasher.finalize()))
}
