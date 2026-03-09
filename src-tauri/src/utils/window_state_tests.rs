use super::{
    load_window_state, mark_window_state_maximized, maximized_state, save_position_state,
    save_window_state, should_restore_position,
};
use crate::models::WindowState;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn should_restore_position_rejects_far_negative_coordinates() {
    let good = WindowState {
        x: -9999,
        y: 100,
        maximized: false,
    };
    let bad_x = WindowState {
        x: -10000,
        y: 100,
        maximized: false,
    };
    let bad_y = WindowState {
        x: 100,
        y: -10000,
        maximized: false,
    };

    assert!(should_restore_position(&good));
    assert!(!should_restore_position(&bad_x));
    assert!(!should_restore_position(&bad_y));
}

#[test]
fn load_window_state_returns_none_for_missing_or_invalid_file() {
    let root = temp_test_dir("window_state_load");
    let missing_path = root.join("missing.json");
    assert!(load_window_state(&missing_path).is_none());

    let invalid_path = root.join("invalid.json");
    fs::write(&invalid_path, "{invalid json").expect("failed to write invalid json");
    assert!(load_window_state(&invalid_path).is_none());

    let dir_path = root.join("dir.json");
    fs::create_dir_all(&dir_path).expect("failed to create dir target");
    assert!(load_window_state(&dir_path).is_none());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn save_and_load_window_state_roundtrip() {
    let root = temp_test_dir("window_state_roundtrip");
    let state_path = root.join("window-state.json");
    let state = WindowState {
        x: 120,
        y: 340,
        maximized: true,
    };

    save_window_state(&state_path, &state).expect("save should succeed");
    let loaded = load_window_state(&state_path).expect("state should load");
    assert_eq!(loaded.x, 120);
    assert_eq!(loaded.y, 340);
    assert!(loaded.maximized);

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn save_position_state_writes_non_maximized_state() {
    let root = temp_test_dir("window_position");
    let state_path = root.join("window-state.json");

    save_position_state(&state_path, 11, 22).expect("save position should succeed");
    let loaded = load_window_state(&state_path).expect("state should load");
    assert_eq!(loaded.x, 11);
    assert_eq!(loaded.y, 22);
    assert!(!loaded.maximized);

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn save_window_state_reports_write_errors_for_directory_target() {
    let root = temp_test_dir("window_state_write_error");
    let state = WindowState {
        x: 1,
        y: 2,
        maximized: false,
    };

    let err = save_window_state(&root, &state).expect_err("directory target should fail write");
    assert!(!err.is_empty(), "expected non-empty filesystem write error");

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn maximized_state_sets_flag_without_moving_coordinates() {
    let maximized = maximized_state(WindowState {
        x: 5,
        y: 6,
        maximized: false,
    });

    assert_eq!(maximized.x, 5);
    assert_eq!(maximized.y, 6);
    assert!(maximized.maximized);
}

#[test]
fn mark_window_state_maximized_updates_existing_state_only() {
    let root = temp_test_dir("window_mark_max");
    let state_path = root.join("window-state.json");

    save_position_state(&state_path, 10, 20).expect("save position should succeed");
    mark_window_state_maximized(&state_path).expect("mark maximized should succeed");
    let loaded = load_window_state(&state_path).expect("state should load");
    assert!(loaded.maximized);
    assert_eq!(loaded.x, 10);
    assert_eq!(loaded.y, 20);

    let missing_path = root.join("missing.json");
    mark_window_state_maximized(&missing_path).expect("missing file should be no-op");
    assert!(!missing_path.exists());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}
