use super::{ensure_mod_info_in_game_path, update_mod_id_in_game_path};
use crate::mods::info_ops::{read_mod_info_file, EnsureModInfoInput};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_mod_command_info_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn make_game_mod_dir(base: &PathBuf, mod_name: &str) -> PathBuf {
    let mod_dir = base.join("GAMEDATA").join("MODS").join(mod_name);
    fs::create_dir_all(&mod_dir).expect("failed to create mod dir");
    mod_dir
}

#[test]
fn ensure_mod_info_in_game_path_writes_expected_file() {
    let game = temp_test_dir("ensure");
    make_game_mod_dir(&game, "TestMod");

    let input = EnsureModInfoInput {
        mod_id: "1".to_string(),
        file_id: "2".to_string(),
        version: "1.0.0".to_string(),
        install_source: "archive.zip".to_string(),
    };

    ensure_mod_info_in_game_path(&game, "TestMod", &input).expect("should create mod_info");

    let mod_dir = game.join("GAMEDATA").join("MODS").join("TestMod");
    let parsed = read_mod_info_file(&mod_dir).expect("expected parsed mod info");
    assert_eq!(parsed.mod_id.as_deref(), Some("1"));
    assert_eq!(parsed.file_id.as_deref(), Some("2"));
    assert_eq!(parsed.version.as_deref(), Some("1.0.0"));
    assert_eq!(parsed.install_source.as_deref(), Some("archive.zip"));

    fs::remove_dir_all(game).expect("cleanup should succeed");
}

#[test]
fn update_mod_id_in_game_path_updates_existing_mod_info() {
    let game = temp_test_dir("update");
    let mod_dir = make_game_mod_dir(&game, "TestMod");
    fs::write(
        mod_dir.join("mod_info.json"),
        r#"{"modId":"old","fileId":"2","version":"1.0"}"#,
    )
    .expect("write file should succeed");

    update_mod_id_in_game_path(&game, "TestMod", "new-id").expect("update should work");

    let value: Value = serde_json::from_str(
        &fs::read_to_string(mod_dir.join("mod_info.json")).expect("read should succeed"),
    )
    .expect("expected json");
    assert_eq!(value.get("id").and_then(|v| v.as_str()), Some("new-id"));

    fs::remove_dir_all(game).expect("cleanup should succeed");
}

#[test]
fn update_mod_id_in_game_path_maps_missing_file_error() {
    let game = temp_test_dir("update_missing");

    let err = update_mod_id_in_game_path(&game, "MissingMod", "new-id")
        .expect_err("missing mod_info should map through command-level error formatting");
    assert!(err.contains("MissingMod"));
    assert!(err.contains("mod_info.json not found"));

    fs::remove_dir_all(game).expect("cleanup should succeed");
}

#[test]
fn ensure_mod_info_in_game_path_propagates_write_error_for_missing_mod_dir() {
    let game = temp_test_dir("ensure_missing_dir");
    let input = EnsureModInfoInput {
        mod_id: "1".to_string(),
        file_id: "2".to_string(),
        version: "1.0.0".to_string(),
        install_source: "archive.zip".to_string(),
    };

    let err = ensure_mod_info_in_game_path(&game, "MissingMod", &input)
        .expect_err("missing mod directory should fail the final write");
    assert!(!err.is_empty(), "expected propagated write error");

    fs::remove_dir_all(game).expect("cleanup should succeed");
}
