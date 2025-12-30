use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationKind {
    Push { entry_id: Uuid, file_count: usize },
    Copy { entry_id: Uuid, file_count: usize },
    Pop { entry_id: Uuid, destination: PathBuf },
    Peek { entry_id: Uuid, destination: PathBuf },
    Drop { entry_id: Uuid, deleted: bool },
    Dump { entry_count: usize, deleted: bool },
    Rename { entry_id: Uuid, old_name: String, new_name: String },
    Clean { removed_count: usize, days: i64 },
    Import { path: PathBuf, entry_count: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub kind: OperationKind,
    pub timestamp: DateTime<Utc>,
}

impl Operation {
    pub fn new(kind: OperationKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            timestamp: Utc::now(),
        }
    }

    pub fn describe(&self) -> String {
        match &self.kind {
            OperationKind::Push { entry_id, file_count } => {
                format!("Pushed {} file(s) to entry {}", file_count, short_uuid(entry_id))
            }
            OperationKind::Copy { entry_id, file_count } => {
                format!("Copied {} file(s) to entry {}", file_count, short_uuid(entry_id))
            }
            OperationKind::Pop { entry_id, destination } => {
                format!("Popped entry {} to {}", short_uuid(entry_id), destination.display())
            }
            OperationKind::Peek { entry_id, destination } => {
                format!("Peeked entry {} to {}", short_uuid(entry_id), destination.display())
            }
            OperationKind::Drop { entry_id, deleted } => {
                if *deleted {
                    format!("Dropped and deleted entry {}", short_uuid(entry_id))
                } else {
                    format!("Dropped entry {} to disk", short_uuid(entry_id))
                }
            }
            OperationKind::Dump { entry_count, deleted } => {
                if *deleted {
                    format!("Dumped and deleted {} entries", entry_count)
                } else {
                    format!("Dumped {} entries to disk", entry_count)
                }
            }
            OperationKind::Rename { entry_id, old_name, new_name } => {
                format!("Renamed entry {} from '{}' to '{}'", short_uuid(entry_id), old_name, new_name)
            }
            OperationKind::Clean { removed_count, days } => {
                format!("Cleaned {} entries older than {} days", removed_count, days)
            }
            OperationKind::Import { path, entry_count } => {
                format!("Imported {} entries from {}", entry_count, path.display())
            }
        }
    }

    pub fn is_undoable(&self) -> bool {
        matches!(
            self.kind,
            OperationKind::Push { .. }
                | OperationKind::Pop { .. }
                | OperationKind::Drop { deleted: false, .. }
                | OperationKind::Rename { .. }
        )
    }


    pub fn involves_entry(&self, entry_id: &Uuid) -> bool {
        self.entry_id() == Some(*entry_id)
    }

    pub fn entry_id(&self) -> Option<Uuid> {
        match &self.kind {
            OperationKind::Push { entry_id, .. }
            | OperationKind::Copy { entry_id, .. }
            | OperationKind::Pop { entry_id, .. }
            | OperationKind::Peek { entry_id, .. }
            | OperationKind::Drop { entry_id, .. }
            | OperationKind::Rename { entry_id, .. } => Some(*entry_id),
            _ => None,
        }
    }
}

fn short_uuid(uuid: &Uuid) -> String {
    uuid.to_string()[..6].to_string()
}
