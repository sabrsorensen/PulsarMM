use crate::path_ops::{downloads_target_from_root, library_target_from_root, validate_move_target};
use crate::utils::path_changes::{
    move_directory_contents, update_downloads_path_in_config, update_library_path_in_config,
};
use std::fs;
use std::path::{Path, PathBuf};

pub fn apply_downloads_path_change(
    old_path: &Path,
    user_selected_root: &Path,
    config_path: &Path,
) -> Result<PathBuf, String> {
    let target_path = downloads_target_from_root(user_selected_root);
    validate_move_target(old_path, &target_path)?;

    if !target_path.exists() {
        fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
    }

    if old_path.exists() {
        move_directory_contents(old_path, &target_path)?;
    }

    update_downloads_path_in_config(config_path, &target_path)?;
    Ok(target_path)
}

pub fn apply_library_path_change(
    old_path: &Path,
    user_selected_root: &Path,
    config_path: &Path,
) -> Result<PathBuf, String> {
    let target_path = library_target_from_root(user_selected_root);
    validate_move_target(old_path, &target_path)?;

    if !target_path.exists() {
        fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
    }

    if old_path.exists() {
        move_directory_contents(old_path, &target_path)?;
    }

    update_library_path_in_config(config_path, &target_path)?;
    Ok(target_path)
}

#[cfg(test)]
#[path = "mutations_tests.rs"]
mod tests;
