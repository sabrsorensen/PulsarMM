use super::{
    build_download_result, download_progress, download_step_label, map_update_mod_id_error,
    metadata_created_secs, mod_info_path_for, preferred_file_time, unix_secs_or_zero,
};
use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_mod_command_logic_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn mod_info_path_for_builds_expected_path() {
    let path = mod_info_path_for(Path::new("/game"), "MyMod");
    assert!(path.ends_with("GAMEDATA/MODS/MyMod/mod_info.json"));
}

#[test]
fn map_update_mod_id_error_rewrites_not_found_message() {
    let msg = map_update_mod_id_error("TestMod", "file not found for path x".to_string());
    assert_eq!(msg, "mod_info.json not found for mod 'TestMod'.");
}

#[test]
fn map_update_mod_id_error_passthrough_for_other_errors() {
    let msg = map_update_mod_id_error("TestMod", "permission denied".to_string());
    assert_eq!(msg, "permission denied");
}

#[test]
fn download_progress_handles_zero_and_computes_percent() {
    assert_eq!(download_progress(50, 0), None);
    assert_eq!(download_progress(50, 200), Some(25));
    assert_eq!(download_progress(200, 200), Some(100));
}

#[test]
fn download_step_label_formats_expected_text() {
    assert_eq!(download_step_label(42), "Downloading: 42%");
}

#[test]
fn metadata_created_secs_returns_valid_unix_time() {
    let dir = temp_test_dir("metadata_time");
    let file_path = dir.join("archive.zip");
    fs::write(&file_path, "data").expect("write should succeed");
    let metadata = fs::metadata(&file_path).expect("metadata should resolve");

    let secs = metadata_created_secs(&metadata, UNIX_EPOCH);
    assert!(secs > 0);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn preferred_file_time_uses_created_then_modified_then_now() {
    let created = UNIX_EPOCH + Duration::from_secs(10);
    let modified = UNIX_EPOCH + Duration::from_secs(20);
    let now = UNIX_EPOCH + Duration::from_secs(30);

    assert_eq!(preferred_file_time(Ok(created), Ok(modified), now), created);
    assert_eq!(
        preferred_file_time(Err(io::Error::other("created")), Ok(modified), now),
        modified
    );
    assert_eq!(
        preferred_file_time(
            Err(io::Error::other("created")),
            Err(io::Error::other("modified")),
            now
        ),
        now
    );
}

#[test]
fn unix_secs_or_zero_handles_pre_unix_epoch() {
    assert_eq!(unix_secs_or_zero(UNIX_EPOCH + Duration::from_secs(7)), 7);
    assert_eq!(unix_secs_or_zero(UNIX_EPOCH - Duration::from_secs(1)), 0);
}

#[test]
fn build_download_result_maps_fields() {
    let path = Path::new("/tmp/archive.zip");
    let out = build_download_result(path, 123, 456);
    assert_eq!(out.path, "/tmp/archive.zip");
    assert_eq!(out.size, 123);
    assert_eq!(out.created_at, 456);
}
