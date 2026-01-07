use anyhow::Result;
use crate::services::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;

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

    println!("Found {} match{}:\n", matches.len(), if matches.len() == 1 { "" } else { "es" });

    for meta in matches {
        let age = humanize_duration(meta.created);
        let size = humanize_size(meta.total_size_bytes);

        println!("  • {} ({} files, {}, {})", meta.name, meta.item_count, size, age);
    }

    Ok(())
}

// Helper functions (reused)
fn humanize_duration(created: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(created);

    if duration.num_days() > 0 {
        let days = duration.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if duration.num_hours() > 0 {
        let hours = duration.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if duration.num_minutes() > 0 {
        let minutes = duration.num_minutes();
        format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
    } else {
        "just now".to_string()
    }
}

fn humanize_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
