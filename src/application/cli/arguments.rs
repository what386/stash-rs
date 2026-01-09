use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum OperationMode {
    Push {
        items: Vec<PathBuf>,
        name: Option<String>,
        copy: bool,
    },
    Pop {
        identifier: Option<String>,
        copy: bool,
        force: bool,
        restore: bool,
    },
    Dump,
    List,
    Search(String),
    Info {
        identifier: Option<String>,
    },
    History,
    Clean(i64),
    Rename {
        old: String,
        new: String,
    },
    Tar(PathBuf),
    Init,
}


#[derive(Parser)]
#[command(name = "stash")]
#[command(about = "A CLI tool for stashing files and folders", version)]
#[command(long_about = "Intelligently stash and restore files based on context.\n\n\
    EXAMPLES:\n  \
    stash file.txt          # Stash if exists, restore if in stash\n  \
    stash                   # Restore most recent entry\n  \
    stash --name work src/  # Stash with custom name\n  \
    stash --list            # Show all entries")]
#[command(version)]
#[command(group(
    clap::ArgGroup::new("operation")
        .args(&["list", "search", "info", "history", "init", "clean", "rename", "tar", "dump"])
        .required(false)
))]

pub struct Cli {
    /// Files or directories to operate on
    pub items: Vec<PathBuf>,
    /// Initialize stash storage and config
    #[arg(long)]
    pub init: bool,
    /// Assign a custom name to a stash entry
    #[arg(short, long)]
    pub name: Option<String>,
    /// Search stash entries by name or pattern
    #[arg(short, long)]
    pub search: Option<String>,
    /// List all stash entries
    #[arg(short, long)]
    pub list: bool,
    /// Show detailed info about a stash entry
    #[arg(short, long)]
    pub info: bool,
    /// Remove entries older than the given number of days
    #[arg(long, value_name = "DAYS", default_missing_value = "30")]
    pub clean: Option<i64>,
    /// Rename a stash entry (format: OLD:NEW)
    #[arg(long, value_name = "OLD:NEW", alias = "rn")]
    pub rename: Option<String>,
    /// Export all entries to a tar archive
    #[arg(long, value_name = "FILE")]
    pub tar: Option<PathBuf>,
    /// Restore or delete all stash entries
    #[arg(long)]
    pub dump: bool,
    /// Show stash operation history
    #[arg(long)]
    pub history: bool,
    /// Copy files instead of moving them
    #[arg(short, long)]
    pub copy: bool,
    /// Overwrite existing files when restoring
    #[arg(short, long)]
    pub force: bool,
    /// Restore files to their original paths
    #[arg(short, long)]
    pub restore: bool,
}
