use std::fs;
use std::path::{Path, PathBuf};

pub fn mods_root_from_game_path(game_path: &Path) -> PathBuf {
    game_path.join("GAMEDATA").join("MODS")
}

pub fn settings_file_from_game_path(game_path: &Path) -> PathBuf {
    crate::settings_paths::mod_settings_file(game_path)
}

pub fn mod_folder_path(game_path: &Path, mod_name: &str) -> PathBuf {
    mods_root_from_game_path(game_path).join(mod_name)
}

pub fn library_rename_paths(
    library_dir: &Path,
    source_zip: &str,
    old_name: &str,
    new_name: &str,
) -> (PathBuf, PathBuf) {
    let unpacked = library_dir.join(format!("{}_unpacked", source_zip));
    (unpacked.join(old_name), unpacked.join(new_name))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryRenameSync {
    SourceMissing,
    TargetExists,
    Renamed,
}

pub fn sync_library_folder_rename(
    library_dir: &Path,
    source_zip: &str,
    old_name: &str,
    new_name: &str,
) -> Result<LibraryRenameSync, String> {
    let (lib_old_path, lib_new_path) =
        library_rename_paths(library_dir, source_zip, old_name, new_name);
    if !lib_old_path.exists() {
        return Ok(LibraryRenameSync::SourceMissing);
    }
    if lib_new_path.exists() {
        return Ok(LibraryRenameSync::TargetExists);
    }

    fs::rename(&lib_old_path, &lib_new_path)
        .map_err(|e| format!("Failed to sync rename to Library: {}", e))?;
    Ok(LibraryRenameSync::Renamed)
}

pub fn validate_rename_paths(old_exists: bool, new_exists: bool) -> Result<(), String> {
    if !old_exists {
        Err("Original mod folder not found.".to_string())
    } else if new_exists {
        Err("A mod with the new name already exists.".to_string())
    } else {
        Ok(())
    }
}

pub fn maybe_remove_mod_folder(path: &Path, mod_name: &str) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(path)
        .map_err(|e| format!("Failed to delete mod folder '{}': {}", mod_name, e))?;
    Ok(true)
}

#[cfg(test)]
#[path = "command_ops_tests.rs"]
mod tests;
