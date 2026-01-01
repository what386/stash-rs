use anyhow::{Result, anyhow};
use std::path::PathBuf;
use crate::operations::entry_manager::EntryManager;
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;
use crate::services::filesystem::tape_archives;

pub fn run(output_path: &PathBuf) -> Result<()> {
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
        return Err(anyhow!("No entries to export"));
    }

    // Create a temporary directory for collecting all entries
    let temp_dir = std::env::temp_dir().join(format!("stash-export-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;

    println!("Exporting {} entries to {}...\n", entries.len(), output_path.display());

    // Copy all entries into temp directory
    for meta in entries {
        let entry = entry_manager.load_entry(&meta.uuid)?;

        let entry_dir = dirs.entries_dir.join(meta.uuid.to_string());
        let dest_dir = temp_dir.join(&entry.name);

        // Copy the entire entry directory (including manifest and data)
        copy_dir_all(&entry_dir, &dest_dir)?;

        println!("  • {}", entry.name);
    }

    // Create tar archive from temp directory
    tape_archives::create_tar(&temp_dir, output_path)?;

    // Cleanup temp directory
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\nExported {} entries to {}", entries.len(), output_path.display());

    Ok(())
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

