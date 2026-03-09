use super::*;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_tracking_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn detects_missing_mod_info_as_untracked() {
    let dir = temp_test_dir("missing_info");
    fs::create_dir_all(dir.join("ModA")).expect("create dir should succeed");
    assert!(has_untracked_mods_in_dir(&dir));
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn detects_invalid_or_empty_install_source() {
    let dir = temp_test_dir("invalid");
    fs::create_dir_all(dir.join("ModA")).expect("create dir should succeed");
    fs::write(dir.join("ModA/mod_info.json"), r#"{"installSource":""}"#)
        .expect("write file should succeed");
    assert!(has_untracked_mods_in_dir(&dir));

    fs::write(
        dir.join("ModA/mod_info.json"),
        r#"{"installSource":"ok.zip"}"#,
    )
    .expect("write file should succeed");
    assert!(!has_untracked_mods_in_dir(&dir));

    fs::write(dir.join("ModA/mod_info.json"), "not json").expect("write file should succeed");
    assert!(has_untracked_mods_in_dir(&dir));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn detects_unreadable_mod_info_and_ignores_non_directory_roots() {
    let dir = temp_test_dir("unreadable");
    fs::create_dir_all(dir.join("ModA/mod_info.json")).expect("create mod_info dir should succeed");
    assert!(has_untracked_mods_in_dir(&dir));
    fs::remove_dir_all(&dir).expect("cleanup should succeed");

    let file_root = temp_test_dir("file_root").join("mods.txt");
    fs::write(&file_root, "not a directory").expect("write file should succeed");
    assert!(!has_untracked_mods_in_dir(&file_root));
    fs::remove_file(&file_root).expect("cleanup file should succeed");
}
