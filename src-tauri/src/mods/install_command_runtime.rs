use crate::models::{InstallProgressPayload, InstallationAnalysis};
use crate::mods::install_command_core::{
    copy_archive_to_downloads_blocking, extract_archive_if_needed, scan_library_mod_path_blocking,
};
use crate::mods::install_command_flow::{
    emit_extraction_step_with, emit_install_step_with, install_mod_from_archive_with,
};
use std::path::PathBuf;

pub async fn install_mod_from_archive_with_progress_events(
    archive_path_str: String,
    download_id: String,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String>,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    emit_event: impl Fn(InstallProgressPayload) + Clone + Send + 'static,
    finalize_installation: impl FnOnce(
        String,
        Vec<String>,
        bool,
    ) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    let emit_progress = {
        let id = download_id.clone();
        let emit = emit_event.clone();
        move |step: &str| emit_install_step_with(&id, step, &emit)
    };

    let extract_progress_callback = {
        let id = download_id;
        let emit = emit_event;
        move |pct: u64| emit_extraction_step_with(&id, pct, &emit)
    };

    install_mod_from_archive_with(
        archive_path_str,
        get_downloads_dir,
        get_library_dir,
        emit_progress,
        extract_progress_callback,
        finalize_installation,
    )
    .await
}

pub async fn copy_archive_to_downloads_async(
    archive_path: PathBuf,
    downloads_dir: PathBuf,
) -> Result<PathBuf, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<PathBuf, String> {
        copy_archive_to_downloads_blocking(archive_path, downloads_dir)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn extract_archive_if_needed_async(
    archive_path: PathBuf,
    library_mod_path: PathBuf,
    mut progress_callback: Box<dyn FnMut(u64) + Send + 'static>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        extract_archive_if_needed(archive_path, library_mod_path, &mut *progress_callback)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn scan_library_mod_path_async(
    library_mod_path: PathBuf,
) -> Result<(Vec<String>, Vec<String>), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(Vec<String>, Vec<String>), String> {
        scan_library_mod_path_blocking(library_mod_path)
    })
    .await
    .map_err(|e| e.to_string())?
}
