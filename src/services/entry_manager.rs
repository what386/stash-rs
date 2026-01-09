use anyhow::{Result, Context, anyhow};
use chrono::{Utc, DateTime};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use std::io::Read;
use crate::models::{Operation, OperationKind};
use crate::models::entry::Entry;
use crate::models::item::{Item, ItemKind};
use crate::services::storage::index_storage::IndexStorage;
use crate::services::storage::journal_storage::JournalStorage;
use crate::services::filesystem::permission_handler;

pub struct EntryManager<'a> {
    entries_root: &'a PathBuf,
    index_storage: &'a mut IndexStorage,
    journal_storage: &'a mut JournalStorage,
}

pub struct PushOptions<'a> {
    pub name: &'a String,
    pub copy: &'a bool,
}

pub struct PopOptions<'a> {
    pub destination: &'a PathBuf,
    pub copy: &'a bool,
    pub force: &'a bool,
}

impl<'a> EntryManager<'a> {
    pub fn new(
        entries_root: &'a PathBuf,
        index_storage: &'a mut IndexStorage,
        journal_storage: &'a mut JournalStorage,
    ) -> Result<Self> {
        fs::create_dir_all(entries_root)?;
        Ok(Self {
            entries_root,
            index_storage,
            journal_storage,
        })
    }

    pub fn create_entry(
        &mut self,
        paths: &Vec<PathBuf>,
        options: PushOptions,
        working_directory: &Path,
    ) -> Result<Entry> {
        if paths.is_empty() {
            return Err(anyhow!("No paths provided"));
        }

        let mut items = Vec::new();
        let mut total_size = 0u64;

        for path in paths {
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("Failed to read {:?}", path))?;

            let kind = if metadata.is_dir() {
                ItemKind::Directory
            } else if metadata.file_type().is_symlink() {
                ItemKind::Symlink
            } else {
                ItemKind::File
            };

            // Calculate actual size including directory contents
            let size = self.calculate_size(path)?;
            total_size += size;

            // Preserve original modified time
            let modified = metadata.modified()
                .ok()
                .and_then(|t| DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0
                ))
                .unwrap_or_else(Utc::now);

            // Calculate hash for files
            let hash = if metadata.is_file() {
                Some(self.calculate_hash(path)?)
            } else {
                None
            };

