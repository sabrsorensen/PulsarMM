use super::{
    detect_game_installation_with, detection_failure_message, found_game_path_message,
    missing_settings_warning, run_game_detection_workflow,
};
use crate::models::GamePaths;
use std::path::PathBuf;

fn sample_paths() -> GamePaths {
    GamePaths {
        game_root_path: "/game".to_string(),
        settings_root_path: "/settings".to_string(),
        version_type: "Steam".to_string(),
        settings_initialized: false,
    }
}

#[test]
fn detect_game_installation_with_returns_none_when_find_fails() {
    let out = detect_game_installation_with(&|| None, &|_p| Some(sample_paths()));
    assert!(out.is_none());
}

#[test]
fn detect_game_installation_with_returns_none_when_detection_fails() {
    let out = detect_game_installation_with(&|| Some(PathBuf::from("/game")), &|_p| None);
    assert!(out.is_none());
}

#[test]
fn detect_game_installation_with_returns_pair_when_successful() {
    let out =
        detect_game_installation_with(&|| Some(PathBuf::from("/game")), &|_p| Some(sample_paths()))
            .expect("expected successful detection");
    assert_eq!(out.0, PathBuf::from("/game"));
    assert_eq!(out.1.game_root_path, "/game");
}

#[test]
fn messages_format_as_expected() {
    assert_eq!(
        found_game_path_message(PathBuf::from("/game").as_path()),
        "Found game path: \"/game\""
    );
    assert_eq!(
        detection_failure_message(),
        "Game detection failed: No valid installation found."
    );
}

#[test]
fn run_game_detection_workflow_logs_and_returns_detected_paths() {
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut log = |level: &str, msg: &str| entries.push((level.to_string(), msg.to_string()));
    let out = run_game_detection_workflow(
        &|| Some(PathBuf::from("/game")),
        &|_p| Some(sample_paths()),
        &|_path| Ok(()),
        &mut log,
        &|_settings_initialized| Some("Run the game once to generate settings files."),
    )
    .expect("expected detected paths");

    assert_eq!(out.game_root_path, "/game");
    assert_eq!(
        entries[0],
        ("INFO".to_string(), "Starting Game Detection...".to_string())
    );
    assert!(entries
        .iter()
        .any(|(_lvl, msg)| msg.contains("Found game path")));
    assert!(entries
        .iter()
        .any(|(lvl, msg)| lvl == "WARN" && msg == "Run the game once to generate settings files."));
}

#[test]
fn run_game_detection_workflow_logs_fs_scope_warning_and_still_returns_paths() {
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut log = |level: &str, msg: &str| entries.push((level.to_string(), msg.to_string()));
    let out = run_game_detection_workflow(
        &|| Some(PathBuf::from("/game")),
        &|_p| Some(sample_paths()),
        &|_path| Err("scope denied".to_string()),
        &mut log,
        &|_settings_initialized| None,
    )
    .expect("expected detected paths despite scope warning");

    assert_eq!(out.game_root_path, "/game");
    assert!(entries.iter().any(|(lvl, msg)| lvl == "WARN"
        && msg.contains("Failed to expand fs scope for game path")
        && msg.contains("scope denied")));
}

#[test]
fn run_game_detection_workflow_logs_failure_when_not_detected() {
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut log = |level: &str, msg: &str| entries.push((level.to_string(), msg.to_string()));
    let out = run_game_detection_workflow(
        &|| None,
        &|_p| Some(sample_paths()),
        &|_path| Ok(()),
        &mut log,
        &|_settings_initialized| None,
    );
    assert!(out.is_none());
    assert!(entries
        .iter()
        .any(|(lvl, msg)| lvl == "WARN"
            && msg == "Game detection failed: No valid installation found."));
}

#[test]
fn missing_settings_warning_only_when_uninitialized() {
    assert!(missing_settings_warning(true).is_none());
    assert!(missing_settings_warning(false)
        .expect("warning expected")
        .contains("run the game once"));
}
