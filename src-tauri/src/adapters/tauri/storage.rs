use super::{install, profiles};
use crate::models::FileNode;
use crate::path_ops::check_library_existence_map;
use crate::staging_contents;
use crate::storage::command_flow::{clear_downloads_and_library, set_downloads_path_with};
use crate::storage::commands::{
    check_library_existence_command_with, clean_staging_folder_with,
    clear_downloads_folder_command_with, delete_library_folder_command_with, get_path_string_with,
    get_staging_contents_command_with, open_folder_path_with, open_special_folder_command_with,
    set_downloads_path_command_with, set_library_path_command_with,
};
use crate::storage::logic::clean_staging_dir;
use crate::storage::ops::{delete_archive_file_if_exists, ensure_folder_exists};
use crate::{get_config_file_path, get_downloads_dir, get_library_dir, log_internal};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub fn show_in_folder(path: String) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("explorer").args(["/select,", &path]).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let p = PathBuf::from(&path);
        crate::storage::commands::show_in_folder_linux_with(&p, &mut |target| {
            Command::new("xdg-open")
                .arg(target)
                .spawn()
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
    }
}

#[tauri::command]
pub fn delete_archive_file(path: String) -> Result<(), String> {
    delete_archive_file_if_exists(std::path::Path::new(&path))
}

#[tauri::command]
pub fn clear_downloads_folder(app: AppHandle) -> Result<(), String> {
    clear_downloads_folder_command_with(
        &|| get_downloads_dir(&app),
        &|| get_library_dir(&app),
        &clear_downloads_and_library,
    )
}

#[tauri::command]
pub fn open_folder_path(path: String) -> Result<(), String> {
    let p = PathBuf::from(path);
    open_folder_path_with(&p, &ensure_folder_exists, &mut |path| {
        open::that(path).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_downloads_path(app: AppHandle) -> Result<String, String> {
    get_path_string_with(&|| get_downloads_dir(&app))
}

#[tauri::command]
pub async fn set_downloads_path(app: AppHandle, new_path: String) -> Result<(), String> {
    set_downloads_path_command_with(
        &|| get_downloads_dir(&app),
        &|| get_config_file_path(&app),
        &new_path,
        &mut |old_path, new_path, config_path| {
            set_downloads_path_with(old_path, new_path, config_path, |level, message| {
                log_internal(&app, level, message)
            })
        },
    )
}

#[tauri::command]
pub fn open_special_folder(app: AppHandle, folder_type: String) -> Result<(), String> {
    open_special_folder_command_with(
        folder_type.as_str(),
        &|| get_downloads_dir(&app),
        &|| profiles::get_profiles_dir(&app),
        &|| get_library_dir(&app),
        &|path| open::that(path).map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub fn clean_staging_folder(app: AppHandle) -> Result<usize, String> {
    clean_staging_folder_with(&|| install::get_staging_dir(&app), &clean_staging_dir)
}

#[tauri::command]
pub fn delete_library_folder(app: AppHandle, zip_filename: String) -> Result<(), String> {
    delete_library_folder_command_with(
        &|| get_library_dir(&app),
        &zip_filename,
        &mut crate::storage::ops::delete_library_folder_if_exists,
    )
}

#[tauri::command]
pub fn get_staging_contents(
    app: AppHandle,
    temp_id: String,
    relative_path: String,
) -> Result<Vec<FileNode>, String> {
    get_staging_contents_command_with(
        &|| get_library_dir(&app),
        &temp_id,
        &relative_path,
        &staging_contents::collect_nodes,
    )
}

#[tauri::command]
pub async fn set_library_path(app: AppHandle, new_path: String) -> Result<(), String> {
    set_library_path_command_with(
        &|| get_library_dir(&app),
        &|| get_config_file_path(&app),
        &new_path,
        &mut crate::storage::command_flow::set_library_path_with,
    )
}

#[tauri::command]
pub fn get_library_path(app: AppHandle) -> Result<String, String> {
    get_path_string_with(&|| get_library_dir(&app))
}

#[tauri::command]
pub fn check_library_existence(
    app: AppHandle,
    filenames: Vec<String>,
) -> Result<HashMap<String, bool>, String> {
    check_library_existence_command_with(
        &|| get_library_dir(&app),
        filenames,
        &check_library_existence_map,
    )
}
