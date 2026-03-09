use super::{
    clean_staging_dir, clear_dirs_in_dir, clear_files_in_dir, library_folder_name,
    linux_show_in_folder_target, select_special_folder_path,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_storage_logic_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn select_special_folder_path_resolves_known_keys() {
    let downloads = PathBuf::from("/tmp/downloads");
    let profiles = PathBuf::from("/tmp/profiles");
    let library = PathBuf::from("/tmp/library");

    assert_eq!(
        select_special_folder_path(
            "downloads",
            downloads.clone(),
            profiles.clone(),
            library.clone()
        )
        .expect("downloads should resolve"),
        downloads
    );
    assert_eq!(
        select_special_folder_path(
            "profiles",
            downloads.clone(),
            profiles.clone(),
            library.clone()
        )
        .expect("profiles should resolve"),
        profiles
    );
    assert_eq!(
        select_special_folder_path("library", downloads, profiles, library.clone())
            .expect("library should resolve"),
        library
    );
}

#[test]
fn select_special_folder_path_rejects_unknown_key() {
    let err = select_special_folder_path(
        "unknown",
        PathBuf::from("/tmp/downloads"),
        PathBuf::from("/tmp/profiles"),
        PathBuf::from("/tmp/library"),
    )
    .expect_err("unknown key should fail");
    assert!(err.contains("Unknown folder type"));
}

#[test]
fn library_folder_name_appends_unpack_suffix() {
    assert_eq!(library_folder_name("mod.zip"), "mod.zip_unpacked");
}

#[test]
fn linux_show_in_folder_target_prefers_parent_when_available() {
    let path = PathBuf::from("/tmp/mods/archive.zip");
    assert_eq!(
        linux_show_in_folder_target(&path),
        PathBuf::from("/tmp/mods")
    );

    let root = PathBuf::from("/");
    assert_eq!(linux_show_in_folder_target(&root), PathBuf::from("/"));
}

#[test]
fn clear_files_in_dir_removes_only_files() {
    let dir = temp_test_dir("clear_files");
    fs::write(dir.join("a.zip"), "a").unwrap();
    fs::write(dir.join("b.zip"), "b").unwrap();
    fs::create_dir_all(dir.join("keep_dir")).unwrap();

    let removed = clear_files_in_dir(&dir).unwrap();
    assert_eq!(removed, 2);
    assert!(!dir.join("a.zip").exists());
    assert!(!dir.join("b.zip").exists());
    assert!(dir.join("keep_dir").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_files_in_dir_errors_when_target_is_file() {
    let dir = temp_test_dir("clear_files_not_dir");
    let file_path = dir.join("not-a-dir");
    fs::write(&file_path, "file").unwrap();

    let err = clear_files_in_dir(&file_path).expect_err("file target should fail read_dir");
    assert!(!err.is_empty(), "expected non-empty filesystem error");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_files_in_dir_returns_zero_for_missing_directory() {
    let dir = temp_test_dir("clear_files_missing");
    let missing = dir.join("missing");

    assert_eq!(clear_files_in_dir(&missing).unwrap(), 0);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_dirs_in_dir_removes_only_directories() {
    let dir = temp_test_dir("clear_dirs");
    fs::create_dir_all(dir.join("old1")).unwrap();
    fs::create_dir_all(dir.join("old2")).unwrap();
    fs::write(dir.join("keep.zip"), "x").unwrap();

    let removed = clear_dirs_in_dir(&dir).unwrap();
    assert_eq!(removed, 2);
    assert!(!dir.join("old1").exists());
    assert!(!dir.join("old2").exists());
    assert!(dir.join("keep.zip").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_dirs_in_dir_errors_when_target_is_file() {
    let dir = temp_test_dir("clear_dirs_not_dir");
    let file_path = dir.join("not-a-dir");
    fs::write(&file_path, "file").unwrap();

    let err = clear_dirs_in_dir(&file_path).expect_err("file target should fail read_dir");
    assert!(!err.is_empty(), "expected non-empty filesystem error");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_dirs_in_dir_returns_zero_for_missing_directory() {
    let dir = temp_test_dir("clear_dirs_missing");
    let missing = dir.join("missing");

    assert_eq!(clear_dirs_in_dir(&missing).unwrap(), 0);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clean_staging_dir_recreates_nonempty_directory_and_returns_count() {
    let dir = temp_test_dir("staging_clean");
    fs::write(dir.join("a.txt"), "a").unwrap();
    fs::create_dir_all(dir.join("nested")).unwrap();

    let count = clean_staging_dir(&dir).unwrap();
    assert_eq!(count, 2);
    let remaining = fs::read_dir(&dir).unwrap().count();
    assert_eq!(remaining, 0);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clean_staging_dir_returns_zero_for_missing_or_empty_dir() {
    let missing = temp_test_dir("staging_missing").join("does-not-exist");
    assert_eq!(clean_staging_dir(&missing).unwrap(), 0);

    let empty = temp_test_dir("staging_empty");
    assert_eq!(clean_staging_dir(&empty).unwrap(), 0);
    fs::remove_dir_all(empty).unwrap();
}

#[test]
fn clean_staging_dir_errors_when_target_is_file() {
    let dir = temp_test_dir("staging_not_dir");
    let file_path = dir.join("not-a-dir");
    fs::write(&file_path, "file").unwrap();

    let err = clean_staging_dir(&file_path).expect_err("file target should fail read_dir");
    assert!(!err.is_empty(), "expected non-empty filesystem error");

    fs::remove_dir_all(dir).unwrap();
}
