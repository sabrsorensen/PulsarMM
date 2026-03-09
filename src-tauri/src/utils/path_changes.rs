use crate::fs_ops::copy_dir_recursive;
use crate::utils::config::{load_config_or_default, save_config};
use std::fs;
use std::path::Path;

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

pub fn move_directory_contents(old_path: &Path, target_path: &Path) -> Result<(), String> {
    if !target_path.exists() {
        fs::create_dir_all(target_path).map_err(io_error_to_string)?;
    }

    if old_path.exists() {
        copy_dir_recursive(old_path, target_path)?;
        fs::remove_dir_all(old_path).map_err(io_error_to_string)?;
    }

    Ok(())
}

pub fn update_downloads_path_in_config(
    config_path: &Path,
    target_path: &Path,
) -> Result<(), String> {
    let mut config = load_config_or_default(config_path, true);
    config.custom_download_path = Some(target_path.to_string_lossy().into_owned());
    save_config(config_path, &config)
}

pub fn update_library_path_in_config(config_path: &Path, target_path: &Path) -> Result<(), String> {
    let mut config = load_config_or_default(config_path, true);
    config.custom_library_path = Some(target_path.to_string_lossy().into_owned());
    save_config(config_path, &config)
}

#[cfg(test)]
#[path = "path_changes_tests.rs"]
mod tests;
