use crate::models::ModInfo;
use crate::models::{DownloadResult, ModRenderData};
use crate::mods::command_flow::{
    delete_mod_flow, maybe_persist_renamed_mod_settings, maybe_sync_library_rename_for_mod,
    rename_mod_folder_flow, reorder_mods_flow,
};
use crate::mods::command_ops::LibraryRenameSync;
use crate::mods::info_ops::EnsureModInfoInput;
use std::future::Future;
use std::path::{Path, PathBuf};

fn rename_request_log_message(old_name: &str, new_name: &str) -> String {
    format!("Requesting rename: '{}' -> '{}'", old_name, new_name)
}

fn delete_request_log_message(mod_name: &str) -> String {
    format!("Requesting deletion of mod: {}", mod_name)
}

fn download_request_log_message(file_name: &str) -> String {
    format!("Starting download request for: {}", file_name)
}

fn build_ensure_mod_info_input(
    mod_id: String,
    file_id: String,
    version: String,
    install_source: String,
) -> EnsureModInfoInput {
    EnsureModInfoInput {
        mod_id,
        file_id,
        version,
        install_source,
    }
}

pub(crate) fn reorder_mods_with(
    ordered_mod_names: &[String],
    find_game_path: impl Fn() -> Option<std::path::PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> std::path::PathBuf,
    reorder_mods_from_settings: impl Fn(&Path, &[String]) -> Result<String, String>,
) -> Result<String, String> {
    reorder_mods_flow(
        ordered_mod_names,
        find_game_path,
        settings_file_from_game_path,
        reorder_mods_from_settings,
    )
}

pub(crate) fn reorder_mods_command_with(
    ordered_mod_names: Vec<String>,
    run_flow: impl FnOnce(Vec<String>) -> Result<String, String>,
) -> Result<String, String> {
    run_flow(ordered_mod_names)
}

pub(crate) fn rename_mod_folder_command_with(
    old_name: String,
    new_name: String,
    mut log: impl FnMut(&str, &str),
    run_flow: impl FnOnce(String, String) -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    log(
        "INFO",
        &rename_request_log_message(old_name.as_str(), new_name.as_str()),
    );
    run_flow(old_name, new_name)
}

pub(crate) fn delete_mod_command_with(
    mod_name: String,
    mut log: impl FnMut(&str, &str),
    run_flow: impl FnOnce(String) -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    log("INFO", &delete_request_log_message(mod_name.as_str()));
    run_flow(mod_name)
}

pub(crate) fn rename_mod_folder_with_deps(
    old_name: String,
    new_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    validate_rename_paths: impl Fn(bool, bool) -> Result<(), String>,
    read_mod_info: impl Fn(&Path) -> Option<ModInfo>,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    sync_library_folder_rename: impl Fn(&Path, &str, &str, &str) -> Result<LibraryRenameSync, String>,
    rename_dir: impl Fn(&Path, &Path) -> Result<(), String>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    update_mod_name_in_xml: impl Fn(String, String) -> Result<String, String>,
    save_file: impl Fn(String, String) -> Result<(), String>,
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
    log: impl Fn(&str, &str),
) -> Result<Vec<ModRenderData>, String> {
    rename_mod_folder_flow(
        old_name,
        new_name,
        find_game_path,
        mod_folder_path,
        validate_rename_paths,
        |old_path, old_name, new_name| {
            maybe_sync_library_rename_for_mod(
                old_path,
                old_name,
                new_name,
                &read_mod_info,
                &get_library_dir,
                &sync_library_folder_rename,
                &log,
            );
        },
        rename_dir,
        settings_file_from_game_path,
        |settings_file, old_name, new_name| {
            maybe_persist_renamed_mod_settings(
                settings_file,
                old_name,
                new_name,
                &update_mod_name_in_xml,
                &save_file,
                &log,
            )
        },
        get_all_mods_for_render,
    )
}

pub(crate) fn delete_mod_with_deps(
    mod_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    maybe_remove_mod_folder: impl Fn(&Path, &str) -> Result<bool, String>,
    delete_mod_and_save_settings: impl Fn(&Path, &str) -> Result<(), String>,
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
    log: impl Fn(&str, &str),
) -> Result<Vec<ModRenderData>, String> {
    delete_mod_flow(
        mod_name,
        find_game_path,
        settings_file_from_game_path,
        mod_folder_path,
        maybe_remove_mod_folder,
        delete_mod_and_save_settings,
        log,
        get_all_mods_for_render,
    )
}

pub(crate) fn update_mod_name_in_xml_with(
    old_name: &str,
    new_name: &str,
    run_flow: impl FnOnce(&str, &str) -> Result<String, String>,
) -> Result<String, String> {
    run_flow(old_name, new_name)
}

pub(crate) fn update_mod_name_in_xml_command_with(
    old_name: String,
    new_name: String,
    run_flow: impl FnOnce(String, String) -> Result<String, String>,
) -> Result<String, String> {
    run_flow(old_name, new_name)
}

pub(crate) fn update_mod_id_in_json_with(
    mod_folder_name: &str,
    new_mod_id: &str,
    run_flow: impl FnOnce(&str, &str) -> Result<(), String>,
) -> Result<(), String> {
    run_flow(mod_folder_name, new_mod_id)
}

pub(crate) fn update_mod_id_in_json_command_with(
    mod_folder_name: String,
    new_mod_id: String,
    run_flow: impl FnOnce(String, String) -> Result<(), String>,
) -> Result<(), String> {
    run_flow(mod_folder_name, new_mod_id)
}

pub(crate) fn ensure_mod_info_with(
    mod_folder_name: &str,
    input: &EnsureModInfoInput,
    run_flow: impl FnOnce(&str, &EnsureModInfoInput) -> Result<(), String>,
) -> Result<(), String> {
    run_flow(mod_folder_name, input)
}

pub(crate) fn ensure_mod_info_command_with(
    mod_folder_name: String,
    mod_id: String,
    file_id: String,
    version: String,
    install_source: String,
    run_flow: impl FnOnce(String, EnsureModInfoInput) -> Result<(), String>,
) -> Result<(), String> {
    let input = build_ensure_mod_info_input(mod_id, file_id, version, install_source);
    run_flow(mod_folder_name, input)
}

pub(crate) async fn download_mod_archive_command_with<F, Fut>(
    file_name: String,
    download_url: String,
    download_id: Option<String>,
    mut run_flow: F,
) -> Result<DownloadResult, String>
where
    F: FnMut(String, String, Option<String>) -> Fut,
    Fut: Future<Output = Result<DownloadResult, String>>,
{
    run_flow(file_name, download_url, download_id).await
}

pub(crate) async fn download_mod_archive_with<F, Fut>(
    file_name: String,
    download_url: String,
    download_id: Option<String>,
    mut log: impl FnMut(&str, &str),
    mut run_flow: F,
) -> Result<DownloadResult, String>
where
    F: FnMut(String, String, Option<String>) -> Fut,
    Fut: Future<Output = Result<DownloadResult, String>>,
{
    log("INFO", &download_request_log_message(&file_name));
    run_flow(file_name, download_url, download_id).await
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
