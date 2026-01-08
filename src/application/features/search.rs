use anyhow::Result;
use crate::services::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;
use crate::utils::display::{humanize_duration, humanize_size};

pub fn run(pattern: &str) -> Result<()> {
    let dirs = AppDirs::new();
    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage,
    )?;

    let entries = entry_manager.list_entries();
    let pattern_lower = pattern.to_lowercase();

    let matches: Vec<_> = entries.iter()
        .filter(|e| e.name.to_lowercase().contains(&pattern_lower))
        .collect();

    if matches.is_empty() {
        println!("No entries match '{}'.", pattern);
        return Ok(());
    }

    println!("Found {} match{}:", matches.len(), if matches.len() == 1 { "" } else { "es" });

    for meta in matches {
        let age = humanize_duration(meta.created);
        let size = humanize_size(meta.total_size_bytes);

        println!("  • {} ({} files, {}, {})", meta.name, meta.item_count, size, age);
    }

    Ok(())
}
