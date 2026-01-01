use clap::Parser;
use std::path::PathBuf;

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
    /// Clean old entries
    Clean(i64),
    /// Rename entry
    Rename {
        old: String,
        new: String,
    },
    /// Export to tar
    Tar(PathBuf),
    /// Initialize
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
        .args(&["push", "pop", "peek", "delete", "list", "search",
                "info", "history", "init", "clean", "rename", "tar", "dump"])
        .required(false)
))]
pub struct Cli {
    pub items: Vec<PathBuf>,

    #[arg(short, long)]
    pub name: Option<String>,

    // Operation flags
    #[arg(short = 'p', long)]
    pub push: bool,

    #[arg(short = 'P', long)]
    pub pop: bool,

    #[arg(long)]
    pub peek: bool,

    #[arg(short = 'd', long)]
    pub delete: bool,

    #[arg(short = 'L', long)]
    pub list: bool,

    #[arg(short, long)]
    pub search: Option<String>,

    #[arg(short, long)]
    pub info: bool,

    #[arg(long)]
    pub history: bool,

    #[arg(long)]
    pub init: bool,

    #[arg(long, value_name = "DAYS", default_missing_value = "30")]
    pub clean: Option<i64>,

    #[arg(long, value_name = "OLD:NEW")]
    pub rename: Option<String>,

    #[arg(long, value_name = "FILE")]
    pub tar: Option<PathBuf>,

    #[arg(long)]
    pub dump: bool,

    // Modifiers (not operations themselves)
    #[arg(short, long, help = "Copy instead of move")]
    pub copy: bool,

    #[arg(short = 'l', long, conflicts_with = "copy")]
    pub link: bool,

    #[arg(short, long, help = "Overwrite existing files")]
    pub force: bool,

    #[arg(long, help = "Restore original paths (use with --pop)")]
    pub restore: bool,
}
