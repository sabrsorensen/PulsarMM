use crate::models::DIR_LOCK;
use std::env;
use std::io;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

#[path = "archive_rar_backend.rs"]
mod rar_backend;

fn io_error_to_string(error: io::Error) -> String {
    error.to_string()
}

pub(super) fn run_rar_processing_loop(
    step: &mut dyn FnMut() -> Result<bool, String>,
    on_progress: &mut dyn FnMut(u64),
) -> Result<(), String> {
    while step()? {
        on_progress(0);
    }
    Ok(())
}

fn with_destination_current_dir<T>(
    destination: &Path,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let _guard = DIR_LOCK.lock().map_err(|e| e.to_string())?;
    let original_dir = env::current_dir().map_err(io_error_to_string)?;
    env::set_current_dir(destination).map_err(io_error_to_string)?;

    let result = action();

    let _ = env::set_current_dir(&original_dir);
    result
}

#[cfg(test)]
pub(super) fn current_dir_locked() -> Result<PathBuf, String> {
    let _guard = DIR_LOCK.lock().map_err(|e| e.to_string())?;
    env::current_dir().map_err(io_error_to_string)
}

pub(super) fn extract_rar_archive_with(
    destination: &Path,
    on_progress: &mut dyn FnMut(u64),
    run_processing: &mut dyn FnMut(&mut dyn FnMut(u64)) -> Result<(), String>,
) -> Result<(), String> {
    with_destination_current_dir(destination, || run_processing(on_progress))?;

    on_progress(100);
    Ok(())
}

pub(super) fn extract_rar_archive(
    abs_archive_path: &Path,
    destination: &Path,
    on_progress: &mut dyn FnMut(u64),
) -> Result<(), String> {
    let mut run_processing = |on_progress: &mut dyn FnMut(u64)| {
        rar_backend::process_rar_archive_entries(abs_archive_path, on_progress)
    };
    extract_rar_archive_with(destination, on_progress, &mut run_processing)
}

#[cfg(test)]
#[path = "archive_rar_runtime_tests.rs"]
mod tests;
