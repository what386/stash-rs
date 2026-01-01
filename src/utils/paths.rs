use anyhow::Result;
use dirs;
use std::{fs, path::PathBuf};

pub struct AppDirs {
    pub user_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub entries_dir: PathBuf,
    pub index_file: PathBuf,
    pub journal_file: PathBuf,
    pub config_file: PathBuf,
}

impl AppDirs {
    pub fn new() -> Self {
        let user_dir = dirs::home_dir().unwrap();
        let config_dir = dirs::config_dir().unwrap().join("stash");

        let data_dir = user_dir.join(".stash");
        let entries_dir = data_dir.join("entries");

        let index_file = data_dir.join("index.json");
        let journal_file = data_dir.join("journal.log");
        let config_file = config_dir.join("config.toml");

        Self {
            user_dir,
            config_dir,
            data_dir,
            entries_dir,
            index_file,
            journal_file,
            config_file
        }
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)?;
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.entries_dir)?;

        Ok(())
    }
}
