use super::command_flow::open_special_folder_with;
use super::logic::linux_show_in_folder_target;
use crate::models::FileNode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type PathProviderFn<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type EnsureFolderFn<'a> = dyn for<'b> Fn(&'b Path) -> Result<(), String> + 'a;
type OpenFolderFn<'a> = dyn for<'b> FnMut(&'b Path) -> Result<(), String> + 'a;
type ClearDownloadsFn<'a> = dyn for<'b, 'c> Fn(&'b Path, &'c Path) -> Result<(), String> + 'a;
type SetPathFn<'a> =
    dyn for<'b, 'c, 'd> FnMut(&'b Path, &'c str, &'d Path) -> Result<(), String> + 'a;
type CollectNodesFn<'a> =
    dyn for<'b, 'c> Fn(&'b Path, &'c str) -> Result<Vec<FileNode>, String> + 'a;
type OpenSpecialFn<'a> = dyn Fn(PathBuf) -> Result<(), String> + 'a;
type DeleteLibraryFn<'a> = dyn for<'b, 'c> FnMut(&'b Path, &'c str) -> Result<(), String> + 'a;
type LibraryExistenceFn<'a> = dyn for<'b> Fn(&'b Path, Vec<String>) -> HashMap<String, bool> + 'a;
type SpawnOpenFn<'a> = dyn for<'b> FnMut(&'b Path) -> Result<(), String> + 'a;

pub(crate) fn open_folder_path_with(
    path: &Path,
    ensure_folder_exists: &EnsureFolderFn<'_>,
    open_path: &mut OpenFolderFn<'_>,
) -> Result<(), String> {
    ensure_folder_exists(path)?;
    open_path(path)
}

pub(crate) fn get_path_string_with(get_path: &PathProviderFn<'_>) -> Result<String, String> {
    Ok(get_path()?.to_string_lossy().into_owned())
}

fn check_library_existence_with(
    library_dir: &Path,
    filenames: Vec<String>,
    check_library_existence_map: &LibraryExistenceFn<'_>,
) -> HashMap<String, bool> {
    check_library_existence_map(library_dir, filenames)
}

fn delete_library_folder_with(
    library_dir: &Path,
    zip_filename: &str,
    delete_library_folder_if_exists: &mut DeleteLibraryFn<'_>,
) -> Result<(), String> {
    delete_library_folder_if_exists(library_dir, zip_filename)
}

fn clear_downloads_folder_with(
    get_downloads_dir: &PathProviderFn<'_>,
    get_library_dir: &PathProviderFn<'_>,
    clear_downloads_and_library: &ClearDownloadsFn<'_>,
) -> Result<(), String> {
    let downloads_path = get_downloads_dir()?;
    let library_path = get_library_dir()?;
    clear_downloads_and_library(&downloads_path, &library_path)
}

pub(crate) fn clear_downloads_folder_command_with(
    get_downloads_dir: &PathProviderFn<'_>,
    get_library_dir: &PathProviderFn<'_>,
    clear_downloads_and_library: &ClearDownloadsFn<'_>,
) -> Result<(), String> {
    clear_downloads_folder_with(
        get_downloads_dir,
        get_library_dir,
        clear_downloads_and_library,
    )
}

pub(crate) fn set_downloads_path_command_with(
    get_old_path: &PathProviderFn<'_>,
    get_config_path: &PathProviderFn<'_>,
    new_path: &str,
    set_downloads_path: &mut SetPathFn<'_>,
) -> Result<(), String> {
    let old_path = get_old_path()?;
    let config_path = get_config_path()?;
    set_downloads_path(&old_path, new_path, &config_path)
}

pub(crate) fn clean_staging_folder_with(
    get_staging_dir: &PathProviderFn<'_>,
    clean_staging_dir: &dyn for<'a> Fn(&'a Path) -> Result<usize, String>,
) -> Result<usize, String> {
    let staging_dir = get_staging_dir()?;
    clean_staging_dir(&staging_dir)
}

fn get_staging_contents_with(
    get_library_dir: &PathProviderFn<'_>,
    temp_id: &str,
    relative_path: &str,
    collect_nodes: &CollectNodesFn<'_>,
) -> Result<Vec<FileNode>, String> {
    let library_dir = get_library_dir()?;
    let root_path = library_dir.join(temp_id);
    collect_nodes(&root_path, relative_path)
}

pub(crate) fn open_special_folder_command_with(
    folder_type: &str,
    get_downloads_dir: &PathProviderFn<'_>,
    get_profiles_dir: &PathProviderFn<'_>,
    get_library_dir: &PathProviderFn<'_>,
    open_path: &OpenSpecialFn<'_>,
) -> Result<(), String> {
    open_special_folder_with(
        folder_type,
        get_downloads_dir()?,
        get_profiles_dir()?,
        get_library_dir()?,
        open_path,
    )
}

pub(crate) fn delete_library_folder_command_with(
    get_library_dir: &PathProviderFn<'_>,
    zip_filename: &str,
    delete_library_folder_if_exists: &mut DeleteLibraryFn<'_>,
) -> Result<(), String> {
    let library_dir = get_library_dir()?;
    delete_library_folder_with(&library_dir, zip_filename, delete_library_folder_if_exists)
}

pub(crate) fn get_staging_contents_command_with(
    get_library_dir: &PathProviderFn<'_>,
    temp_id: &str,
    relative_path: &str,
    collect_nodes: &CollectNodesFn<'_>,
) -> Result<Vec<FileNode>, String> {
    get_staging_contents_with(get_library_dir, temp_id, relative_path, collect_nodes)
}

pub(crate) fn set_library_path_command_with(
    get_old_path: &PathProviderFn<'_>,
    get_config_path: &PathProviderFn<'_>,
    new_path: &str,
    set_library_path: &mut SetPathFn<'_>,
) -> Result<(), String> {
    let old_path = get_old_path()?;
    let config_path = get_config_path()?;
    set_library_path(&old_path, new_path, &config_path)
}

pub(crate) fn check_library_existence_command_with(
    get_library_dir: &PathProviderFn<'_>,
    filenames: Vec<String>,
    check_library_existence_map: &LibraryExistenceFn<'_>,
) -> Result<HashMap<String, bool>, String> {
    let library_dir = get_library_dir()?;
    Ok(check_library_existence_with(
        &library_dir,
        filenames,
        check_library_existence_map,
    ))
}

#[cfg(target_os = "linux")]
pub(crate) fn show_in_folder_linux_with(path: &Path, spawn_open: &mut SpawnOpenFn<'_>) {
    let target = linux_show_in_folder_target(path);
    let _ = spawn_open(&target);
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
