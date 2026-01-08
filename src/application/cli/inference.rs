use anyhow::{Result, bail};
use crate::application::cli::arguments::{Cli, OperationMode};
use std::path::PathBuf;

pub fn infer_operation(cli: &Cli) -> Result<OperationMode> {
    // ========================================================================
    // Priority 1: Explicit, non-inferable operations
    // ========================================================================

    if cli.init {
        return Ok(OperationMode::Init);
    }

    if cli.list {
        return Ok(OperationMode::List);
    }

    if let Some(pattern) = &cli.search {
        return Ok(OperationMode::Search(pattern.clone()));
    }

    if cli.info {
        let identifier = cli
            .items
            .first()
            .map(|p| p.to_string_lossy().to_string());

        return Ok(OperationMode::Info { identifier });
    }

    if cli.history {
        return Ok(OperationMode::History);
    }

    if let Some(days) = cli.clean {
        return Ok(OperationMode::Clean(days));
    }

    if let Some(spec) = &cli.rename {
        let (old, new) = spec
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("--rename must be in OLD:NEW format"))?;

        return Ok(OperationMode::Rename {
            old: old.into(),
            new: new.into(),
        });
    }

    if let Some(path) = &cli.tar {
        return Ok(OperationMode::Tar(path.clone()));
    }

    if cli.dump {
        return Ok(OperationMode::Dump);
    }

    // ========================================================================
    // Priority 2: Context-based inference
    // ========================================================================

    infer_from_context(cli)
}

fn infer_from_context(cli: &Cli) -> Result<OperationMode> {
    let items = &cli.items;

    // ------------------------------------------------------------------------
    // Rule 1: No arguments → pop most recent entry
    // ------------------------------------------------------------------------
    if items.is_empty() {
        return Ok(OperationMode::Pop {
            identifier: None,
            copy: cli.copy,
            force: cli.force,
            restore: cli.restore,
        });
    }

    // ------------------------------------------------------------------------
    // Rule 2: All items exist locally → push
    // ------------------------------------------------------------------------
    let all_exist = items.iter().all(|p| p.exists());
    if all_exist {
        return Ok(OperationMode::Push {
            items: items.clone(),
            name: cli.name.clone(),
            copy: cli.copy,
        });
    }

    // ------------------------------------------------------------------------
    // Rule 3: None exist locally → treat as stash identifier
    // ------------------------------------------------------------------------
    let none_exist = items.iter().all(|p| !p.exists());
    if none_exist {
        if items.len() == 1 {
            return Ok(OperationMode::Pop {
                identifier: Some(items[0].to_string_lossy().to_string()),
                copy: cli.copy,
                force: cli.force,
                restore: cli.restore,
            });
        }

        bail!(
            "Cannot restore multiple entries at once: {}\n\
             Entries are referenced by a single name or ID.\n\
             Use --list to see available entries.",
            format_paths(items)
        );
    }

    // ------------------------------------------------------------------------
    // Rule 4: Mixed existence → ambiguous
    // ------------------------------------------------------------------------
    let existing: Vec<_> = items.iter().filter(|p| p.exists()).collect();
    let missing: Vec<_> = items.iter().filter(|p| !p.exists()).collect();

    bail!(
        "Ambiguous operation:\n\
         - These paths exist locally: {}\n\
         - These paths do not exist: {}\n\n\
         Stash infers actions from context.\n\
         Try separating the operations or using --copy / --force / --restore.",
        format_paths_refs(&existing),
        format_paths_refs(&missing)
    );
}

fn format_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| format!("'{}'", p.display()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_paths_refs(paths: &[&PathBuf]) -> String {
    paths
        .iter()
        .map(|p| format!("'{}'", p.display()))
        .collect::<Vec<_>>()
        .join(", ")
}

