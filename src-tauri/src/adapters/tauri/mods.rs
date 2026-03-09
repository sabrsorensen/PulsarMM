use super::app_commands as tauri_app_commands;
use super::install::get_all_mods_for_render;
use crate::installation_detection::find_game_path;
use crate::mods::command_download_ops::{
    download_archive_to_path_with, maybe_emit_download_progress_with,
};
use crate::mods::command_info_ops::{ensure_mod_info_in_game_path, update_mod_id_in_game_path};
use crate::mods::command_mutations::{
    delete_mod_and_save_settings, rename_mod_in_settings, reorder_mods_from_settings,
};
use crate::mods::command_ops::{
    maybe_remove_mod_folder, mod_folder_path, settings_file_from_game_path,
};
use crate::mods::commands_runtime::{
    delete_mod_command_entry_with, delete_mod_runtime_with, download_mod_archive_app_flow_with,
    download_mod_archive_command_entry_with, download_mod_archive_runtime_with,
    ensure_mod_info_command_entry_with, ensure_mod_info_runtime_with,
    rename_mod_folder_command_entry_with, rename_mod_folder_runtime_with,
    reorder_mods_command_entry_with, reorder_mods_runtime_with,
    update_mod_id_in_json_command_entry_with, update_mod_id_in_json_runtime_with,
    update_mod_name_in_xml_command_entry_with, update_mod_name_in_xml_runtime_with,
};
use crate::{get_downloads_dir, get_library_dir, log_internal};
use std::fs;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn rename_mod_folder(
    app: AppHandle,
    old_name: String,
    new_name: String,
) -> Result<Vec<crate::models::ModRenderData>, String> {
    rename_mod_folder_runtime_with(
        old_name,
        new_name,
        |level, message| log_internal(&app, level, message),
        |old_name, new_name| {
            rename_mod_folder_command_entry_with(
                old_name,
                new_name,
                find_game_path,
                settings_file_from_game_path,
                rename_mod_in_settings,
                || get_library_dir(&app),
                |file_path, content| {
                    tauri_app_commands::save_file_for_app(app.clone(), file_path, content)
                },
                || get_all_mods_for_render(app.clone()),
                |level, message| log_internal(&app, level, message),
                |old_path, new_path| {
                    fs::rename(old_path, new_path).map_err(|e| {
                        let err = format!("Failed to rename folder: {}", e);
                        log_internal(&app, "ERROR", &err);
                        err
                    })
                },
            )
        },
    )
}

#[tauri::command]
pub fn delete_mod(
    app: AppHandle,
    mod_name: String,
) -> Result<Vec<crate::models::ModRenderData>, String> {
    delete_mod_runtime_with(
        mod_name,
        |level, message| log_internal(&app, level, message),
        |mod_name| {
            delete_mod_command_entry_with(
                mod_name,
                find_game_path,
                settings_file_from_game_path,
                mod_folder_path,
                maybe_remove_mod_folder,
                delete_mod_and_save_settings,
                || get_all_mods_for_render(app.clone()),
                |level, message| log_internal(&app, level, message),
            )
        },
    )
}

#[tauri::command]
pub fn reorder_mods(ordered_mod_names: Vec<String>) -> Result<String, String> {
    reorder_mods_runtime_with(ordered_mod_names, |ordered_mod_names| {
        reorder_mods_command_entry_with(
            ordered_mod_names,
            find_game_path,
            crate::mods::command_ops::settings_file_from_game_path,
            reorder_mods_from_settings,
        )
    })
}

#[tauri::command]
pub fn update_mod_name_in_xml(old_name: String, new_name: String) -> Result<String, String> {
    update_mod_name_in_xml_runtime_with(old_name, new_name, |old_name, new_name| {
        update_mod_name_in_xml_command_entry_with(
            old_name,
            new_name,
            find_game_path,
            crate::mods::command_ops::settings_file_from_game_path,
            rename_mod_in_settings,
        )
    })
}

#[tauri::command]
pub fn update_mod_id_in_json(mod_folder_name: String, new_mod_id: String) -> Result<(), String> {
    update_mod_id_in_json_runtime_with(
        mod_folder_name,
        new_mod_id,
        |mod_folder_name, new_mod_id| {
            update_mod_id_in_json_command_entry_with(
                mod_folder_name,
                new_mod_id,
                find_game_path,
                update_mod_id_in_game_path,
            )
        },
    )
}

#[tauri::command]
pub fn ensure_mod_info(
    mod_folder_name: String,
    mod_id: String,
    file_id: String,
    version: String,
    install_source: String,
) -> Result<(), String> {
    ensure_mod_info_runtime_with(
        mod_folder_name,
        mod_id,
        file_id,
        version,
        install_source,
        |mod_folder_name, mod_id, file_id, version, install_source| {
            ensure_mod_info_command_entry_with(
                mod_folder_name,
                mod_id,
                file_id,
                version,
                install_source,
                find_game_path,
                ensure_mod_info_in_game_path,
            )
        },
    )
}

#[tauri::command]
pub async fn download_mod_archive(
    app: AppHandle,
    download_url: String,
    file_name: String,
    download_id: Option<String>,
) -> Result<crate::models::DownloadResult, String> {
    download_mod_archive_runtime_with(
        download_url,
        file_name,
        download_id,
        |file_name, download_url, download_id| {
            let app_for_downloads = app.clone();
            let app_for_transfer = app.clone();
            let app_for_log = app.clone();
            let app_for_start_log = app.clone();
            async move {
                download_mod_archive_command_entry_with(
                    file_name,
                    download_url,
                    download_id,
                    |level, message| log_internal(&app_for_start_log, level, message),
                    |file_name, download_url, download_id| {
                        let app_for_downloads = app_for_downloads.clone();
                        let app_for_transfer = app_for_transfer.clone();
                        let app_for_log = app_for_log.clone();
                        async move {
                            download_mod_archive_app_flow_with(
                                file_name,
                                download_url,
                                download_id,
                                || get_downloads_dir(&app_for_downloads),
                                |url, path, id| {
                                    let app_handle = app_for_transfer.clone();
                                    async move {
                                        let mut on_progress = |downloaded, total_size| {
                                            let mut emit_payload = |payload| {
                                                let _ =
                                                    app_handle.emit("install-progress", payload);
                                            };
                                            maybe_emit_download_progress_with(
                                                id.as_deref(),
                                                downloaded,
                                                total_size,
                                                &mut emit_payload,
                                            );
                                        };
                                        download_archive_to_path_with(
                                            &url,
                                            path.as_path(),
                                            &mut on_progress,
                                        )
                                        .await
                                    }
                                },
                                |level, message| log_internal(&app_for_log, level, message),
                            )
                            .await
                        }
                    },
                )
                .await
            }
        },
    )
    .await
}
