use crate::models::{DownloadResult, ModInfo, ModRenderData};
use crate::mods::command_flow::{
    ensure_mod_info_flow, update_mod_id_in_json_flow, update_mod_name_in_xml_flow,
};
use crate::mods::command_ops::{
    mod_folder_path, sync_library_folder_rename, validate_rename_paths,
};
use crate::mods::commands::{
    delete_mod_command_with, delete_mod_with_deps, download_mod_archive_command_with,
    download_mod_archive_with, ensure_mod_info_command_with, ensure_mod_info_with,
    rename_mod_folder_command_with, rename_mod_folder_with_deps, reorder_mods_command_with,
    reorder_mods_with, update_mod_id_in_json_command_with, update_mod_id_in_json_with,
    update_mod_name_in_xml_command_with, update_mod_name_in_xml_with,
};
use crate::read_mod_info;
use std::future::Future;
use std::path::{Path, PathBuf};

pub(crate) fn rename_mod_folder_runtime_with(
    old_name: String,
    new_name: String,
    mut log: impl FnMut(&str, &str),
    run_flow: impl FnOnce(String, String) -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    rename_mod_folder_command_with(
        old_name,
        new_name,
        |level, message| log(level, message),
        run_flow,
    )
}

pub(crate) fn delete_mod_runtime_with(
    mod_name: String,
    mut log: impl FnMut(&str, &str),
    run_flow: impl FnOnce(String) -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    delete_mod_command_with(mod_name, |level, message| log(level, message), run_flow)
}

pub(crate) fn reorder_mods_runtime_with(
    ordered_mod_names: Vec<String>,
    run_flow: impl FnOnce(Vec<String>) -> Result<String, String>,
) -> Result<String, String> {
    reorder_mods_command_with(ordered_mod_names, run_flow)
}

pub(crate) fn update_mod_name_in_xml_runtime_with(
    old_name: String,
    new_name: String,
    run_flow: impl FnOnce(String, String) -> Result<String, String>,
) -> Result<String, String> {
    update_mod_name_in_xml_command_with(old_name, new_name, run_flow)
}

pub(crate) fn update_mod_id_in_json_runtime_with(
    mod_folder_name: String,
    new_mod_id: String,
    run_flow: impl FnOnce(String, String) -> Result<(), String>,
) -> Result<(), String> {
    update_mod_id_in_json_command_with(mod_folder_name, new_mod_id, run_flow)
}

pub(crate) fn ensure_mod_info_runtime_with(
    mod_folder_name: String,
    mod_id: String,
    file_id: String,
    version: String,
    install_source: String,
    run_flow: impl FnOnce(String, String, String, String, String) -> Result<(), String>,
) -> Result<(), String> {
    run_flow(mod_folder_name, mod_id, file_id, version, install_source)
}

pub(crate) async fn download_mod_archive_runtime_with<F, Fut>(
    download_url: String,
    file_name: String,
    download_id: Option<String>,
    run_flow: F,
) -> Result<DownloadResult, String>
where
    F: FnMut(String, String, Option<String>) -> Fut,
    Fut: Future<Output = Result<DownloadResult, String>>,
{
    download_mod_archive_command_with(file_name, download_url, download_id, run_flow).await
}

pub(crate) async fn download_mod_archive_command_entry_with<F, Fut>(
    file_name: String,
    download_url: String,
    download_id: Option<String>,
    mut log: impl FnMut(&str, &str),
    run_download_flow: F,
) -> Result<DownloadResult, String>
where
    F: Fn(String, String, Option<String>) -> Fut,
    Fut: Future<Output = Result<DownloadResult, String>>,
{
    download_mod_archive_with(
        file_name,
        download_url,
        download_id,
        |level, message| log(level, message),
        run_download_flow,
    )
    .await
}

fn rename_mod_folder_app_flow_with(
    old_name: String,
    new_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    validate_rename_paths: impl Fn(bool, bool) -> Result<(), String>,
    read_mod_info: impl Fn(&Path) -> Option<ModInfo>,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    sync_library_folder_rename: impl Fn(
        &Path,
        &str,
        &str,
        &str,
    )
        -> Result<crate::mods::command_ops::LibraryRenameSync, String>,
    rename_dir: impl Fn(&Path, &Path) -> Result<(), String>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    update_mod_name_in_xml: impl Fn(String, String) -> Result<String, String>,
    save_file: impl Fn(String, String) -> Result<(), String>,
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
    log: impl Fn(&str, &str),
) -> Result<Vec<ModRenderData>, String> {
    rename_mod_folder_with_deps(
        old_name,
        new_name,
        find_game_path,
        mod_folder_path,
        validate_rename_paths,
        read_mod_info,
        get_library_dir,
        sync_library_folder_rename,
        rename_dir,
        settings_file_from_game_path,
        update_mod_name_in_xml,
        save_file,
        get_all_mods_for_render,
        log,
    )
}

fn delete_mod_app_flow_with(
    mod_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    maybe_remove_mod_folder: impl Fn(&Path, &str) -> Result<bool, String>,
    delete_mod_and_save_settings: impl Fn(&Path, &str) -> Result<(), String>,
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
    log: impl Fn(&str, &str),
) -> Result<Vec<ModRenderData>, String> {
    delete_mod_with_deps(
        mod_name,
        find_game_path,
        settings_file_from_game_path,
        mod_folder_path,
        maybe_remove_mod_folder,
        delete_mod_and_save_settings,
        get_all_mods_for_render,
        log,
    )
}

