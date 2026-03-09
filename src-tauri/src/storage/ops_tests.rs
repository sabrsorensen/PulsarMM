use super::{
    delete_archive_file_if_exists, delete_library_folder_if_exists, ensure_folder_exists,
    library_folder_path,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_storage_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn ensure_folder_exists_validates_path_presence() {
    let dir = temp_test_dir("ensure");
    ensure_folder_exists(&dir).expect("existing dir should be valid");
    let err = ensure_folder_exists(&dir.join("missing")).expect_err("missing should fail");
    assert_eq!(err, "Folder does not exist");
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_archive_file_if_exists_is_idempotent() {
    let dir = temp_test_dir("archive");
    let file = dir.join("a.zip");
    fs::write(&file, "x").expect("write should succeed");

    delete_archive_file_if_exists(&file).expect("delete should work");
    assert!(!file.exists());
    delete_archive_file_if_exists(&file).expect("delete on missing should be noop");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_archive_file_if_exists_surfaces_remove_errors() {
    let dir = temp_test_dir("archive_error");
    let folder = dir.join("archive-as-dir.zip");
    fs::create_dir_all(&folder).expect("create dir should succeed");

    let err = delete_archive_file_if_exists(&folder)
        .expect_err("directory target should fail remove_file");
    assert!(!err.is_empty(), "expected non-empty remove_file error");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_library_folder_if_exists_removes_expected_unpack_dir() {
    let dir = temp_test_dir("library");
    let target = library_folder_path(&dir, "mod.zip");
    fs::create_dir_all(&target).expect("create dir should succeed");
    fs::write(target.join("x.pak"), "x").expect("write should succeed");

    delete_library_folder_if_exists(&dir, "mod.zip").expect("delete should work");
    assert!(!target.exists());
    delete_library_folder_if_exists(&dir, "mod.zip").expect("missing delete should be noop");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_library_folder_if_exists_surfaces_remove_errors() {
    let dir = temp_test_dir("library_error");
    let target = library_folder_path(&dir, "mod.zip");
    fs::write(&target, "not a directory").expect("write should succeed");

    let err = delete_library_folder_if_exists(&dir, "mod.zip")
        .expect_err("file target should fail remove_dir_all");
    assert!(!err.is_empty(), "expected non-empty remove_dir_all error");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
