use anyhow::Result;
use crate::operations::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

pub fn run(identifier: &str) -> Result<()> {
    let dirs = AppDirs::new();
    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let mut entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let entry = entry_manager.load_entry_by_identifier(identifier)?;

    entry_manager.delete_entry(&entry.uuid)?;

    println!("Deleted entry '{}' ({} files)", entry.name, entry.items.len());

    Ok(())
}
