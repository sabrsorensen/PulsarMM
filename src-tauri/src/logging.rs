use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

type PathProviderFn<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type TimestampFn<'a> = dyn Fn() -> String + 'a;
type AppendFn<'a> = dyn for<'b, 'c> FnMut(&'b Path, &'c str) -> Result<(), String> + 'a;
type PrintFn<'a> = dyn FnMut(&str) + 'a;

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

fn log_paths(app_data_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        app_data_dir.join("pulsar.log"),
        app_data_dir.join("pulsar-previous.log"),
        app_data_dir.join("pulsar-older.log"),
    )
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(io_error_to_string)?;
    }
    Ok(())
}

fn format_log_entry(timestamp: &str, level: &str, message: &str) -> String {
    format!("[{}] [{}] {}\n", timestamp, level, message)
}

pub(crate) fn get_log_file_path_with(
    resolve_app_data_dir: &PathProviderFn<'_>,
) -> Result<PathBuf, String> {
    let app_data = resolve_app_data_dir()?;
    ensure_dir(&app_data)?;
    Ok(log_paths(&app_data).0)
}

pub(crate) fn append_log_entry(log_path: &Path, log_entry: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(io_error_to_string)?;
    file.write_all(log_entry.as_bytes())
        .map_err(io_error_to_string)
}

pub(crate) fn rotate_logs_in_dir(app_data_dir: &Path) {
    let (log_current, log_previous, log_older) = log_paths(app_data_dir);

    if log_previous.exists() {
        let _ = std::fs::rename(&log_previous, &log_older);
    }
    if log_current.exists() {
        let _ = std::fs::rename(&log_current, &log_previous);
    }
}

pub(crate) fn log_internal_with(
    level: &str,
    message: &str,
    get_log_file_path: &PathProviderFn<'_>,
    now_timestamp: &TimestampFn<'_>,
    append_log_entry: &mut AppendFn<'_>,
    print_line: &mut PrintFn<'_>,
) {
    if let Ok(log_path) = get_log_file_path() {
        let timestamp = now_timestamp();
        let log_entry = format_log_entry(&timestamp, level, message);
        let _ = append_log_entry(&log_path, &log_entry);
    }
    print_line(&format!("[{}] {}", level, message));
}

#[cfg(test)]
#[path = "logging_tests.rs"]
mod tests;
