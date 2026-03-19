use crate::mods::archive::extract_archive;
use crate::mods::install_archive_flow::{
    copy_archive_to_downloads, library_folder_name_for_archive, scan_library_mod_path,
};
use std::path::{Path, PathBuf};
use url::Url;

pub struct InstallArchiveContext {
    pub final_archive_path_str: String,
    pub library_id: String,
    pub library_mod_path: PathBuf,
}

pub fn resolve_conflict_with(
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    new_mod_name: &str,
    old_mod_folder_name: &str,
    temp_mod_path_str: &str,
    replace: bool,
    resolve_conflict_in_paths: impl FnOnce(&Path, &str, &str, &Path, bool) -> Result<(), String>,
) -> Result<(), String> {
    let game_path = find_game_path().ok_or_else(|| "Could not find game path.".to_string())?;
    let mods_path = game_path.join("GAMEDATA").join("MODS");
    let temp_mod_path = PathBuf::from(temp_mod_path_str);
    resolve_conflict_in_paths(
        &mods_path,
        new_mod_name,
        old_mod_folder_name,
        &temp_mod_path,
        replace,
    )
}

pub fn archive_path_from_input(archive_path_str: &str) -> PathBuf {
    if let Ok(url) = Url::parse(archive_path_str) {
        if url.scheme() == "file" {
            if let Ok(path) = url.to_file_path() {
                return path;
            }
        }
    }

    PathBuf::from(archive_path_str)
}

pub fn build_install_archive_context(
    final_archive_path: &Path,
    library_dir: &Path,
) -> Result<InstallArchiveContext, String> {
    let final_archive_path_str = final_archive_path.to_string_lossy().into_owned();
    let library_id = library_folder_name_for_archive(final_archive_path)?;
    let library_mod_path = library_dir.join(&library_id);

    Ok(InstallArchiveContext {
        final_archive_path_str,
        library_id,
        library_mod_path,
    })
}

pub fn copy_archive_to_downloads_blocking(
    archive_path: PathBuf,
    downloads_dir: PathBuf,
) -> Result<PathBuf, String> {
    let (final_archive_path, _) = copy_archive_to_downloads(&archive_path, &downloads_dir)?;
    Ok(final_archive_path)
}

pub fn extract_archive_if_needed(
    archive_path: PathBuf,
    library_mod_path: PathBuf,
    progress_callback: &mut dyn FnMut(u64),
) -> Result<(), String> {
    if library_mod_path.exists() {
        return Ok(());
    }
    extract_archive(&archive_path, &library_mod_path, progress_callback)
}

pub fn scan_library_mod_path_blocking(
    library_mod_path: PathBuf,
) -> Result<(Vec<String>, Vec<String>), String> {
    scan_library_mod_path(&library_mod_path)
}

#[cfg(test)]
#[path = "install_command_core_conflict_tests.rs"]
mod install_command_core_conflict_tests;

#[cfg(test)]
#[path = "install_command_core_context_tests.rs"]
mod install_command_core_context_tests;

#[cfg(test)]
#[path = "install_command_core_archive_tests.rs"]
mod install_command_core_archive_tests;
