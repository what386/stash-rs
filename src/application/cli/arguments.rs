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
    pub items: Vec<PathBuf>,

    // External
    #[arg(long)]
    pub init: bool,

    // Commands
    #[arg(short, long)]
    pub name: Option<String>,

    #[arg(short, long)]
    pub search: Option<String>,

    #[arg(short, long)]
    pub list: bool,

    #[arg(short, long)]
    pub info: bool,

    #[arg(long, value_name = "DAYS", default_missing_value = "30")]
    pub clean: Option<i64>,

    #[arg(long, value_name = "OLD:NEW", alias = "rn")]
    pub rename: Option<String>,

    #[arg(long, value_name = "FILE")]
    pub tar: Option<PathBuf>,

    #[arg(long)]
    pub dump: bool,

    #[arg(long)]
    pub history: bool,

    // Modifiers
    #[arg(short, long)]
    pub copy: bool,

    #[arg(short, long)]
    pub force: bool,

    #[arg(short, long)]
    pub restore: bool,
}
