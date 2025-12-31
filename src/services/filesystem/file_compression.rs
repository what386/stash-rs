use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression as BzCompression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression as GzCompression;
use tar::Archive;
use crate::services::filesystem::tape_archives::{create_tar, unpack_tar};

/// Compression level - algorithm is chosen automatically based on level
#[derive(Debug, Clone, Copy)]
pub enum CompressionLevel {
    Fast,     // gzip fast
    Medium,   // gzip default
    Maximum,  // gzip best
    Extreme,  // bzip2 best
}

impl CompressionLevel {
    fn is_bzip2(&self) -> bool {
        matches!(self, CompressionLevel::Extreme)
    }

    fn extension(&self) -> &'static str {
        if self.is_bzip2() { "tar.bz2" } else { "tar.gz" }
    }

    fn gzip(&self) -> GzCompression {
        match self {
            CompressionLevel::Fast => GzCompression::fast(),
            CompressionLevel::Medium => GzCompression::default(),
            _ => GzCompression::best(),
        }
    }

    fn bzip2(&self) -> BzCompression {
        match self {
            CompressionLevel::Fast => BzCompression::fast(),
            CompressionLevel::Extreme => BzCompression::best(),
            _ => BzCompression::default(),
        }
    }
}

/// Decompress a file into the output folder and return the root path extracted
pub fn decompress(input: &Path, output: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(output)?;

    let name = input.file_name().unwrap().to_string_lossy().to_lowercase();

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        decompress_tar_wrapped(input, output, GzDecoder::new)
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz") || name.ends_with(".tbz2") {
        decompress_tar_wrapped(input, output, BzDecoder::new)
    } else if name.ends_with(".tar") {
        unpack_tar(input, output)
    } else {
        match input.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
            "gz" => decompress_single(input, output, GzDecoder::new),
            "bz2" => decompress_single(input, output, BzDecoder::new),
            ext => Err(anyhow!("Unsupported archive format: {}", ext)),
        }
    }
}

/// Compress a source with specified compression level
/// Returns the actual output path with correct extension
pub fn compress(source: &Path, output: &Path, level: CompressionLevel) -> Result<PathBuf> {
    let output_path = output.with_extension(level.extension());

    if level.is_bzip2() {
        compress_tar_wrapped(source, &output_path, level, |f, l| BzEncoder::new(f, l.bzip2()))?;
    } else {
        compress_tar_wrapped(source, &output_path, level, |f, l| GzEncoder::new(f, l.gzip()))?;
    }

    Ok(output_path)
}

fn decompress_tar_wrapped<R, F>(input: &Path, output: &Path, wrapper: F) -> Result<PathBuf>
where
    R: Read,
    F: FnOnce(File) -> R,
{
    let file = File::open(input)?;
    let mut archive = Archive::new(wrapper(file));

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

fn decompress_single<R, F>(input: &Path, output: &Path, wrapper: F) -> Result<PathBuf>
where
    R: Read,
    F: FnOnce(File) -> R,
{
    let file = File::open(input)?;
    let mut decoder = wrapper(file);

    let output_name = input.file_stem().ok_or_else(|| anyhow!("Invalid file name"))?;
    let output_path = output.join(output_name);

    let mut output_file = File::create(&output_path)?;
    std::io::copy(&mut decoder, &mut output_file)?;

    Ok(output_path)
}

fn compress_tar_wrapped<W, F>(source: &Path, output: &Path, level: CompressionLevel, wrapper: F) -> Result<()>
where
    W: Write,
    F: FnOnce(File, CompressionLevel) -> W,
{
    let temp_tar = output.with_extension("tar.tmp");
    create_tar(source, &temp_tar)?;

    let mut tar_file = File::open(&temp_tar)?;
    let output_file = File::create(output)?;
    let mut encoder = wrapper(output_file, level);

    std::io::copy(&mut tar_file, &mut encoder)?;
    encoder.flush()?;

    std::fs::remove_file(&temp_tar)?;
    Ok(())
}

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
