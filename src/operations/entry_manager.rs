use anyhow::{Result, Context, anyhow};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use dircpy;

use crate::models::{Operation, OperationKind};
use crate::models::entry::Entry;
use crate::models::item::{Item, ItemKind};
use crate::services::storage::index_storage::IndexStorage;
use crate::services::storage::journal_storage::JournalStorage;
use crate::services::filesystem::permission_handler;

pub struct EntryManager {
    stash_root: PathBuf,
    entries_dir: PathBuf,
    index: IndexStorage,
    journal: JournalStorage,
}

pub struct PushOptions {
    pub name: Option<String>,
    pub copy: bool,
    pub link: bool,
}

pub struct PopOptions {
    pub destination: PathBuf,
    pub delete_after: bool,
    pub force: bool,
}

impl EntryManager {
    pub fn new(
        stash_root: PathBuf,
        index: IndexStorage,
        journal: JournalStorage,
    ) -> Result<Self> {
        let entries_dir = stash_root.join("entries");
        fs::create_dir_all(&entries_dir)?;

        Ok(Self {
            stash_root,
            entries_dir,
            index,
            journal,
        })
    }

    pub fn create_entry(
        &mut self,
        paths: Vec<PathBuf>,
        options: PushOptions,
        working_directory: &Path,
    ) -> Result<Entry> {
        if paths.is_empty() {
            return Err(anyhow!("No paths provided"));
        }

        let mut items = Vec::new();

        for path in &paths {
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("Failed to read {:?}", path))?;

            let kind = if metadata.is_dir() {
                ItemKind::Directory
            } else if metadata.file_type().is_symlink() {
                ItemKind::Symlink
            } else {
                ItemKind::File
            };

            let size = if metadata.is_file() {
                metadata.len()
            } else {
                0
            };

            items.push(Item {
                original_path: path.clone(),
                stashed_path: path.clone(),
                kind,
                size_bytes: size,
                permissions: permission_handler::get_permissions(path)?,
                modified: Utc::now(),
                hash: None,
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

        for item in &entry.items {
            let src = &item.original_path;
            let dest = data_dir.join(&item.stashed_path);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            if options.link {
                #[cfg(unix)]
                std::os::unix::fs::symlink(src, &dest)?;
                #[cfg(windows)]
                return Err(anyhow!("Symlink-in-place is unsupported on Windows"));
            } else if options.copy {
                dircpy::copy_dir(src, &dest)?;
            } else {
                fs::rename(src, &dest)?;
            }
        }

        self.write_manifest(&entry)?;

        self.index.add_entry(
            entry.uuid,
            entry.name.clone(),
            entry.total_size_bytes,
            entry.items.len(),
        )?;

        let kind = if options.copy {
            OperationKind::Copy {
                entry_id: entry.uuid,
                file_count: entry.items.len(),
            }
        } else {
            OperationKind::Push {
                entry_id: entry.uuid,
                file_count: entry.items.len(),
            }
        };

        self.journal.append(Operation::new(kind))?;

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
            let dest = options.destination.join(&item.original_path);

            if dest.exists() && !options.force {
                return Err(anyhow!(
                    "Destination {:?} already exists",
                    dest
                ));
            }

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::rename(&src, &dest)?;
            permission_handler::set_permissions(&dest, item.permissions)?;
        }

        if options.delete_after {
            self.delete_entry(uuid)?;
        }

        self.journal.append(Operation::new(
            OperationKind::Pop {
                entry_id: *uuid,
                destination: options.destination.clone(),
            }
        ))?;

        Ok(entry)
    }

    pub fn delete_entry(&mut self, uuid: &Uuid) -> Result<()> {
        let entry_dir = self.entry_dir(uuid);

        fs::remove_dir_all(&entry_dir)
            .with_context(|| format!("Failed to remove {:?}", entry_dir))?;

        self.index.remove_entry(uuid)?;

        self.journal.append(Operation::new(
            OperationKind::Drop {
                entry_id: *uuid,
                deleted: true,
            }
        ))?;

        Ok(())
    }

    pub fn clean_old_entries(&mut self, days: i64) -> Result<Vec<Uuid>> {
        let removed = self.index.remove_older_than_days(days)?;

        for uuid in &removed {
            let dir = self.entry_dir(uuid);
            let _ = fs::remove_dir_all(dir);
        }

        self.journal.append(Operation::new(
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
        let meta = self.index
            .find_by_identifier(ident)
            .ok_or_else(|| anyhow!("Entry not found: {}", ident))?;

        self.load_entry(&meta.uuid)
    }

    pub fn list_entries(&self) -> &[crate::models::index::EntryMetadata] {
        self.index.list_all()
    }

    pub fn most_recent_entry(&self) -> Option<&crate::models::index::EntryMetadata> {
        self.index.most_recent()
    }

    pub fn find_entries_containing_path(
        &self,
        path: &Path,
    ) -> Result<Vec<Uuid>> {
        let mut matches = Vec::new();

        for meta in self.index.list_all() {
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
        self.entries_dir.join(uuid.to_string())
    }
}