pub(crate) fn rename_mod_folder_command_entry_with(
    old_name: String,
    new_name: String,
    find_game_path: impl Fn() -> Option<PathBuf> + Clone,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf + Clone,
    rename_mod_in_settings: impl Fn(&Path, &str, &str) -> Result<String, String> + Clone,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    save_file: impl Fn(String, String) -> Result<(), String>,
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
    log: impl Fn(&str, &str),
    rename_dir: impl Fn(&Path, &Path) -> Result<(), String>,
) -> Result<Vec<ModRenderData>, String> {
    let update_find_game_path = find_game_path.clone();
    let update_settings_file_from_game_path = settings_file_from_game_path.clone();
    let update_rename_mod_in_settings = rename_mod_in_settings.clone();

    rename_mod_folder_app_flow_with(
        old_name,
        new_name,
        find_game_path,
        mod_folder_path,
        validate_rename_paths,
        read_mod_info,
        get_library_dir,
        sync_library_folder_rename,
        rename_dir,
        settings_file_from_game_path,
        move |old_name, new_name| {
            update_mod_name_in_xml_command_entry_with(
                old_name,
                new_name,
                update_find_game_path.clone(),
                update_settings_file_from_game_path.clone(),
                update_rename_mod_in_settings.clone(),
            )
        },
        save_file,
        get_all_mods_for_render,
        log,
    )
}

pub(crate) fn delete_mod_command_entry_with(
    mod_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    maybe_remove_mod_folder: impl Fn(&Path, &str) -> Result<bool, String>,
    delete_mod_and_save_settings: impl Fn(&Path, &str) -> Result<(), String>,
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
    log: impl Fn(&str, &str),
) -> Result<Vec<ModRenderData>, String> {
    delete_mod_app_flow_with(
        mod_name,
        find_game_path,
        settings_file_from_game_path,
        mod_folder_path,
        maybe_remove_mod_folder,
        delete_mod_and_save_settings,
        get_all_mods_for_render,
        log,
    )
}

pub(crate) fn reorder_mods_command_entry_with(
    ordered_mod_names: Vec<String>,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    reorder_mods_from_settings: impl Fn(&Path, &[String]) -> Result<String, String>,
) -> Result<String, String> {
    reorder_mods_with(
        &ordered_mod_names,
        find_game_path,
        settings_file_from_game_path,
        reorder_mods_from_settings,
    )
}

pub(crate) fn update_mod_name_in_xml_command_entry_with(
    old_name: String,
    new_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    rename_mod_in_settings: impl Fn(&Path, &str, &str) -> Result<String, String>,
) -> Result<String, String> {
    update_mod_name_in_xml_with(
        old_name.as_str(),
        new_name.as_str(),
        |old_name, new_name| {
            update_mod_name_in_xml_flow(
                old_name,
                new_name,
                find_game_path,
                settings_file_from_game_path,
                rename_mod_in_settings,
            )
        },
    )
}

pub(crate) fn update_mod_id_in_json_command_entry_with(
    mod_folder_name: String,
    new_mod_id: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    update_mod_id_in_game_path: impl Fn(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    update_mod_id_in_json_with(
        mod_folder_name.as_str(),
        new_mod_id.as_str(),
        |mod_folder_name, new_mod_id| {
            update_mod_id_in_json_flow(
                mod_folder_name,
                new_mod_id,
                find_game_path,
                update_mod_id_in_game_path,
            )
        },
    )
}

pub(crate) fn ensure_mod_info_command_entry_with(
    mod_folder_name: String,
    mod_id: String,
    file_id: String,
    version: String,
    install_source: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    ensure_mod_info_in_game_path: impl Fn(
        &Path,
        &str,
        &crate::mods::info_ops::EnsureModInfoInput,
    ) -> Result<(), String>,
) -> Result<(), String> {
    ensure_mod_info_command_with(
        mod_folder_name,
        mod_id,
        file_id,
        version,
        install_source,
        |mod_folder_name, input| {
            ensure_mod_info_with(
                mod_folder_name.as_str(),
                &input,
                |mod_folder_name, input| {
                    ensure_mod_info_flow(
                        mod_folder_name,
                        input,
                        find_game_path,
                        ensure_mod_info_in_game_path,
                    )
                },
            )
        },
    )
}

pub(crate) async fn download_mod_archive_app_flow_with<F, Fut>(
    file_name: String,
    download_url: String,
    download_id: Option<String>,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String>,
    download_archive_to_path: F,
    log: impl Fn(&str, &str),
) -> Result<DownloadResult, String>
where
    F: Fn(String, PathBuf, Option<String>) -> Fut,
    Fut: Future<Output = Result<DownloadResult, String>>,
{
    crate::mods::command_flow::download_mod_archive_flow(
        &file_name,
        &download_url,
        download_id.as_deref(),
        get_downloads_dir,
        download_archive_to_path,
        log,
    )
    .await
}

#[cfg(test)]
#[path = "commands_runtime_tests.rs"]
mod tests;
