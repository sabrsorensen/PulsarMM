use std::fs;
use std::path::{Path, PathBuf};

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

pub fn linux_show_in_folder_target(path: &Path) -> PathBuf {
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| path.to_path_buf())
}

pub fn library_folder_name(zip_filename: &str) -> String {
    format!("{}_unpacked", zip_filename)
}

pub fn clear_files_in_dir(path: &Path) -> Result<usize, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut removed = 0usize;
    let entries = fs::read_dir(path).map_err(io_error_to_string)?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() {
            fs::remove_file(entry_path).map_err(io_error_to_string)?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn clear_dirs_in_dir(path: &Path) -> Result<usize, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut removed = 0usize;
    let entries = fs::read_dir(path).map_err(io_error_to_string)?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            fs::remove_dir_all(entry_path).map_err(io_error_to_string)?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn clean_staging_dir(staging_dir: &Path) -> Result<usize, String> {
    if !staging_dir.exists() {
        return Ok(0);
    }

    let count = fs::read_dir(staging_dir)
        .map_err(io_error_to_string)?
        .count();
    if count == 0 {
        return Ok(0);
    }

    fs::remove_dir_all(staging_dir).map_err(io_error_to_string)?;
    fs::create_dir_all(staging_dir).map_err(io_error_to_string)?;
    Ok(count)
}

pub fn select_special_folder_path(
    folder_type: &str,
    downloads_path: PathBuf,
    profiles_path: PathBuf,
    library_path: PathBuf,
) -> Result<PathBuf, String> {
    match folder_type {
        "downloads" => Ok(downloads_path),
        "profiles" => Ok(profiles_path),
        "library" => Ok(library_path),
        _ => Err("Unknown folder type".to_string()),
    }
}

#[cfg(test)]
#[path = "logic_tests.rs"]
mod tests;
