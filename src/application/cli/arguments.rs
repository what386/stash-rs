use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "stash")]
#[command(about = "A CLI tool for stashing files and folders", version)]
#[command(long_about = "Intelligently stash and restore files based on context.\n\n\
    EXAMPLES:\n  \
    stash file.txt          # Stash if exists, restore if in stash\n  \
    stash                   # Restore most recent entry\n  \
    stash --name work src/  # Stash with custom name\n  \
    stash --list            # Show all entries")]
pub struct Cli {
    /// Files/directories or entry names to operate on
    pub items: Vec<PathBuf>,

    /// Name for this stash entry
    #[arg(short, long)]
    pub name: Option<String>,

    /// Explicit push (stash files/directories)
    #[arg(short = 'p', long, conflicts_with_all = &["pop", "peek"])]
    pub push: bool,

    /// Explicit pop (restore and remove from stash)
    #[arg(short = 'P', long, conflicts_with_all = &["push", "peek"])]
    pub pop: bool,

    /// Copy instead of move
    #[arg(short, long, conflicts_with = "link")]
    pub copy: bool,

    /// Create symlinks to stashed files (leave symlinks in place after stashing)
    #[arg(short, long, conflicts_with = "copy")]
    pub link: bool,

    /// Copy out without removing from stash
    #[arg(long, conflicts_with_all = &["push", "copy", "link"])]
    pub peek: bool,

    /// Force overwrite existing files
    #[arg(short, long)]
    pub force: bool,

    /// Delete instead of restore
    #[arg(short = 'd', long)]
    pub delete: bool,

    /// List all stashed entries
    #[arg(short = 'l', long)]
    pub list: bool,

    /// Search entries by pattern
    #[arg(short, long)]
    pub search: Option<String>,

    /// Show detailed info about entry
    #[arg(short, long)]
    pub info: bool,

    /// Show operation history
    #[arg(long)]
    pub history: bool,

    /// Undo last operation
    #[arg(short, long)]
    pub undo: bool,

    /// Clean entries older than N days (default: 30)
    #[arg(long)]
    pub clean: Option<Option<i64>>,

    /// Restore entry to its original location
    #[arg(long)]
    pub restore: bool,

    /// Rename an entry (format: old:new)
    #[arg(long, value_name = "OLD:NEW")]
    pub rename: Option<String>,

    /// Export stash to tar archive
    #[arg(long, value_name = "FILE")]
    pub tar: Option<PathBuf>,

    /// Dump all entries (restore or delete all)
    #[arg(long)]
    pub dump: bool,
}

#[derive(Debug, Clone)]
pub enum OperationMode {
    /// Force push files
    Push {
        items: Vec<PathBuf>,
        name: Option<String>,
        copy: bool,
        link: bool,
    },
    /// Force pop entry by identifier
    Pop {
        identifier: Option<String>,
        copy: bool,
        force: bool,
        restore: bool,
    },
    /// Copy out without removing
    Peek {
        identifier: Option<String>,
        force: bool,
    },
    /// Delete entry without restoring
    Delete {
        identifier: String,
    },
    /// Dump all entries
    Dump {
        delete: bool,
    },
    /// List all entries
    List,
    /// Search by pattern
    Search(String),
    /// Show info about entry or stash
    Info {
        identifier: Option<String>,
    },
    /// Show history
    History,
    /// Undo last operation
    Undo,
    /// Clean old entries
    Clean(i64),
    /// Rename entry
    Rename {
        old: String,
        new: String,
    },
    /// Export to tar
    Tar(PathBuf),
}
