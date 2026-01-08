use std::path::PathBuf;

use anyhow::Result;

use crate::services::entry_manager;
use crate::services::entry_manager::EntryManager;
use crate::services::storage::{ConfigStorage, IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

pub fn run(
    items: &Vec<PathBuf>,
    name: &Option<String>,
    copy: &bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dirs = AppDirs::new();

    // Ensure config exists
    ConfigStorage::new(&dirs.config_file)?;

    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;

    let mut entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let default_name = items[0]
        .file_name()
        .expect("item must have filename")
        .to_string_lossy()
        .to_string();

    let options = entry_manager::PushOptions {
        name: name.as_ref().unwrap_or(&default_name),
        copy,
    };

    entry_manager.create_entry(items, options, &cwd)?;

    Ok(())
}

