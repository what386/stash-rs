
use anyhow::Result;
use crate::application::cli::arguments::{Cli, OperationMode};
use crate::application::cli::inference;
use crate::application::features;
use crate::utils::paths::AppDirs;

impl Cli {
    pub fn run(self) -> Result<()> {
        let operation = inference::infer_operation(&self)?;

        match operation {
            OperationMode::Push { items, name, copy, link } => {
                features::push::run(&items, &name, &copy, &link)
            }

            OperationMode::Pop { identifier, copy, force, restore } => {
                features::pop::run(&identifier, &copy, &force, &restore)
            }

            OperationMode::Peek { identifier, force } => {
                features::peek::run(&identifier, &force)
            }

            OperationMode::Delete { identifier } => {
                features::delete::run(&identifier)
            }

            OperationMode::Dump { delete } => {
                features::dump::run(delete)
            }

            OperationMode::List => {
                features::list::run()
            }

            OperationMode::Search(pattern) => {
                features::search::run(&pattern)
            }

            OperationMode::Info { identifier } => {
                features::info::run(&identifier)
            }

            OperationMode::History => {
                features::history::run()
            }

            OperationMode::Undo => {
                //features::undo::run()
                // TODO: implement
                Ok(())
            }

            OperationMode::Clean(days) => {
                features::clean::run(days)
            }

            OperationMode::Rename { old, new } => {
                features::rename::run(&old, &new)
            }

            OperationMode::Tar(path) => {
                features::tar::run(&path)
            }

            OperationMode::Init => {
                AppDirs::new().init()
            }
        }
    }
}
