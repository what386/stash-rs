use anyhow::{Result, bail};
use crate::application::cli::arguments::{Cli, OperationMode};
use std::path::PathBuf;

pub fn infer_operation(cli: &Cli) -> Result<OperationMode> {
    // ========================================================================
    // Priority 1: Information and management flags
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
        let identifier = cli.items.first().map(|p| p.to_string_lossy().to_string());
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
        return Ok(OperationMode::Dump {
            delete: cli.delete,
        });
    }

    // ========================================================================
    // Priority 2: Explicit operations (--push, --pop, --peek)
    // ========================================================================

    if cli.push {
        if cli.items.is_empty() {
            bail!("--push requires at least one file or directory");
        }
        return Ok(OperationMode::Push {
            items: cli.items.clone(),
            name: cli.name.clone(),
            copy: cli.copy,
            link: cli.link,
        });
    }

    if cli.pop {
        // Pop takes optional identifier, not file paths
        let identifier = if !cli.items.is_empty() {
            Some(cli.items[0].to_string_lossy().to_string())
        } else {
            cli.name.clone()
        };

        return Ok(OperationMode::Pop {
            identifier,
            copy: cli.copy,
            force: cli.force,
            restore: cli.restore,
        });
    }

    if cli.peek {
        let identifier = if !cli.items.is_empty() {
            Some(cli.items[0].to_string_lossy().to_string())
        } else {
            cli.name.clone()
        };

        return Ok(OperationMode::Peek {
            identifier,
            force: cli.force,
        });
    }

    // Delete specific entry
    if cli.delete {
        if cli.items.is_empty() {
            bail!("--delete requires an entry name or UUID");
        }
        return Ok(OperationMode::Delete {
            identifier: cli.items[0].to_string_lossy().to_string(),
        });
    }

    // ========================================================================
    // Priority 3: Magic inference based on context
    // ========================================================================

    infer_from_context(cli)
}

fn infer_from_context(cli: &Cli) -> Result<OperationMode> {
    let items = &cli.items;

    // Rule 1: No arguments → Pop most recent entry
    if items.is_empty() {
        return Ok(OperationMode::Pop {
            identifier: None,
            copy: cli.copy,
            force: cli.force,
            restore: cli.restore,
        });
    }

    // Rule 2: All paths exist locally → Stash them
    let all_exist = items.iter().all(|p| p.exists());
    if all_exist {
        return Ok(OperationMode::Push {
            items: items.clone(),
            name: cli.name.clone(),
            copy: cli.copy,
            link: cli.link,
        });
    }

    // Rule 3 & 4: Paths don't exist → Treat as entry name
    // The entry name is either:
    // - Custom name provided via --name when stashing
    // - Single filename (for single-file stashes)
    // - Generated UUID/timestamp (for multi-file stashes without --name)
    let none_exist = items.iter().all(|p| !p.exists());
    if none_exist {
        if items.len() == 1 {
            // Single item: use it as the entry identifier
            return Ok(OperationMode::Pop {
                identifier: Some(items[0].to_string_lossy().to_string()),
                copy: cli.copy,
                force: cli.force,
                restore: cli.restore,
            });
        } else {
            // Multiple non-existent items: ambiguous or error
            bail!(
                "Cannot pop multiple items: {}\n\
                Entries are referenced by a single name.\n\
                Use --list to see available entries.",
                format_paths(&items.iter().collect::<Vec<_>>())
            );
        }
    }

    // Rule 5: Ambiguous (some exist, some don't)
    let existing: Vec<_> = items.iter().filter(|p| p.exists()).collect();
    let missing: Vec<_> = items.iter().filter(|p| !p.exists()).collect();

    bail!(
        "Ambiguous operation:\n\
        - These paths exist locally: {}\n\
        - These paths don't exist: {}\n\n\
        Use --push to stash existing files or --pop to restore an entry.",
        format_paths(&existing),
        format_paths(&missing)
    );
}

fn format_paths(paths: &[&PathBuf]) -> String {
    paths.iter()
        .map(|p| format!("'{}'", p.display()))
        .collect::<Vec<_>>()
        .join(", ")
}
