use super::{
    app_data_file_path_with, ensure_dir, get_pulsar_root_with, resolve_custom_path_with,
    storage_dir_from_custom_or_default, storage_dir_with,
};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("pulsarmm_app_paths_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn storage_dir_prefers_custom_path() {
    let root = Path::new("/tmp/root");
    let resolved =
        storage_dir_from_custom_or_default(Some("/custom/location".to_string()), root, "downloads");
    assert_eq!(resolved, PathBuf::from("/custom/location"));
}

#[test]
fn storage_dir_uses_default_leaf_when_custom_missing() {
    let root = Path::new("/tmp/root");
    let resolved = storage_dir_from_custom_or_default(None, root, "Library");
    assert_eq!(resolved, PathBuf::from("/tmp/root/Library"));
}

#[test]
fn ensure_dir_creates_missing_directory() {
    let parent = temp_test_dir("ensure_dir");
    let missing = parent.join("nested").join("target");
    assert!(!missing.exists());

    ensure_dir(&missing).unwrap();
    assert!(missing.exists());
    assert!(missing.is_dir());

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn ensure_dir_is_noop_for_existing_directory() {
    let dir = temp_test_dir("ensure_existing");
    ensure_dir(&dir).unwrap();
    assert!(dir.exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn app_data_file_path_with_creates_directory_and_appends_file() {
    let app_data = temp_test_dir("app_data_file_path").join("data");
    assert!(!app_data.exists());

    let path = app_data_file_path_with(&|| Ok(app_data.clone()), "config.json")
        .expect("expected file path");

    assert_eq!(path, app_data.join("config.json"));
    assert!(app_data.exists());
    fs::remove_dir_all(app_data.parent().expect("parent exists")).unwrap();
}

#[test]
fn app_data_file_path_with_propagates_provider_errors() {
    let result = app_data_file_path_with(&|| Err("no app data".to_string()), "state.json");
    assert_eq!(result.unwrap_err(), "no app data");
}

#[test]
fn app_data_file_path_with_propagates_ensure_dir_errors() {
    let base = temp_test_dir("app_data_ensure_err");
    let parent_file = base.join("parent-file");
    fs::write(&parent_file, "x").expect("create parent file");
    let bad_app_data = parent_file.join("app-data");

    let err = app_data_file_path_with(&|| Ok(bad_app_data.clone()), "state.json")
        .expect_err("ensure_dir failure should bubble");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn storage_dir_with_creates_default_and_custom_roots() {
    let base = temp_test_dir("storage_dir_with");
    let default_root = base.join("root");
    let custom = base.join("custom").to_string_lossy().to_string();

    let default_dir =
        storage_dir_with(None, &|| Ok(default_root.clone()), "Library").expect("default path");
    assert_eq!(default_dir, default_root.join("Library"));
    assert!(default_dir.exists());

    let custom_dir = storage_dir_with(
        Some(custom.clone()),
        &|| Ok(default_root.clone()),
        "Library",
    )
    .expect("custom");
    assert_eq!(custom_dir, PathBuf::from(custom));
    assert!(custom_dir.exists());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn storage_dir_with_propagates_root_errors() {
    let result = storage_dir_with(None, &|| Err("no root".to_string()), "downloads");
    assert_eq!(result.unwrap_err(), "no root");
}

#[test]
fn storage_dir_with_propagates_ensure_dir_errors() {
    let base = temp_test_dir("storage_dir_ensure_err");
    let parent_file = base.join("parent-file");
    fs::write(&parent_file, "x").expect("create parent file");
    let bad_custom = parent_file
        .join("custom-storage")
        .to_string_lossy()
        .to_string();

    let err = storage_dir_with(Some(bad_custom), &|| Ok(base.join("root")), "downloads")
        .expect_err("ensure_dir failure should bubble");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn ensure_dir_is_noop_for_existing_file_path() {
    let base = temp_test_dir("ensure_dir_file_error");
    let file_path = base.join("not-a-dir");
    fs::write(&file_path, "x").expect("should create file");

    ensure_dir(&file_path).expect("existing path is treated as already ensured");
    assert!(file_path.is_file());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn get_pulsar_root_with_creates_directory_and_propagates_errors() {
    let base = temp_test_dir("pulsar_root");
    let root = base.join("Pulsar");
    assert!(!root.exists());

    let out = get_pulsar_root_with(&|| Ok(root.clone())).expect("expected pulsar root");
    assert_eq!(out, root);
    assert!(out.exists());

    let err = get_pulsar_root_with(&|| Err("resolve failed".to_string()))
        .expect_err("expected resolve error");
    assert_eq!(err, "resolve failed");

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn get_pulsar_root_with_propagates_ensure_dir_errors() {
    let base = temp_test_dir("pulsar_root_ensure_err");
    let parent_file = base.join("parent-file");
    fs::write(&parent_file, "x").expect("create parent file");
    let bad_root = parent_file.join("Pulsar");

    let err = get_pulsar_root_with(&|| Ok(bad_root.clone()))
        .expect_err("ensure_dir failure should bubble");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn resolve_custom_path_with_handles_present_missing_and_path_errors() {
    let cfg = PathBuf::from("/tmp/config.json");
    let resolved = resolve_custom_path_with(&|| Ok(cfg.clone()), &|path| {
        assert_eq!(path, &cfg);
        Some("/tmp/custom".to_string())
    });
    assert_eq!(resolved.as_deref(), Some("/tmp/custom"));

    let none = resolve_custom_path_with(&|| Ok(cfg.clone()), &|_path| None);
    assert_eq!(none, None);

    let on_err = resolve_custom_path_with(&|| Err("no config".to_string()), &|_path| {
        Some("/tmp/custom".to_string())
    });
    assert_eq!(on_err, None);
}

#[test]
fn ensure_dir_propagates_create_errors_when_parent_is_file() {
    let base = temp_test_dir("ensure_dir_parent_file");
    let parent_file = base.join("parent-file");
    fs::write(&parent_file, "x").expect("create parent file");
    let missing_child = parent_file.join("child");

    let err = ensure_dir(&missing_child).expect_err("expected create_dir_all error");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}
