use crate::logging::{
    append_log_entry, get_log_file_path_with, log_internal_with, rotate_logs_in_dir,
};
use chrono::Local;
use tauri::{AppHandle, Manager};

fn get_log_file_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    get_log_file_path_with(&|| app.path().app_data_dir().map_err(|e| e.to_string()))
}

pub(crate) fn rotate_logs(app: &AppHandle) {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(p) => p,
        Err(_) => return,
    };

    rotate_logs_in_dir(&app_data_dir);
}

pub(crate) fn log_internal(app: &AppHandle, level: &str, message: &str) {
    log_internal_with(
        level,
        message,
        &|| get_log_file_path(app),
        &|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        &mut append_log_entry,
        &mut |line| println!("{line}"),
    );
}
