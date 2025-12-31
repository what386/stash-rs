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

    /// Copy instead of move when stashing
    #[arg(short, long, conflicts_with = "link")]
    pub copy: bool,

    /// Symlink to stashed copy instead of moving
    #[arg(short, long, conflicts_with = "copy")]
    pub link: bool,

    /// Copy out without removing from stash
    #[arg(long, conflicts_with_all = &["push", "copy", "link"])]
    pub peek: bool,

    /// Delete files instead of restoring
    #[arg(short, long)]
    pub delete: bool,

    /// Allow symlinking directories (normally disabled)
    #[arg(long, requires = "link")]
    pub allow_dirs: bool,

    /// List all stashed entries
    #[arg(short = 'l', long)]
    pub list: bool,

    /// Search entries by pattern
    #[arg(short, long)]
    pub search: Option<String>,

    /// Show detailed info about entry or entire stash
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

    /// Rename an entry
    #[arg(short, long, value_name = "OLD:NEW")]
    pub rename: Option<String>,

    /// Export stash to tar archive
    #[arg(long, value_name = "FILE")]
    pub tar: Option<PathBuf>,

    /// Import stash from tar archive
    #[arg(long, value_name = "FILE")]
    pub import: Option<PathBuf>,

    /// Restore entry to its original location
    #[arg(long)]
    pub restore: bool,

    /// Dump all entries (restore or delete all)
    #[arg(long)]
    pub dump: bool,
}

#[derive(Debug, Clone)]
pub enum OperationMode {
    /// Infer from context (magic mode)
    Infer,
    /// Force push files
    Push { copy: bool, link: bool },
    /// Force pop entry
    Pop,
    /// Pop most recent entry
    PopRecent,
    /// Copy out without removing
    Peek,
    /// Restore to original location
    Restore { delete: bool },
    /// Remove all entries
    Dump { delete: bool },
    /// List all entries
    List,
    /// Search by pattern
    Search(String),
    /// Show info
    Info,
    /// Show history
    History,
    /// Undo last operation
    Undo,
    /// Clean old entries
    Clean(i64),
    /// Rename entry
    Rename { old: String, new: String },
    /// Export to tar
    Tar(PathBuf),
    /// Import from tar
    Import(PathBuf),
}
