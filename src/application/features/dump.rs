use anyhow::Result;

use crate::services::entry_manager::{EntryManager, PopOptions};
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dirs = AppDirs::new();

    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;

    let mut entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let entries: Vec<_> = entry_manager
        .list_entries()
        .iter()
        .map(|m| m.uuid)
        .collect();

    if entries.is_empty() {
        println!("No entries to dump.");
        return Ok(());
    }

    println!("Restoring {} entries...\n", entries.len());

    for uuid in entries {
        let entry = entry_manager.load_entry(&uuid)?;

        let options = PopOptions {
            destination: &cwd,
            copy: &false,
            force: &true,
        };

        entry_manager.pop_entry(&uuid, options)?;
        println!("  Restored: {}", entry.name);
    }

    println!("\nDump complete.");

    Ok(())
}

