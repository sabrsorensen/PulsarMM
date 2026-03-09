use super::services::TauriRuntime;
use crate::installation_detection::find_game_path;
use crate::models::{InstallationAnalysis, ModRenderData};
use crate::mods::conflict_resolution::resolve_conflict_in_paths;
use crate::mods::install_commands::{
    finalize_installation_command_entry_with, get_all_mods_for_render_command_entry_with,
    get_staging_dir_command_entry_with, install_mod_from_archive_command_entry_with,
    resolve_conflict_command_entry_with,
};
use crate::mods::install_service;
use crate::{get_downloads_dir, get_library_dir};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub fn get_staging_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let runtime = TauriRuntime::new(app);
    get_staging_dir_command_entry_with(&runtime, |runtime_ref| {
        install_service::get_staging_dir_with(runtime_ref)
    })
}

#[tauri::command]
pub fn get_all_mods_for_render(app: AppHandle) -> Result<Vec<ModRenderData>, String> {
    let runtime = TauriRuntime::new(&app);
    get_all_mods_for_render_command_entry_with(&runtime, |runtime_ref| {
        install_service::get_all_mods_for_render_with(runtime_ref)
    })
}

#[tauri::command]
pub async fn install_mod_from_archive(
    app: AppHandle,
    archive_path_str: String,
    download_id: String,
) -> Result<InstallationAnalysis, String> {
    let app_for_finalize = app.clone();
    let app_for_emit = app.clone();
    let app_for_downloads = app.clone();
    let app_for_library = app.clone();
    install_mod_from_archive_command_entry_with(
        archive_path_str,
        download_id,
        move || get_downloads_dir(&app_for_downloads),
        move || get_library_dir(&app_for_library),
        move |payload| {
            let _ = app_for_emit.emit("install-progress", payload);
        },
        |library_id, selected_folders, flatten_paths| {
            finalize_installation(
                app_for_finalize,
                library_id,
                selected_folders,
                flatten_paths,
            )
        },
    )
    .await
}

#[tauri::command]
pub fn finalize_installation(
    app: AppHandle,
    library_id: String,
    selected_folders: Vec<String>,
    flatten_paths: bool,
) -> Result<InstallationAnalysis, String> {
    let runtime = TauriRuntime::new(&app);
    finalize_installation_command_entry_with(
        &runtime,
        library_id,
        selected_folders,
        flatten_paths,
        |runtime_ref, lib_id, folders, flatten| {
            install_service::finalize_installation_with(runtime_ref, lib_id, folders, flatten)
        },
    )
}

#[tauri::command]
pub fn resolve_conflict(
    new_mod_name: String,
    old_mod_folder_name: String,
    temp_mod_path_str: String,
    replace: bool,
) -> Result<(), String> {
    resolve_conflict_command_entry_with(
        &new_mod_name,
        &old_mod_folder_name,
        &temp_mod_path_str,
        replace,
        find_game_path,
        resolve_conflict_in_paths,
    )
}
