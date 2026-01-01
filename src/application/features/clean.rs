use anyhow::Result;
use crate::operations::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

pub fn run(days: i64) -> Result<()> {
    let dirs = AppDirs::new();
    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let mut entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let removed = entry_manager.clean_old_entries(days)?;

    if removed.is_empty() {
        println!("No entries older than {} days.", days);
    } else {
        println!("✓ Cleaned {} entries older than {} days.", removed.len(), days);
    }

    Ok(())
}
