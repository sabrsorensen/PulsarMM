use crate::adapters::tauri::profiles as tauri_profiles;
use crate::app::commands::{
    check_for_untracked_mods_command_with, delete_settings_file_command_with,
    detect_game_installation_with, http_request_with, open_mods_folder_command_with,
    resize_window_with, run_legacy_migration_command_with, save_file_with, take_pending_intent,
    write_to_log_with,
};
use crate::app::detection::missing_settings_warning;
use crate::app::http::perform_http_request;
use crate::app::migration::run_legacy_migration_in_paths;
use crate::app::ops::{
    check_untracked_mods_for_game_path, delete_settings_at_path, delete_settings_without_game_path,
    open_mods_folder_for_game_path, save_text_file,
};
use crate::installation_detection::{
    detect_game_paths, find_game_path, resolve_game_root_from_selection, set_manual_game_path,
};
use crate::models::{GamePaths, HttpResponse, StartupState};
use crate::settings_paths;
use crate::utils::config::{load_config_or_default, save_config};
use crate::{get_config_file_path, log_internal};
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, State};
use tauri_plugin_fs::FsExt;

#[cfg(target_os = "windows")]
use crate::app::commands::has_uninstaller_in_parent;

pub(crate) fn save_file_for_app(
    app: AppHandle,
    file_path: String,
    content: String,
) -> Result<(), String> {
    save_file_with(
        Path::new(&file_path),
        &content,
        |level, msg| log_internal(&app, level, msg),
        save_text_file,
    )
}

#[tauri::command]
pub async fn http_request(
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
) -> Result<HttpResponse, String> {
    http_request_with(url, method, headers, |url, method, headers| {
        Box::pin(perform_http_request(url, method, headers))
    })
    .await
}

#[tauri::command]
pub fn delete_settings_file() -> Result<String, String> {
    delete_settings_file_command_with(
        find_game_path,
        settings_paths::mod_settings_file,
        delete_settings_at_path,
        delete_settings_without_game_path,
    )
}

#[tauri::command]
pub fn detect_game_installation(app: AppHandle) -> Option<GamePaths> {
    let mut log = |level: &str, msg: &str| log_internal(&app, level, msg);
    detect_game_installation_with(
        &find_game_path,
        &detect_game_paths,
        &|path| {
            app.fs_scope()
                .allow_directory(path, true)
                .map_err(|e| e.to_string())
        },
        &mut log,
        &missing_settings_warning,
    )
}

#[tauri::command]
pub fn set_game_install_path(app: AppHandle, selected_path: String) -> Result<GamePaths, String> {
    let selected = Path::new(&selected_path);
    let game_root = resolve_game_root_from_selection(selected).ok_or_else(|| {
        "Could not resolve a No Man's Sky install from the selected path.".to_string()
    })?;

    let config_path = get_config_file_path(&app)?;
    let mut config = load_config_or_default(&config_path, true);
    config.custom_game_path = Some(game_root.to_string_lossy().into_owned());
    save_config(&config_path, &config)?;

    set_manual_game_path(Some(game_root.clone()));
    app.fs_scope()
        .allow_directory(&game_root, true)
        .map_err(|e| e.to_string())?;

    detect_game_paths(&game_root)
        .ok_or_else(|| "Selected path is not a valid No Man's Sky install.".to_string())
}

#[tauri::command]
pub fn open_mods_folder() -> Result<(), String> {
    open_mods_folder_command_with(find_game_path, |game_path| {
        open_mods_folder_for_game_path(game_path, |mods_path| {
            open::that(mods_path).map_err(|e| e.to_string())
        })
    })
}

#[tauri::command]
pub fn save_file(app: AppHandle, file_path: String, content: String) -> Result<(), String> {
    save_file_for_app(app, file_path, content)
}

#[tauri::command]
pub fn resize_window(window: tauri::Window, width: f64) -> Result<(), String> {
    let current_height = window.outer_size().map_err(|e| e.to_string())?.height;
    resize_window_with(width, current_height, |size| {
        window.set_size(size).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn check_for_untracked_mods() -> bool {
    check_for_untracked_mods_command_with(find_game_path, check_untracked_mods_for_game_path)
}

#[tauri::command]
pub async fn run_legacy_migration(app: AppHandle) -> Result<(), String> {
    run_legacy_migration_command_with(
        || get_config_file_path(&app),
        || tauri_profiles::get_profiles_dir(&app),
        find_game_path,
        run_legacy_migration_in_paths,
    )
}

#[tauri::command]
pub fn write_to_log(app: AppHandle, level: String, message: String) -> Result<(), String> {
    write_to_log_with(&level, &message, |lvl, msg| log_internal(&app, lvl, msg));
    Ok(())
}

#[tauri::command]
pub fn check_startup_intent(state: State<'_, StartupState>) -> Option<String> {
    take_pending_intent(&state.pending_nxm)
}

#[tauri::command]
pub fn is_app_installed(_app: AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        let current_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => return false,
        };
        return has_uninstaller_in_parent(&current_exe);
    }

    #[cfg(target_os = "linux")]
    return true;

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    return true;
}
