use super::detection::run_game_detection_workflow;
use crate::models::{GamePaths, HttpResponse};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::LogicalSize;

#[cfg(any(test, target_os = "windows"))]
pub(crate) fn has_uninstaller_in_parent(exe_path: &std::path::Path) -> bool {
    exe_path
        .parent()
        .map(|parent| parent.join("Uninstall.exe").exists())
        .unwrap_or(false)
}

pub fn take_pending_intent(pending: &Mutex<Option<String>>) -> Option<String> {
    let mut pending = match pending.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    pending.take()
}

pub(crate) fn delete_settings_file_with(
    game_path: Option<PathBuf>,
    mod_settings_file: impl Fn(&Path) -> PathBuf,
    delete_settings_at_path: impl Fn(&Path) -> Result<String, String>,
    delete_settings_without_game_path: impl Fn() -> Result<String, String>,
) -> Result<String, String> {
    if let Some(game_path) = game_path {
        let settings_file = mod_settings_file(&game_path);
        delete_settings_at_path(&settings_file)
    } else {
        delete_settings_without_game_path()
    }
}

pub(crate) fn open_mods_folder_with(
    game_path: Option<PathBuf>,
    open_mods_folder_for_game_path: impl FnOnce(Option<&Path>) -> Result<(), String>,
) -> Result<(), String> {
    open_mods_folder_for_game_path(game_path.as_deref())
}

pub(crate) fn check_for_untracked_mods_with(
    game_path: Option<PathBuf>,
    check_untracked_mods_for_game_path: impl Fn(Option<&Path>) -> bool,
) -> bool {
    check_untracked_mods_for_game_path(game_path.as_deref())
}

pub(crate) fn run_legacy_migration_with(
    config_path: &Path,
    profiles_dir: &Path,
    game_path: Option<&Path>,
    run_legacy_migration_in_paths: impl Fn(&Path, &Path, Option<&Path>) -> Result<bool, String>,
) -> Result<(), String> {
    let _ = run_legacy_migration_in_paths(config_path, profiles_dir, game_path)?;
    Ok(())
}

pub(crate) fn detect_game_installation_with(
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    detect_game_paths: &dyn Fn(&Path) -> Option<GamePaths>,
    allow_directory: &dyn Fn(&Path) -> Result<(), String>,
    log: &mut dyn FnMut(&str, &str),
    missing_settings_warning: &dyn Fn(bool) -> Option<&'static str>,
) -> Option<GamePaths> {
    run_game_detection_workflow(
        find_game_path,
        detect_game_paths,
        allow_directory,
        log,
        missing_settings_warning,
    )
}

pub(crate) fn save_file_with(
    file_path: &Path,
    content: &str,
    mut log: impl FnMut(&str, &str),
    save_text_file: impl FnOnce(&Path, &str) -> Result<(), String>,
) -> Result<(), String> {
    log("INFO", &format!("Saving MXML to: {}", file_path.display()));
    save_text_file(file_path, content).map_err(|e| {
        let err = format!("Failed to write to file '{}': {}", file_path.display(), e);
        log("ERROR", &err);
        err
    })
}

pub(crate) fn resize_window_with(
    width: f64,
    current_height: u32,
    set_size: impl FnOnce(LogicalSize<f64>) -> Result<(), String>,
) -> Result<(), String> {
    set_size(LogicalSize::new(width, current_height as f64))
}

pub(crate) fn write_to_log_with(level: &str, message: &str, mut log: impl FnMut(&str, &str)) {
    log(level, message);
}

pub(crate) async fn http_request_with(
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    request: impl FnOnce(
        String,
        Option<String>,
        Option<HashMap<String, String>>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<HttpResponse, String>> + Send>,
    >,
) -> Result<HttpResponse, String> {
    request(url, method, headers).await
}

pub(crate) fn check_for_untracked_mods_command_with(
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    check_untracked_mods_for_game_path: impl Fn(Option<&Path>) -> bool,
) -> bool {
    check_for_untracked_mods_with(find_game_path(), check_untracked_mods_for_game_path)
}

pub(crate) fn run_legacy_migration_command_with(
    get_config_path: impl FnOnce() -> Result<PathBuf, String>,
    get_profiles_dir: impl FnOnce() -> Result<PathBuf, String>,
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    run_legacy_migration_in_paths: impl Fn(&Path, &Path, Option<&Path>) -> Result<bool, String>,
) -> Result<(), String> {
    let config_path = get_config_path()?;
    let profiles_dir = get_profiles_dir()?;
    let game_path = find_game_path();
    run_legacy_migration_with(
        &config_path,
        &profiles_dir,
        game_path.as_deref(),
        run_legacy_migration_in_paths,
    )
}

pub(crate) fn delete_settings_file_command_with(
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    mod_settings_file: impl Fn(&Path) -> PathBuf,
    delete_settings_at_path: impl Fn(&Path) -> Result<String, String>,
    delete_settings_without_game_path: impl Fn() -> Result<String, String>,
) -> Result<String, String> {
    delete_settings_file_with(
        find_game_path(),
        mod_settings_file,
        delete_settings_at_path,
        delete_settings_without_game_path,
    )
}

pub(crate) fn open_mods_folder_command_with(
    find_game_path: impl FnOnce() -> Option<PathBuf>,
    open_mods_folder_for_game_path: impl FnOnce(Option<&Path>) -> Result<(), String>,
) -> Result<(), String> {
    open_mods_folder_with(find_game_path(), open_mods_folder_for_game_path)
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
