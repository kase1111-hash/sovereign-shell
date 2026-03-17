//! Archive handling: zip read, extract, and create.

use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};

/// An entry inside a zip archive.
#[derive(Debug, Clone, Serialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub compressed_size: u64,
}

/// List the contents of a zip archive.
pub fn list_zip(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("Cannot open archive: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip archive: {e}"))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i)
            .map_err(|e| format!("Zip entry error: {e}"))?;

        entries.push(ArchiveEntry {
            name: entry.name().rsplit('/').next().unwrap_or(entry.name()).to_string(),
            path: entry.name().to_string(),
            is_dir: entry.is_dir(),
            size: entry.size(),
            compressed_size: entry.compressed_size(),
        });
    }

    Ok(entries)
}

/// Extract a zip archive to a destination directory.
pub fn extract_zip(archive_path: &str, dest_dir: &str) -> Result<usize, String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("Cannot open archive: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip archive: {e}"))?;

    let dest = PathBuf::from(dest_dir);
    std::fs::create_dir_all(&dest)
        .map_err(|e| format!("Cannot create destination: {e}"))?;

    let mut count = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Zip entry error: {e}"))?;

        let out_path = dest.join(entry.name());

        // Prevent zip-slip by ensuring path stays within dest
        if !out_path.starts_with(&dest) {
            continue;
        }

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("Create dir error: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Create parent dir error: {e}"))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| format!("Create file error: {e}"))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("Extract error: {e}"))?;
            count += 1;
        }
    }

    Ok(count)
}

/// Create a zip archive from a list of file paths.
pub fn create_zip(sources: &[String], archive_path: &str) -> Result<usize, String> {
    let file = std::fs::File::create(archive_path)
        .map_err(|e| format!("Cannot create archive: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut count = 0;
    for source in sources {
        let src = PathBuf::from(source);
        if src.is_dir() {
            count += add_dir_to_zip(&mut zip, &src, &src, options)?;
        } else {
            let name = src.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            zip.start_file(&name, options)
                .map_err(|e| format!("Zip start error: {e}"))?;
            let mut f = std::fs::File::open(&src)
                .map_err(|e| format!("Read error: {e}"))?;
            std::io::copy(&mut f, &mut zip)
                .map_err(|e| format!("Zip write error: {e}"))?;
            count += 1;
        }
    }

    zip.finish().map_err(|e| format!("Zip finish error: {e}"))?;
    Ok(count)
}

/// Recursively add a directory to a zip archive.
fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<usize, String> {
    let mut count = 0;

    for entry in std::fs::read_dir(dir).map_err(|e| format!("Read dir error: {e}"))? {
        let entry = entry.map_err(|e| format!("Dir entry error: {e}"))?;
        let path = entry.path();
        let relative = path.strip_prefix(base)
            .map_err(|e| format!("Strip prefix error: {e}"))?
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            zip.add_directory(&format!("{}/", relative), options)
                .map_err(|e| format!("Zip dir error: {e}"))?;
            count += add_dir_to_zip(zip, base, &path, options)?;
        } else {
            zip.start_file(&relative, options)
                .map_err(|e| format!("Zip start error: {e}"))?;
            let mut f = std::fs::File::open(&path)
                .map_err(|e| format!("Read error: {e}"))?;
            std::io::copy(&mut f, zip)
                .map_err(|e| format!("Zip write error: {e}"))?;
            count += 1;
        }
    }

    Ok(count)
}
