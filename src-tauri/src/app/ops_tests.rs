use super::{
    check_untracked_mods_for_game_path, delete_settings_at_path, delete_settings_without_game_path,
    ensure_mods_dir_exists, open_mods_folder_for_game_path, save_text_file,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_app_command_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn save_text_file_writes_content() {
    let dir = temp_test_dir("save");
    let file = dir.join("x.txt");
    save_text_file(&file, "hello").expect("save should work");
    assert_eq!(
        fs::read_to_string(&file).expect("read should succeed"),
        "hello"
    );
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn save_text_file_propagates_write_errors() {
    let dir = temp_test_dir("save_err");
    let missing_parent = dir.join("missing").join("x.txt");
    let err = save_text_file(&missing_parent, "hello").expect_err("save should fail");
    assert!(err.contains("Failed to write to file"));
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_settings_helpers_cover_missing_and_success() {
    let dir = temp_test_dir("delete");
    let file = dir.join("GCMODSETTINGS.MXML");

    let missing = delete_settings_at_path(&file).expect("missing file should be ok");
    assert_eq!(missing, "alertDeleteNotFound");

    fs::write(&file, "<Data/>").expect("write should succeed");
    let deleted = delete_settings_at_path(&file).expect("delete should succeed");
    assert_eq!(deleted, "alertDeleteSuccess");
    assert!(!file.exists());

    let err = delete_settings_without_game_path().expect_err("expected missing game path error");
    assert_eq!(err, "alertDeleteError");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_settings_at_path_propagates_remove_error() {
    let dir = temp_test_dir("delete_err");
    let err = delete_settings_at_path(&dir).expect_err("delete should fail when target is a dir");
    assert!(err.contains("Failed to delete file"));
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn ensure_mods_dir_exists_builds_expected_structure() {
    let dir = temp_test_dir("mods");
    let mods = ensure_mods_dir_exists(&dir).expect("mods dir should be created");
    assert!(mods.ends_with("GAMEDATA/MODS"));
    assert!(mods.exists());
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn ensure_mods_dir_exists_propagates_create_errors() {
    let dir = temp_test_dir("mods_err");
    let game_path_file = dir.join("game_path_file");
    fs::write(&game_path_file, "not-a-dir").expect("write should succeed");
    let err = ensure_mods_dir_exists(&game_path_file).expect_err("mods dir creation should fail");
    assert!(err.contains("Could not create MODS folder"));
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn check_untracked_mods_for_game_path_handles_none_and_empty_mods() {
    assert!(!check_untracked_mods_for_game_path(None));

    let empty_dir = temp_test_dir("tracked_empty");
    let _ = ensure_mods_dir_exists(&empty_dir).expect("mods dir should exist");
    assert!(!check_untracked_mods_for_game_path(Some(&empty_dir)));
    fs::remove_dir_all(empty_dir).expect("cleanup should succeed");

    let dir = temp_test_dir("tracked");
    let mods = ensure_mods_dir_exists(&dir).expect("mods dir should exist");
    fs::create_dir_all(mods.join("NoInfoMod")).expect("create dir should succeed");
    assert!(check_untracked_mods_for_game_path(Some(&dir)));
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn open_mods_folder_for_game_path_handles_missing_and_success() {
    let err = open_mods_folder_for_game_path(None, |_p| Ok(()))
        .expect_err("expected missing game path error");
    assert_eq!(err, "Game path not found.");

    let dir = temp_test_dir("open_mods");
    let mut opened: Option<PathBuf> = None;
    open_mods_folder_for_game_path(Some(&dir), |p| {
        opened = Some(p.to_path_buf());
        Ok(())
    })
    .expect("expected open helper success");
    let opened = opened.expect("expected captured open path");
    assert!(opened.ends_with("GAMEDATA/MODS"));

    let open_err = open_mods_folder_for_game_path(Some(&dir), |_p| Err("open failed".to_string()))
        .expect_err("expected open error");
    assert!(open_err.contains("Could not open MODS folder"));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
