use anyhow::Result;
use crate::services::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;
use crate::utils::display::{humanize_duration, humanize_size};

pub fn run() -> Result<()> {
    let dirs = AppDirs::new();
    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let entries = entry_manager.list_entries();

    if entries.is_empty() {
        println!("No stashed entries.");
        return Ok(());
    }

    println!("Stashed entries:");
    for (i, meta) in entries.iter().enumerate() {

        let age = humanize_duration(meta.created);
        let size = humanize_size(meta.total_size_bytes);

        println!(
            "{}. {} ({} files, {}, {})",
            i + 1,
            meta.name,
            meta.item_count,
            size,
            age
        );
    }

    Ok(())
}
