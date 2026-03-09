use super::*;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_install_detection_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn detect_requires_binaries_folder() {
    let dir = temp_test_dir("requires_binaries");
    let detected = detect_game_paths(&dir);
    assert!(detected.is_none());
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn detect_marks_settings_initialized_only_when_mxml_exists() {
    let dir = temp_test_dir("settings_flag");
    fs::create_dir_all(settings_paths::binaries_dir(&dir)).expect("create dir should succeed");

    let before = detect_game_paths(&dir).expect("expected detection");
    assert!(!before.settings_initialized);

    let settings_file = settings_paths::mod_settings_file(&dir);
    fs::create_dir_all(settings_file.parent().expect("parent should exist"))
        .expect("create dir should succeed");
    fs::write(&settings_file, "<Data/>").expect("write should succeed");

    let after = detect_game_paths(&dir).expect("expected detection");
    assert!(after.settings_initialized);

    #[cfg(target_os = "linux")]
    assert_eq!(after.version_type, "Steam");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn parse_steam_library_folders_extracts_paths() {
    let content = r#"
"libraryfolders"
{
    "path" "E:\\SteamLibrary"
    "0" { "path" "C:\\Program Files (x86)\\Steam" }
    "1" { "path" "D:\\SteamLibrary" }
}
"#;

    let folders = parse_steam_library_folders(content);
    assert!(folders.contains(&PathBuf::from("E:\\SteamLibrary")));
    assert!(folders.contains(&PathBuf::from("C:\\Program Files (x86)\\Steam")));
    assert!(folders.contains(&PathBuf::from("D:\\SteamLibrary")));
}

#[test]
fn parse_steam_installdir_extracts_dir_name() {
    let content = r#"
"AppState"
{
    "appid" "275850"
    "installdir" "No Man's Sky"
}
"#;
    assert_eq!(
        parse_steam_installdir(content),
        Some("No Man's Sky".to_string())
    );
}

#[test]
fn parse_steam_installdir_handles_missing_key() {
    let content = r#""AppState" { "appid" "275850" }"#;
    assert_eq!(parse_steam_installdir(content), None);
}
