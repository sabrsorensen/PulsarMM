use crate::models::{InstallProgressPayload, InstallationAnalysis, ModRenderData};
use crate::mods::install_command_core::resolve_conflict_with;
use crate::mods::install_command_runtime::install_mod_from_archive_with_progress_events;
use std::path::{Path, PathBuf};

fn get_staging_dir_with_provider(
    get_staging_dir: impl FnOnce() -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    get_staging_dir()
}

fn get_all_mods_for_render_with_provider(
    get_mods: impl FnOnce() -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    get_mods()
}

fn resolve_conflict_with_provider(
    new_mod_name: &str,
    old_mod_folder_name: &str,
    temp_mod_path_str: &str,
    replace: bool,
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    resolve_conflict_in_paths: impl FnOnce(&Path, &str, &str, &Path, bool) -> Result<(), String>,
) -> Result<(), String> {
    resolve_conflict_with(
        find_game_path,
        new_mod_name,
        old_mod_folder_name,
        temp_mod_path_str,
        replace,
        resolve_conflict_in_paths,
    )
}

pub(crate) async fn install_mod_from_archive_with_provider(
    archive_path_str: String,
    download_id: String,
    install: impl FnOnce(
        String,
        String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<InstallationAnalysis, String>> + Send>,
    >,
) -> Result<InstallationAnalysis, String> {
    install(archive_path_str, download_id).await
}

pub(crate) async fn install_mod_from_archive_command_with(
    archive_path_str: String,
    download_id: String,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String>,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    emit_event: impl Fn(InstallProgressPayload) + Clone + Send + 'static,
    finalize_installation: impl FnOnce(
        String,
        Vec<String>,
        bool,
    ) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    install_mod_from_archive_with_progress_events(
        archive_path_str,
        download_id,
        get_downloads_dir,
        get_library_dir,
        emit_event,
        finalize_installation,
    )
    .await
}

pub(crate) fn get_staging_dir_command_with(
    get_staging_dir: impl FnOnce() -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    get_staging_dir_with_provider(get_staging_dir)
}

pub(crate) fn get_all_mods_for_render_command_with(
    get_mods: impl FnOnce() -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    get_all_mods_for_render_with_provider(get_mods)
}

pub(crate) fn finalize_installation_command_with<R>(
    runtime: &R,
    library_id: String,
    selected_folders: Vec<String>,
    flatten_paths: bool,
    finalize: impl FnOnce(&R, String, Vec<String>, bool) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    finalize_installation_with_provider(
        runtime,
        library_id,
        selected_folders,
        flatten_paths,
        finalize,
    )
}

pub(crate) fn resolve_conflict_command_with(
    new_mod_name: &str,
    old_mod_folder_name: &str,
    temp_mod_path_str: &str,
    replace: bool,
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    resolve_conflict_in_paths: impl FnOnce(&Path, &str, &str, &Path, bool) -> Result<(), String>,
) -> Result<(), String> {
    resolve_conflict_with_provider(
        new_mod_name,
        old_mod_folder_name,
        temp_mod_path_str,
        replace,
        find_game_path,
        resolve_conflict_in_paths,
    )
}

fn finalize_installation_with_provider<R>(
    runtime: &R,
    library_id: String,
    selected_folders: Vec<String>,
    flatten_paths: bool,
    finalize: impl FnOnce(&R, String, Vec<String>, bool) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    finalize(runtime, library_id, selected_folders, flatten_paths)
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
