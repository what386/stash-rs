use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "stash")]
#[command(about = "A CLI tool for stashing files and folders")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Move files/folders to stash
    Push {
        names: Vec<PathBuf>,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Copy files/folders to stash
    Copy {
        names: Vec<PathBuf>,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Move items to CWD (top item if no name given)
    Pop { names: Option<Vec<String>> },
    /// Copy items to CWD (top item if no name given)
    Peek { names: Option<Vec<String>> },
    /// Undo last operation
    Undo,
    /// Return file, optionally delete
    Drop {
        name: String,
        #[arg(long)]
        delete: bool,
    },
    /// Return all files, optionally delete
    Dump {
        #[arg(long)]
        delete: bool,
    },
    /// Show/clean old items
    Clean {
        #[arg(long)]
        days: Option<i64>,
    },
    /// Show all stashed items
    #[command(alias = "ls")]
    List,
    /// Find items by pattern
    Search { pattern: String },
    /// Get metadata for item, or entire stash
    Info { name: Option<String> },
    /// Rename stashed item
    Rename { old: String, new: String },
    /// Show operation history
    History,
    /// Export to tar/tar.gz
    Tar {
        output: PathBuf,
        #[arg(long)]
        compress: bool,
    },
    /// Initialize stash from tar/tar.gz
    Import { path: PathBuf },
}
