use anyhow::{bail, Result};

use crate::application::cli::arguments::{Cli, OperationMode};

pub fn infer_operation(cli: &Cli) -> Result<OperationMode> {
    if cli.list {
        return Ok(OperationMode::List);
    }

    if let Some(pattern) = &cli.search {
        return Ok(OperationMode::Search(pattern.clone()));
    }

    if cli.info {
        return Ok(OperationMode::Info);
    }

    if cli.history {
        return Ok(OperationMode::History);
    }

    if cli.undo {
        return Ok(OperationMode::Undo);
    }

    if let Some(days) = cli.clean {
        return Ok(OperationMode::Clean(days.unwrap_or(30)));
    }

    if let Some(spec) = &cli.rename {
        let (old, new) = spec
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("rename must be in OLD:NEW format"))?;
        return Ok(OperationMode::Rename {
            old: old.into(),
            new: new.into(),
        });
    }

    if let Some(path) = &cli.tar {
        return Ok(OperationMode::Tar(path.clone()));
    }

    if let Some(path) = &cli.import {
        return Ok(OperationMode::Import(path.clone()));
    }

    if cli.dump {
        return Ok(OperationMode::Dump {
            delete: cli.delete,
        });
    }

    /* ---------- Explicit operations ---------- */

    if cli.push {
        return Ok(OperationMode::Push {
            copy: cli.copy,
            link: cli.link,
        });
    }

    if cli.pop {
        return Ok(OperationMode::Pop);
    }

    if cli.peek {
        return Ok(OperationMode::Peek);
    }

    if cli.restore {
        return Ok(OperationMode::Restore {
            delete: cli.delete,
        });
    }

    /* ---------- Inference mode ---------- */

    infer_from_context(cli)
}

fn infer_from_context(cli: &Cli) -> Result<OperationMode> {
    let items = &cli.items;

    // No args → pop most recent
    if items.is_empty() {
        return Ok(OperationMode::PopRecent);
    }

    let (existing, missing): (Vec<_>, Vec<_>) =
        items.iter().partition(|p| p.exists());

    match (existing.is_empty(), missing.is_empty()) {
        // All exist → push
        (false, true) => Ok(OperationMode::Push {
            copy: cli.copy,
            link: cli.link,
        }),

        // All missing → pop by name
        (true, false) => Ok(OperationMode::Pop),

        // Mixed → ambiguous
        (false, false) => bail!(
            "cannot infer operation: some paths exist and some do not"
        ),

        // Defensive (should be unreachable)
        (true, true) => bail!("no valid items provided"),
    }
}

