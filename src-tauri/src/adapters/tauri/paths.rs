use crate::app::paths::{
    app_data_file_path_with, get_pulsar_root_with, resolve_custom_path_with, storage_dir_with,
};
use crate::utils::config::load_config_or_default;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

fn resolve_custom_library_path(app: &AppHandle) -> Option<String> {
    resolve_custom_path_with(&|| get_config_file_path(app), &|config_path| {
        load_config_or_default(config_path, true).custom_library_path
    })
}

fn resolve_custom_download_path(app: &AppHandle) -> Option<String> {
    resolve_custom_path_with(&|| get_config_file_path(app), &|config_path| {
        load_config_or_default(config_path, true).custom_download_path
    })
}

pub(crate) fn get_pulsar_root(app: &AppHandle) -> Result<PathBuf, String> {
    get_pulsar_root_with(&|| {
        app.path()
            .resolve("Pulsar", BaseDirectory::Data)
            .map_err(|e| e.to_string())
    })
}

pub(crate) fn get_config_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file_path_with(
        &|| app.path().app_data_dir().map_err(|e| e.to_string()),
        "config.json",
    )
}

pub(crate) fn get_state_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file_path_with(
        &|| app.path().app_data_dir().map_err(|e| e.to_string()),
        "window-state.json",
    )
}

pub(crate) fn get_library_dir(app: &AppHandle) -> Result<PathBuf, String> {
    storage_dir_with(
        resolve_custom_library_path(app),
        &|| get_pulsar_root(app),
        "Library",
    )
}

pub(crate) fn get_downloads_dir(app: &AppHandle) -> Result<PathBuf, String> {
    storage_dir_with(
        resolve_custom_download_path(app),
        &|| get_pulsar_root(app),
        "downloads",
    )
}
