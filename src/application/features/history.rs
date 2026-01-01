use anyhow::Result;
use crate::services::storage::JournalStorage;
use crate::utils::paths::AppDirs;

pub fn run() -> Result<()> {
    let dirs = AppDirs::new();
    let journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let operations = journal_storage.recent(20)?;

    if operations.is_empty() {
        println!("No operation history.");
        return Ok(());
    }

    println!("Operation history:\n");
    for op in operations {
        let timestamp = op.timestamp.format("%Y-%m-%d %H:%M:%S");
        println!("[{}] {}", timestamp, op.describe());
    }

    Ok(())
}
