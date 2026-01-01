use anyhow::{Result, anyhow};
use crate::operations::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

pub fn run(identifier: &Option<String>, force: &bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dirs = AppDirs::new();
    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let entry = if let Some(ident) = identifier {
        entry_manager.load_entry_by_identifier(ident)?
    } else {
        let meta = entry_manager.most_recent_entry()
            .ok_or_else(|| anyhow!("No stashed entries found"))?;
        entry_manager.load_entry(&meta.uuid)?
    };

    entry_manager.peek_entry(&entry.uuid, &cwd, *force)?;

    println!(
        "Peeked {} file(s) from '{}'",
        entry.items.len(),
        entry.name
    );

    Ok(())
}