            items.push(Item {
                original_path: path.clone(),
                stashed_path: path.clone(),
                kind,
                size_bytes: size,
                permissions: permission_handler::get_permissions(path)?,
                modified,
                hash,
            });
        }

        let entry = Entry::new(
            options.name.clone(),
            items,
            working_directory.to_path_buf(),
            !options.copy,
        );

        let entry_dir = self.entry_dir(&entry.uuid);
        let data_dir = entry_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        // Move/copy files to stash
        for item in &entry.items {
            let src = &item.original_path;
            let dest = data_dir.join(&item.stashed_path);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            if *options.copy {
                // Copy mode: leave originals in place
                self.copy_recursively(src, &dest)?;
            } else {
                // Move mode: relocate to stash
                self.move_recursively(src, &dest)?;
            }

            // Preserve timestamps
            self.preserve_timestamps(src, &dest)?;
        }

        self.write_manifest(&entry)?;

        self.index_storage.add_entry(
            entry.uuid,
            entry.name.clone(),
            total_size,
            entry.items.len(),
        )?;

        // Log the operation (don't log copy operations for undo purposes)
        if !*options.copy {
            let kind = OperationKind::Push {
                entry_id: entry.uuid,
                file_count: entry.items.len(),
            };
            self.journal_storage.append(Operation::new(kind))?;
        }

        Ok(entry)
    }

    pub fn pop_entry(
        &mut self,
        uuid: &Uuid,
        options: PopOptions,
    ) -> Result<Entry> {
        let entry = self.load_entry(uuid)?;
        let data_dir = self.entry_dir(uuid).join("data");

        for item in &entry.items {
            let src = data_dir.join(&item.stashed_path);
            let dest = options.destination.join(&item.stashed_path);

            // Check for existing files
            if dest.exists() && !options.force {
                return Err(anyhow!(
                    "Destination {:?} already exists. Use --force to overwrite.",
                    dest
                ));
            }

            // Ensure parent directories exist
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy or move the item
            if *options.copy {
                self.copy_recursively(&src, &dest)?;
            } else {
                self.move_recursively(&src, &dest)?;
            }

            // Restore permissions
            permission_handler::set_permissions(&dest, item.permissions)?;

            // Restore timestamps
            self.restore_timestamps(&dest, item.modified)?;
        }

        // Remove entry from stash if not copying
        if !*options.copy {
            self.delete_entry_internal(uuid)?;
        }

        self.journal_storage.append(Operation::new(
            OperationKind::Pop {
                entry_id: *uuid,
                destination: options.destination.clone(),
            }
        ))?;

        Ok(entry)
    }

    /// Peek: copy files out without removing from stash
    pub fn peek_entry(
        &self,
        uuid: &Uuid,
        destination: &Path,
        force: bool,
    ) -> Result<Entry> {
        let entry = self.load_entry(uuid)?;
        let data_dir = self.entry_dir(uuid).join("data");

        for item in &entry.items {
            let src = data_dir.join(&item.stashed_path);
            let dest = destination.join(&item.stashed_path);

            if dest.exists() && !force {
                return Err(anyhow!(
                    "Destination {:?} already exists. Use --force to overwrite.",
                    dest
                ));
            }

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            self.copy_recursively(&src, &dest)?;
            permission_handler::set_permissions(&dest, item.permissions)?;
            self.restore_timestamps(&dest, item.modified)?;
        }

        // Note: peek doesn't modify the stash or journal
        Ok(entry)
    }

    /// Restore to original working directory
    pub fn restore_entry(
        &mut self,
        uuid: &Uuid,
        force: bool,
    ) -> Result<Entry> {
        let entry = self.load_entry(uuid)?;
        let original_dir = entry.working_directory.clone();

        self.pop_entry(uuid, PopOptions {
            destination: &original_dir,
            copy: &false,
            force: &force,
        })
    }

    pub fn rename_entry(&mut self, uuid: &Uuid, new_name: String) -> Result<()> {
        let entry = self.load_entry(uuid)?;
        let old_name = entry.name.clone();

        self.write_manifest(&entry)?;
        self.index_storage.update_entry_name(uuid, new_name.clone())?;

        self.journal_storage.append(Operation::new(
            OperationKind::Rename {
                entry_id: *uuid,
                old_name,
                new_name,
            }
        ))?;

        Ok(())
    }

    pub fn delete_entry(&mut self, uuid: &Uuid) -> Result<()> {
        self.delete_entry_internal(uuid)?;

        self.journal_storage.append(Operation::new(
            OperationKind::Drop {
                entry_id: *uuid,
                deleted: true,
            }
        ))?;

        Ok(())
    }

    fn delete_entry_internal(&mut self, uuid: &Uuid) -> Result<()> {
        let entry_dir = self.entry_dir(uuid);
        fs::remove_dir_all(&entry_dir)
            .with_context(|| format!("Failed to remove {:?}", entry_dir))?;
        self.index_storage.remove_entry(uuid)?;
        Ok(())
    }

    pub fn clean_old_entries(&mut self, days: i64) -> Result<Vec<Uuid>> {
        let removed = self.index_storage.remove_older_than_days(days)?;

        for uuid in &removed {
            let dir = self.entry_dir(uuid);
            let _ = fs::remove_dir_all(dir);
        }

        self.journal_storage.append(Operation::new(
            OperationKind::Clean {
                removed_count: removed.len(),
                days,
            }
        ))?;

        Ok(removed)
    }

    pub fn load_entry(&self, uuid: &Uuid) -> Result<Entry> {
        let manifest = self.entry_dir(uuid).join("manifest.json");
        let json = fs::read_to_string(&manifest)
            .with_context(|| format!("Failed to read {:?}", manifest))?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn load_entry_by_identifier(&self, ident: &str) -> Result<Entry> {
        let meta = self.index_storage
            .find_by_identifier(ident)
            .ok_or_else(|| anyhow!("Entry not found: {}", ident))?;
        self.load_entry(&meta.uuid)
    }

    pub fn list_entries(&self) -> &[crate::models::index::EntryMetadata] {
        self.index_storage.list_all()
    }

    pub fn most_recent_entry(&self) -> Option<&crate::models::index::EntryMetadata> {
        self.index_storage.most_recent()
    }

    pub fn find_entries_containing_path(
        &self,
        path: &Path,
    ) -> Result<Vec<Uuid>> {
        let mut matches = Vec::new();
        for meta in self.index_storage.list_all() {
            let entry = self.load_entry(&meta.uuid)?;
            if entry.get_item(path).is_some() {
                matches.push(meta.uuid);
            }
        }
        Ok(matches)
    }

    fn write_manifest(&self, entry: &Entry) -> Result<()> {
        let path = self.entry_dir(&entry.uuid).join("manifest.json");
        let json = serde_json::to_string_pretty(entry)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn entry_dir(&self, uuid: &Uuid) -> PathBuf {
        self.entries_root.join(uuid.to_string())
    }

    /// Calculate total size including directory contents
    fn calculate_size(&self, path: &Path) -> Result<u64> {
        let metadata = fs::symlink_metadata(path)?;

        if metadata.is_file() {
            Ok(metadata.len())
        } else if metadata.is_dir() {
            let mut total = 0u64;
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                total += self.calculate_size(&entry.path())?;
            }
            Ok(total)
        } else {
            Ok(0) // Symlinks
        }
    }

    /// Calculate SHA256 hash of a file
    fn calculate_hash(&self, path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("sha256:{:x}", hasher.finalize()))
    }

    /// Copy files/directories recursively
    fn copy_recursively(&self, src: &Path, dest: &Path) -> Result<()> {
        let metadata = fs::symlink_metadata(src)?;

        if metadata.is_dir() {
            fs::create_dir_all(dest)?;
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let src_path = entry.path();
                let dest_path = dest.join(entry.file_name());
                self.copy_recursively(&src_path, &dest_path)?;
            }
        } else if metadata.file_type().is_symlink() {
            #[cfg(unix)]
            {
                let target = fs::read_link(src)?;
                std::os::unix::fs::symlink(target, dest)?;
            }
            #[cfg(windows)]
            {
                fs::copy(src, dest)?;
            }
        } else {
            fs::copy(src, dest)?;
        }

        Ok(())
    }

    /// Move files/directories recursively
    fn move_recursively(&self, src: &Path, dest: &Path) -> Result<()> {
        // Try simple rename first (works if on same filesystem)
        if fs::rename(src, dest).is_ok() {
            return Ok(());
        }

        // Fall back to copy + delete for cross-filesystem moves
        self.copy_recursively(src, dest)?;

        if src.is_dir() {
            fs::remove_dir_all(src)?;
        } else {
            fs::remove_file(src)?;
        }

        Ok(())
    }

    /// Preserve timestamps from source to destination
    fn preserve_timestamps(&self, src: &Path, dest: &Path) -> Result<()> {
        if let Ok(metadata) = fs::metadata(src) {
            if let (Ok(accessed), Ok(modified)) = (metadata.accessed(), metadata.modified()) {
                let _ = filetime::set_file_times(
                    dest,
                    filetime::FileTime::from_system_time(accessed),
                    filetime::FileTime::from_system_time(modified),
                );
            }
        }
        Ok(())
    }

    /// Restore specific timestamp to a file
    fn restore_timestamps(&self, path: &Path, modified: DateTime<chrono::Utc>) -> Result<()> {
        let mtime = filetime::FileTime::from_unix_time(modified.timestamp(), 0);
        let _ = filetime::set_file_mtime(path, mtime);
        Ok(())
    }
}
