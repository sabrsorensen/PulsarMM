use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;
#[path = "archive_rar_runtime.rs"]
mod rar_runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveKind {
    Zip,
    Rar,
    SevenZ,
}

fn io_error_to_string(error: io::Error) -> String {
    error.to_string()
}

fn zip_error_to_string(error: zip::result::ZipError) -> String {
    error.to_string()
}

fn ensure_destination_exists(destination: &Path) -> Result<(), String> {
    if !destination.exists() {
        fs::create_dir_all(destination).map_err(|e| format!("Could not create dest dir: {}", e))?;
    }
    Ok(())
}

fn canonical_archive_path(archive_path: &Path) -> Result<PathBuf, String> {
    archive_path
        .canonicalize()
        .map_err(|e| format!("Invalid archive path '{}': {}", archive_path.display(), e))
}

fn detect_archive_kind(archive_path: &Path) -> Result<ArchiveKind, String> {
    let extension = archive_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "zip" => Ok(ArchiveKind::Zip),
        "rar" => Ok(ArchiveKind::Rar),
        "7z" => Ok(ArchiveKind::SevenZ),
        _ => Err(format!("Unsupported file type: .{}", extension)),
    }
}

fn ensure_parent_dir_exists(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(io_error_to_string)?;
        }
    }
    Ok(())
}

fn extract_zip_archive(
    abs_archive_path: &Path,
    destination: &Path,
    on_progress: &mut dyn FnMut(u64),
) -> Result<(), String> {
    let file = fs::File::open(abs_archive_path).map_err(io_error_to_string)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error_to_string)?;

    let total_files = archive.len();
    for i in 0..total_files {
        let mut file = archive.by_index(i).map_err(zip_error_to_string)?;

        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(io_error_to_string)?;
        } else {
            ensure_parent_dir_exists(&outpath)?;
            let mut outfile = fs::File::create(&outpath).map_err(io_error_to_string)?;
            io::copy(&mut file, &mut outfile).map_err(io_error_to_string)?;
        }

        let pct = ((i as u64 + 1) * 100) / total_files as u64;
        on_progress(pct);
    }

    Ok(())
}

fn extract_7z_archive(
    abs_archive_path: &Path,
    destination: &Path,
    on_progress: &mut dyn FnMut(u64),
) -> Result<(), String> {
    on_progress(50);
    sevenz_rust::decompress_file(abs_archive_path, destination).map_err(|e| e.to_string())?;
    on_progress(100);
    Ok(())
}

pub fn extract_archive(
    archive_path: &Path,
    destination: &Path,
    on_progress: &mut dyn FnMut(u64),
) -> Result<(), String> {
    ensure_destination_exists(destination)?;
    let abs_archive_path = canonical_archive_path(archive_path)?;

    match detect_archive_kind(archive_path)? {
        ArchiveKind::Zip => extract_zip_archive(&abs_archive_path, destination, on_progress)?,
        ArchiveKind::Rar => {
            rar_runtime::extract_rar_archive(&abs_archive_path, destination, on_progress)?
        }
        ArchiveKind::SevenZ => extract_7z_archive(&abs_archive_path, destination, on_progress)?,
    }

    Ok(())
}

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
