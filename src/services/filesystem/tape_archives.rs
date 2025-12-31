use anyhow::Result;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Unpack a TAR archive into the output folder
pub fn unpack_tar(input: &Path, output: &Path) -> Result<PathBuf> {
    let file = File::open(input)?;
    let mut archive = Archive::new(file);

    // Configure archive to work cross-platform
    archive.set_preserve_permissions(cfg!(unix));
    archive.set_preserve_mtime(true);
    archive.set_unpack_xattrs(false);

    let mut paths = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = output.join(entry.path()?);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        entry.unpack(&path)?;
        paths.push(path);
    }

    Ok(common_root(&paths, output))
}

/// Create a TAR archive from a source directory or file
pub fn create_tar(source: &Path, output: &Path) -> Result<()> {
    let file = File::create(output)?;
    let mut archive = tar::Builder::new(file);

    // Configure builder to work cross-platform
    #[cfg(unix)]
    archive.mode(tar::HeaderMode::Deterministic);

    if source.is_dir() {
        archive.append_dir_all(".", source)?;
    } else {
        let file_name = source.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid source file name"))?;
        archive.append_path_with_name(source, file_name)?;
    }

    archive.finish()?;
    Ok(())
}

/// Determine the common root of extracted paths
fn common_root(paths: &[PathBuf], output: &Path) -> PathBuf {
    if paths.is_empty() {
        return output.to_path_buf();
    }

    let first = paths[0].strip_prefix(output).unwrap();
    let mut components: Vec<_> = first.components().collect();

    for path in &paths[1..] {
        let path_comps: Vec<_> = path.strip_prefix(output).unwrap().components().collect();
        components.truncate(
            components.iter()
                .zip(&path_comps)
                .take_while(|(a, b)| a == b)
                .count()
        );
    }

    output.join(components.iter().fold(PathBuf::new(), |acc, c| acc.join(c)))
}
