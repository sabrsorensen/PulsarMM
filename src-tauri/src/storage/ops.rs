use super::logic::library_folder_name;
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_folder_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        Ok(())
    } else {
        Err("Folder does not exist".to_string())
    }
}

pub fn delete_archive_file_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn library_folder_path(library_dir: &Path, zip_filename: &str) -> PathBuf {
    library_dir.join(library_folder_name(zip_filename))
}

pub fn delete_library_folder_if_exists(
    library_dir: &Path,
    zip_filename: &str,
) -> Result<(), String> {
    let target_path = library_folder_path(library_dir, zip_filename);
    if target_path.exists() {
        fs::remove_dir_all(target_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
