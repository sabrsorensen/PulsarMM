use pulsar::installation_detection::{detect_game_paths, find_game_path};
use pulsar::linux::game_paths::find_linux_game_path_with;
use pulsar::settings_paths;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_it_game_detection_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn detects_linux_game_installation_from_steam_manifest_and_binaries_only() {
    let _ = find_game_path();

    let root = temp_test_dir("steam_manifest");
    let home = root.join("home");
    let steam_root = home.join(".steam/steam");
    let game_path = steam_root.join("steamapps/common/No Man's Sky");

    fs::create_dir_all(game_path.join("Binaries")).unwrap();
    write_file(
        &steam_root.join("steamapps/appmanifest_275850.acf"),
        r#""installdir" "No Man's Sky""#,
    );

    let home_str = home.to_string_lossy().into_owned();
    let found = find_linux_game_path_with(
        |key| match key {
            "PULSAR_NMS_PATH" => None,
            "HOME" => Some(home_str.clone()),
            _ => None,
        },
        |p| fs::read_to_string(p).ok(),
        |p| p.is_dir(),
    )
    .expect("expected game path");

    assert_eq!(found, game_path);

    let detected = detect_game_paths(&found).expect("expected detection");
    assert_eq!(detected.version_type, "Steam");
    assert!(!detected.settings_initialized);
    assert_eq!(
        detected.settings_root_path,
        game_path.to_string_lossy().into_owned()
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn marks_settings_initialized_after_game_creates_mxml() {
    let root = temp_test_dir("settings_mxml");
    let game_path = root.join("No Man's Sky");
    fs::create_dir_all(game_path.join("Binaries")).unwrap();

    let before = detect_game_paths(&game_path).expect("expected detection");
    assert!(!before.settings_initialized);

    let settings_file = settings_paths::mod_settings_file(&game_path);
    write_file(&settings_file, "<Data/>");

    let after = detect_game_paths(&game_path).expect("expected detection");
    assert!(after.settings_initialized);
    assert_eq!(
        after.game_root_path,
        game_path.to_string_lossy().into_owned()
    );

    fs::remove_dir_all(root).unwrap();
}
