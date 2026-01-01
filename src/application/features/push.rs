use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::operations::entry_manager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

use crate::{operations::entry_manager::EntryManager, services::storage::ConfigStorage};

pub fn run(
    items: &Vec<PathBuf>,
    name: &Option<String>,
    clone: &bool,
    link: &bool
) -> Result<()>{
    let cwd = std::env::current_dir()?;

    let dirs = AppDirs::new();

    ConfigStorage::new(&dirs.config_file);

    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;

    let mut entry_manager = EntryManager::new(&dirs.entries_dir, &mut index_storage, &mut journal_storage)?;

    let options = entry_manager::PushOptions {
        name: name,
        copy: clone,
        link: link,
    };

    entry_manager.create_entry(items, options, &cwd)?;

    Ok(())
}
