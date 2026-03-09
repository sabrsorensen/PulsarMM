use crate::models::{DownloadResult, InstallProgressPayload};
use crate::mods::command_logic::{
    build_download_result, download_progress, download_step_label, metadata_created_secs,
};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

fn request_start_error(error: reqwest::Error) -> String {
    format!("Failed to initiate HTTP request: {}", error)
}

pub fn ensure_success_status(status: reqwest::StatusCode) -> Result<(), String> {
    if status.is_success() {
        Ok(())
    } else {
        Err(format!("Download failed with HTTP status: {}", status))
    }
}

pub fn progress_payload(id: &str, pct: u64) -> InstallProgressPayload {
    InstallProgressPayload {
        id: id.to_string(),
        step: download_step_label(pct),
        progress: Some(pct),
    }
}

pub fn maybe_emit_download_progress_with(
    download_id: Option<&str>,
    downloaded: u64,
    total_size: u64,
    emit: &mut dyn FnMut(InstallProgressPayload),
) {
    if let (Some(id), Some(pct)) = (download_id, download_progress(downloaded, total_size)) {
        emit(progress_payload(id, pct));
    }
}

pub async fn download_archive_to_path_with(
    download_url: &str,
    final_archive_path: &Path,
    on_progress: &mut (dyn FnMut(u64, u64) + Send),
) -> Result<DownloadResult, String> {
    let mut response = reqwest::get(download_url)
        .await
        .map_err(request_start_error)?;
    ensure_success_status(response.status())?;

    let total_size = response.content_length().unwrap_or(0);
    let mut file = fs::File::create(final_archive_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).map_err(io_error_to_string)?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    let metadata = fs::metadata(final_archive_path).map_err(io_error_to_string)?;
    let file_size = metadata.len();
    let created_time = metadata_created_secs(&metadata, SystemTime::now());
    Ok(build_download_result(
        final_archive_path,
        file_size,
        created_time,
    ))
}

#[cfg(test)]
#[path = "command_download_ops_tests.rs"]
mod tests;
