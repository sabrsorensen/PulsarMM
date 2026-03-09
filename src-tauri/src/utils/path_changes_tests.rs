use super::*;
use crate::utils::config::load_config_or_default;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_path_changes_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn move_directory_contents_copies_and_removes_old_tree() {
    let dir = temp_test_dir("move");
    let old = dir.join("old");
    let target = dir.join("target");
    fs::create_dir_all(old.join("nested")).expect("create dir should succeed");
    fs::write(old.join("nested/file.txt"), "x").expect("write file should succeed");

    move_directory_contents(&old, &target).expect("move should succeed");
    assert!(target.join("nested/file.txt").exists());
    assert!(!old.exists());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn move_directory_contents_creates_target_when_source_is_missing() {
    let dir = temp_test_dir("move_missing_source");
    let old = dir.join("missing-old");
    let target = dir.join("new-target");

    move_directory_contents(&old, &target).expect("missing source should be a noop");
    assert!(
        target.exists(),
        "target should be created even when source is absent"
    );
    assert!(!old.exists());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn move_directory_contents_propagates_target_create_and_copy_errors() {
    let dir = temp_test_dir("move_errors");
    let file_parent = dir.join("not-a-dir");
    fs::write(&file_parent, "x").expect("write file should succeed");

    let create_err = move_directory_contents(&dir.join("unused"), &file_parent.join("target"))
        .expect_err("non-directory parent should fail target creation");
    assert!(
        !create_err.is_empty(),
        "expected non-empty create_dir_all error"
    );

    let old_file = dir.join("old-file");
    let target = dir.join("copy-target");
    fs::write(&old_file, "not a directory").expect("write file should succeed");

    let copy_err = move_directory_contents(&old_file, &target)
        .expect_err("file source should fail recursive copy");
    assert!(!copy_err.is_empty(), "expected non-empty copy error");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn update_path_in_config_persists_new_values() {
    let dir = temp_test_dir("config");
    let config_path = dir.join("config.json");
    let downloads_target = dir.join("downloads");
    let library_target = dir.join("library");

    update_downloads_path_in_config(&config_path, &downloads_target)
        .expect("downloads config update should succeed");
    update_library_path_in_config(&config_path, &library_target)
        .expect("library config update should succeed");

    let cfg = load_config_or_default(&config_path, true);
    assert_eq!(
        cfg.custom_download_path.as_deref(),
        Some(downloads_target.to_string_lossy().as_ref())
    );
    assert_eq!(
        cfg.custom_library_path.as_deref(),
        Some(library_target.to_string_lossy().as_ref())
    );

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
