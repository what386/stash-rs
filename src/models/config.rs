use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Policy for handling name conflicts in the stash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictPolicy {
    /// Abort the operation if a conflict exists
    Abort,
    /// Automatically rename the stashed item to avoid conflict
    Rename,
    /// Overwrite existing stashed item
    Overwrite,
    /// Prompt the user interactively
    Prompt,
}

/// Compression level for stash entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// No compression
    None,
    /// Fastest, minimal compression
    Fast,
    /// Balanced speed/size
    Medium,
    /// Maximum compression (slower)
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Persistent settings
    pub stash_dir: PathBuf,

    // Defaults section
    pub clean_days: u64,  // Renamed from clean_after_days
    pub warn_size_mb: u64,
    pub ambiguity_mode: AmbiguityMode,  // New enum

    // Behavior section
    pub preserve_mtime: bool,
    pub verify_integrity: bool,
    pub follow_symlinks: bool,

    // Display section
    pub date_format: String,
    pub show_sizes: bool,

    // Future features
    pub compress_entries: bool,
    pub compression_level: CompressionLevel,

    // REMOVE: dry_run and verbose (these are CLI flags, not config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbiguityMode {
    Ask,
    PreferPush,
    PreferPop,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stash_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".stash"),
            clean_days: 30,
            warn_size_mb: 100,
            ambiguity_mode: AmbiguityMode::Ask,
            preserve_mtime: true,
            verify_integrity: true,
            follow_symlinks: false,
            date_format: "%Y-%m-%d %H:%M".to_string(),
            show_sizes: true,
            compress_entries: false,
            compression_level: CompressionLevel::Medium,
        }
    }
}

