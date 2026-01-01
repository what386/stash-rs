use std::path::PathBuf;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use crate::operations::entry_manager::{EntryManager, PopOptions};
use crate::services::storage::{IndexStorage, JournalStorage};
use crate::utils::paths::AppDirs;
use crate::services::storage::ConfigStorage;

pub fn run(
    identifier: &Option<String>,
    copy: &bool,
    force: &bool,
    restore: &bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dirs = AppDirs::new();

    let _config = ConfigStorage::new(&dirs.config_file);
    let mut index_storage = IndexStorage::new(&dirs.index_file)?;
    let mut journal_storage = JournalStorage::new(&dirs.journal_file)?;
    let mut entry_manager = EntryManager::new(
        &dirs.entries_dir,
        &mut index_storage,
        &mut journal_storage
    )?;

    // Determine which entry to pop
    let uuid = if let Some(ident) = identifier {
        // Try to parse as UUID first
        if let Ok(parsed_uuid) = Uuid::parse_str(ident) {
            // Verify it exists
            entry_manager.load_entry(&parsed_uuid)?;
            println!("Restoring entry by UUID: {}", parsed_uuid);
            parsed_uuid
        } else {
            // Try to find by name or partial UUID
            let entry = entry_manager.load_entry_by_identifier(ident)?;
            println!("Restoring entry: {}", ident);
            entry.uuid
        }
    } else {
        // No identifier → pop most recent
        let recent = entry_manager.most_recent_entry()
            .ok_or_else(|| anyhow!("No stashed entries found"))?;

        println!("Restoring most recent entry: {}",
            recent.name.as_ref().unwrap_or(&recent.uuid.to_string()));

        recent.uuid
    };

    // Execute the pop operation
    let entry = if *restore {
        // --restore flag: restore to original working directory
        entry_manager.restore_entry(&uuid, *force)?
    } else {
        // Default: restore to current directory
        let options = PopOptions {
            destination: &cwd,
            copy,
            force,
        };
        entry_manager.pop_entry(&uuid, options)?
    };

    // Success message
    let action = if *copy {
        "Copied out"
    } else if *restore {
        "Restored to original location"
    } else {
        "Restored"
    };

    let destination = if *restore {
        format!("to {}", entry.working_directory.display())
    } else {
        "to current directory".to_string()
    };

    println!(
        "✓ {} {} file(s) from '{}' {}",
        action,
        entry.items.len(),
        entry.name.as_ref().unwrap_or(&uuid.to_string()),
        destination
    );

    // Show what was restored (up to 10 files)
    if entry.items.len() <= 10 {
        for item in &entry.items {
            println!("  • {}", item.original_path.display());
        }
    } else {
        println!("  ({} files total)", entry.items.len());
    }

    Ok(())
}
