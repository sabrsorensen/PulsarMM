use crate::models::DownloadResult;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn mod_info_path_for(game_path: &Path, mod_folder_name: &str) -> PathBuf {
    game_path
        .join("GAMEDATA")
        .join("MODS")
        .join(mod_folder_name)
        .join("mod_info.json")
}

pub fn map_update_mod_id_error(mod_folder_name: &str, err: String) -> String {
    if err.contains("not found for path") {
        format!("mod_info.json not found for mod '{}'.", mod_folder_name)
    } else {
        err
    }
}

pub fn download_progress(downloaded: u64, total_size: u64) -> Option<u64> {
    if total_size == 0 {
        None
    } else {
        Some((downloaded * 100) / total_size)
    }
}

pub fn download_step_label(pct: u64) -> String {
    format!("Downloading: {}%", pct)
}

pub fn preferred_file_time(
    created: io::Result<SystemTime>,
    modified: io::Result<SystemTime>,
    now: SystemTime,
) -> SystemTime {
    created.or(modified).unwrap_or(now)
}

pub fn unix_secs_or_zero(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn metadata_created_secs(metadata: &fs::Metadata, now: SystemTime) -> u64 {
    unix_secs_or_zero(preferred_file_time(
        metadata.created(),
        metadata.modified(),
        now,
    ))
}

pub fn build_download_result(path: &Path, size: u64, created_at: u64) -> DownloadResult {
    DownloadResult {
        path: path.to_string_lossy().into_owned(),
        size,
        created_at,
    }
}

#[cfg(test)]
#[path = "command_logic_tests.rs"]
mod tests;
