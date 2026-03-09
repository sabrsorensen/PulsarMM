use crate::models::{InstallationAnalysis, ModRenderData};
use crate::mods::install::{
    finalize_installation_command_with, get_all_mods_for_render_command_with,
    get_staging_dir_command_with, install_mod_from_archive_command_with,
    install_mod_from_archive_with_provider, resolve_conflict_command_with,
};
use std::path::PathBuf;

pub(crate) async fn install_mod_from_archive_command_entry_with(
    archive_path_str: String,
    download_id: String,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String> + Clone + Send + 'static,
    get_library_dir: impl Fn() -> Result<PathBuf, String> + Clone + Send + 'static,
    emit_progress: impl Fn(crate::models::InstallProgressPayload) + Clone + Send + 'static,
    finalize: impl FnOnce(String, Vec<String>, bool) -> Result<InstallationAnalysis, String>
        + Send
        + 'static,
) -> Result<InstallationAnalysis, String> {
    install_mod_from_archive_runtime_with(
        archive_path_str,
        download_id,
        get_downloads_dir,
        get_library_dir,
        emit_progress,
        finalize,
    )
    .await
}

pub(crate) async fn install_mod_from_archive_runtime_with(
    archive_path_str: String,
    download_id: String,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String> + Clone + Send + 'static,
    get_library_dir: impl Fn() -> Result<PathBuf, String> + Clone + Send + 'static,
    emit_progress: impl Fn(crate::models::InstallProgressPayload) + Clone + Send + 'static,
    finalize: impl FnOnce(String, Vec<String>, bool) -> Result<InstallationAnalysis, String>
        + Send
        + 'static,
) -> Result<InstallationAnalysis, String> {
    install_mod_from_archive_with_provider(
        archive_path_str,
        download_id,
        move |archive, download| {
            Box::pin(install_mod_from_archive_command_with(
                archive,
                download,
                get_downloads_dir,
                get_library_dir,
                emit_progress,
                finalize,
            ))
        },
    )
    .await
}

pub(crate) fn get_staging_dir_runtime_with<R>(
    runtime: &R,
    get_staging_dir: impl FnOnce(&R) -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    get_staging_dir_command_with(|| get_staging_dir(runtime))
}

pub(crate) fn get_staging_dir_command_entry_with<R>(
    runtime: &R,
    get_staging_dir: impl FnOnce(&R) -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    get_staging_dir_runtime_with(runtime, get_staging_dir)
}

pub(crate) fn get_all_mods_for_render_runtime_with<R>(
    runtime: &R,
    get_mods: impl FnOnce(&R) -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    get_all_mods_for_render_command_with(|| get_mods(runtime))
}

pub(crate) fn get_all_mods_for_render_command_entry_with<R>(
    runtime: &R,
    get_mods: impl FnOnce(&R) -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    get_all_mods_for_render_runtime_with(runtime, get_mods)
}

pub(crate) fn finalize_installation_runtime_with<R>(
    runtime: &R,
    library_id: String,
    selected_folders: Vec<String>,
    flatten_paths: bool,
    finalize: impl FnOnce(&R, String, Vec<String>, bool) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    finalize_installation_command_with(
        runtime,
        library_id,
        selected_folders,
        flatten_paths,
        finalize,
    )
}

pub(crate) fn finalize_installation_command_entry_with<R>(
    runtime: &R,
    library_id: String,
    selected_folders: Vec<String>,
    flatten_paths: bool,
    finalize: impl FnOnce(&R, String, Vec<String>, bool) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    finalize_installation_runtime_with(
        runtime,
        library_id,
        selected_folders,
        flatten_paths,
        finalize,
    )
}

pub(crate) fn resolve_conflict_runtime_with(
    new_mod_name: &str,
    old_mod_folder_name: &str,
    temp_mod_path_str: &str,
    replace: bool,
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    resolve_conflict: impl FnOnce(
        &std::path::Path,
        &str,
        &str,
        &std::path::Path,
        bool,
    ) -> Result<(), String>,
) -> Result<(), String> {
    resolve_conflict_command_with(
        new_mod_name,
        old_mod_folder_name,
        temp_mod_path_str,
        replace,
        find_game_path,
        resolve_conflict,
    )
}

pub(crate) fn resolve_conflict_command_entry_with(
    new_mod_name: &str,
    old_mod_folder_name: &str,
    temp_mod_path_str: &str,
    replace: bool,
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    resolve_conflict: impl FnOnce(
        &std::path::Path,
        &str,
        &str,
        &std::path::Path,
        bool,
    ) -> Result<(), String>,
) -> Result<(), String> {
    resolve_conflict_runtime_with(
        new_mod_name,
        old_mod_folder_name,
        temp_mod_path_str,
        replace,
        find_game_path,
        resolve_conflict,
    )
}

#[cfg(test)]
#[path = "install_commands_tests.rs"]
mod tests;
