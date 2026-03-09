use crate::mods::command_ops::mods_root_from_game_path;
use crate::mods::tracking::has_untracked_mods_in_dir;
use std::fs;
use std::path::Path;

pub fn save_text_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content)
        .map_err(|e| format!("Failed to write to file '{}': {}", path.display(), e))
}

pub fn delete_settings_at_path(settings_file: &Path) -> Result<String, String> {
    if !settings_file.exists() {
        return Ok("alertDeleteNotFound".to_string());
    }

    fs::remove_file(settings_file).map_err(|e| {
        format!(
            "Failed to delete file at '{}': {}",
            settings_file.display(),
            e
        )
    })?;
    Ok("alertDeleteSuccess".to_string())
}

pub fn delete_settings_without_game_path() -> Result<String, String> {
    Err("alertDeleteError".to_string())
}

pub fn ensure_mods_dir_exists(game_path: &Path) -> Result<std::path::PathBuf, String> {
    let mods_path = mods_root_from_game_path(game_path);
    fs::create_dir_all(&mods_path).map_err(|e| {
        format!(
            "Could not create MODS folder at '{}': {}",
            mods_path.display(),
            e
        )
    })?;
    Ok(mods_path)
}

pub fn check_untracked_mods_for_game_path(game_path: Option<&Path>) -> bool {
    match game_path {
        Some(path) => has_untracked_mods_in_dir(&mods_root_from_game_path(path)),
        None => false,
    }
}

pub fn open_mods_folder_for_game_path(
    game_path: Option<&Path>,
    open_path: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let game_path = game_path.ok_or_else(|| "Game path not found.".to_string())?;
    let mods_path = ensure_mods_dir_exists(game_path)?;
    open_path(&mods_path).map_err(|e| {
        format!(
            "Could not open MODS folder at '{}': {}",
            mods_path.display(),
            e
        )
    })
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
