pub(crate) mod apply;
pub mod apply_logic;
pub mod apply_ops;
pub mod engine;
pub(crate) mod storage;
use std::path::{Path, PathBuf};

fn profile_json_path(profiles_dir: &Path, profile_name: &str) -> PathBuf {
    profiles_dir.join(format!("{}.json", profile_name))
}

pub(crate) fn list_profiles_with(
    profiles_dir: &Path,
    collect_profile_names_from_dir: impl FnOnce(&Path) -> Vec<String>,
) -> Vec<String> {
    collect_profile_names_from_dir(profiles_dir)
}

pub(crate) fn delete_profile_with(
    profiles_dir: &Path,
    profile_name: &str,
    delete_profile_files_in_dir: impl FnOnce(&Path, &str) -> Result<(), String>,
) -> Result<(), String> {
    delete_profile_files_in_dir(profiles_dir, profile_name)
}

pub(crate) fn rename_profile_with(
    profiles_dir: &Path,
    old_name: &str,
    new_name: &str,
    rename_profile_files_in_dir: impl FnOnce(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    rename_profile_files_in_dir(profiles_dir, old_name, new_name)
}

pub(crate) fn create_empty_profile_with(
    profiles_dir: &Path,
    profile_name: &str,
    create_empty_profile_in_dir: impl FnOnce(&Path, &str) -> Result<(), String>,
) -> Result<(), String> {
    create_empty_profile_in_dir(profiles_dir, profile_name)
}

pub(crate) fn get_profile_mod_list_with(
    profiles_dir: &Path,
    profile_name: &str,
    read_profile_mod_list_from_json_path: impl FnOnce(&Path) -> Result<Vec<String>, String>,
) -> Result<Vec<String>, String> {
    let json_path = profile_json_path(profiles_dir, profile_name);
    read_profile_mod_list_from_json_path(&json_path)
}

pub(crate) fn copy_profile_with(
    profiles_dir: &Path,
    source_name: &str,
    new_name: &str,
    copy_profile_from_dir: impl FnOnce(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    copy_profile_from_dir(profiles_dir, source_name, new_name)
}

pub(crate) fn with_profiles_dir<R>(
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    run: impl FnOnce(&Path) -> Result<R, String>,
) -> Result<R, String> {
    let dir = resolve_profiles_dir()?;
    run(&dir)
}

pub(crate) fn list_profiles_command_with(
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    list: impl FnOnce(&Path) -> Vec<String>,
) -> Result<Vec<String>, String> {
    with_profiles_dir(resolve_profiles_dir, |dir| Ok(list(dir)))
}

pub(crate) fn delete_profile_command_with(
    profile_name: &str,
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    delete: impl FnOnce(&Path, &str) -> Result<(), String>,
) -> Result<(), String> {
    with_profiles_dir(resolve_profiles_dir, |dir| {
        delete_profile_with(dir, profile_name, delete)
    })
}

pub(crate) fn rename_profile_command_with(
    old_name: &str,
    new_name: &str,
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    rename: impl FnOnce(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    with_profiles_dir(resolve_profiles_dir, |dir| {
        rename_profile_with(dir, old_name, new_name, rename)
    })
}

pub(crate) fn create_empty_profile_command_with(
    profile_name: &str,
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    create: impl FnOnce(&Path, &str) -> Result<(), String>,
) -> Result<(), String> {
    with_profiles_dir(resolve_profiles_dir, |dir| {
        create_empty_profile_with(dir, profile_name, create)
    })
}

pub(crate) fn get_profile_mod_list_command_with(
    profile_name: &str,
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    read_list: impl FnOnce(&Path) -> Result<Vec<String>, String>,
) -> Result<Vec<String>, String> {
    with_profiles_dir(resolve_profiles_dir, |dir| {
        get_profile_mod_list_with(dir, profile_name, read_list)
    })
}

pub(crate) fn copy_profile_command_with(
    source_name: &str,
    new_name: &str,
    resolve_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    copy: impl FnOnce(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    with_profiles_dir(resolve_profiles_dir, |dir| {
        copy_profile_with(dir, source_name, new_name, copy)
    })
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
