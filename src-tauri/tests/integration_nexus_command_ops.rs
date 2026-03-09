use pulsar::nexus::command_ops::{
    ensure_auth_file_path, linux_protocol_handler_registered, linux_register_nxm_protocol_with,
    linux_unregister_nxm_protocol_with, remove_auth_file_if_exists, save_api_key_to_auth_path,
};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

thread_local! {
    static CREATED_FLAG: RefCell<bool> = const { RefCell::new(false) };
    static REMOVED_COUNT: RefCell<usize> = const { RefCell::new(0) };
    static WRITTEN_FILE: RefCell<Option<(PathBuf, String)>> = const { RefCell::new(None) };
    static XDG_CALLED: RefCell<bool> = const { RefCell::new(false) };
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_it_nexus_command_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
    dir
}

fn reset_records() {
    CREATED_FLAG.with(|flag| *flag.borrow_mut() = false);
    REMOVED_COUNT.with(|count| *count.borrow_mut() = 0);
    WRITTEN_FILE.with(|written| *written.borrow_mut() = None);
    XDG_CALLED.with(|flag| *flag.borrow_mut() = false);
}

fn created_flag() -> bool {
    CREATED_FLAG.with(|flag| *flag.borrow())
}

fn removed_count() -> usize {
    REMOVED_COUNT.with(|count| *count.borrow())
}

fn written_file() -> Option<(PathBuf, String)> {
    WRITTEN_FILE.with(|written| written.borrow().clone())
}

fn xdg_called() -> bool {
    XDG_CALLED.with(|flag| *flag.borrow())
}

fn create_dir_all_record(path: &std::path::Path) -> Result<(), String> {
    CREATED_FLAG.with(|flag| *flag.borrow_mut() = true);
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

fn create_dir_all_fs(path: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

fn create_dir_all_err(_path: &std::path::Path) -> Result<(), String> {
    Err("create failed".to_string())
}

fn create_dir_all_should_not_be_called(_path: &std::path::Path) -> Result<(), String> {
    Err("create_dir_all should not be called".to_string())
}

fn remove_file_record(path: &std::path::Path) -> Result<(), String> {
    REMOVED_COUNT.with(|count| *count.borrow_mut() += 1);
    fs::remove_file(path).map_err(|e| e.to_string())
}

fn remove_file_err(_path: &std::path::Path) -> Result<(), String> {
    Err("remove failed".to_string())
}

fn remove_file_should_not_be_called(_path: &std::path::Path) -> Result<(), String> {
    Err("should not be called for missing desktop file".to_string())
}

fn write_file_record(path: &std::path::Path, content: &str) -> Result<(), String> {
    WRITTEN_FILE
        .with(|written| *written.borrow_mut() = Some((path.to_path_buf(), content.to_string())));
    fs::write(path, content).map_err(|e| e.to_string())
}

fn write_file_fs(path: &std::path::Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

fn write_file_err(_path: &std::path::Path, _content: &str) -> Result<(), String> {
    Err("write failed".to_string())
}

fn xdg_ok_record() -> Result<(), String> {
    XDG_CALLED.with(|flag| *flag.borrow_mut() = true);
    Ok(())
}

fn xdg_ok() -> Result<(), String> {
    Ok(())
}

fn xdg_err() -> Result<(), String> {
    Err("xdg failed".to_string())
}

#[test]
fn ensure_auth_file_path_creates_directory_when_missing() {
    reset_records();
    let dir = temp_test_dir("auth_path");
    let app_data = dir.join("app_data");
    let path =
        ensure_auth_file_path(&app_data, &create_dir_all_record).expect("path should resolve");
    assert!(created_flag());
    assert_eq!(path, app_data.join("auth.json"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_auth_file_path_skips_create_when_directory_exists() {
    let dir = temp_test_dir("auth_exists");
    let app_data = dir.join("app_data");
    fs::create_dir_all(&app_data).unwrap();

    let path = ensure_auth_file_path(&app_data, &create_dir_all_should_not_be_called)
        .expect("existing directory should not need creation");
    assert_eq!(path, app_data.join("auth.json"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_unregister_nxm_protocol_with_removes_existing_file_only() {
    reset_records();
    let dir = temp_test_dir("unregister");
    let home = dir.to_string_lossy().to_string();
    let desktop_file = PathBuf::from(&home)
        .join(".local")
        .join("share")
        .join("applications")
        .join("nxm-handler.desktop");
    fs::create_dir_all(desktop_file.parent().unwrap()).unwrap();
    fs::write(&desktop_file, "desktop").unwrap();

    linux_unregister_nxm_protocol_with(&home, &remove_file_record)
        .expect("unregister should succeed");
    assert_eq!(removed_count(), 1);
    assert!(!desktop_file.exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_unregister_nxm_protocol_with_propagates_remove_error() {
    let dir = temp_test_dir("unregister_err");
    let home = dir.to_string_lossy().to_string();
    let desktop_file = PathBuf::from(&home)
        .join(".local")
        .join("share")
        .join("applications")
        .join("nxm-handler.desktop");
    fs::create_dir_all(desktop_file.parent().unwrap()).unwrap();
    fs::write(&desktop_file, "desktop").unwrap();

    let err = linux_unregister_nxm_protocol_with(&home, &remove_file_err)
        .expect_err("remove error should bubble");
    assert_eq!(err, "remove failed");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_protocol_handler_registered_checks_expected_path() {
    let dir = temp_test_dir("is_registered");
    let home = dir.to_string_lossy().to_string();
    assert!(!linux_protocol_handler_registered(&home));

    let desktop_file = PathBuf::from(&home)
        .join(".local")
        .join("share")
        .join("applications")
        .join("nxm-handler.desktop");
    fs::create_dir_all(desktop_file.parent().unwrap()).unwrap();
    fs::write(&desktop_file, "desktop").unwrap();
    assert!(linux_protocol_handler_registered(&home));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_register_nxm_protocol_with_creates_and_writes_expected_file() {
    reset_records();
    let dir = temp_test_dir("register");
    let home = dir.to_string_lossy().to_string();

    linux_register_nxm_protocol_with(
        &home,
        "desktop-file-content",
        &create_dir_all_fs,
        &write_file_record,
        &xdg_ok_record,
    )
    .expect("register should succeed");

    let wrote_value = written_file().expect("write expected");
    assert!(wrote_value.0.ends_with("nxm-handler.desktop"));
    assert_eq!(wrote_value.1, "desktop-file-content");
    assert!(xdg_called());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_register_nxm_protocol_with_skips_create_when_apps_dir_exists() {
    let dir = temp_test_dir("register_existing_apps_dir");
    let home = dir.to_string_lossy().to_string();
    let apps_dir = dir.join(".local/share/applications");
    fs::create_dir_all(&apps_dir).unwrap();

    linux_register_nxm_protocol_with(
        &home,
        "desktop-file-content",
        &create_dir_all_should_not_be_called,
        &write_file_fs,
        &xdg_ok,
    )
    .expect("existing apps dir should skip creation");

    assert!(apps_dir.join("nxm-handler.desktop").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_and_remove_auth_file_helpers_roundtrip() {
    let dir = temp_test_dir("auth_roundtrip");
    let auth_path = dir.join("auth.json");

    save_api_key_to_auth_path(&auth_path, "abc123").expect("save should succeed");
    let saved = fs::read_to_string(&auth_path).expect("auth file should exist");
    assert!(saved.contains("abc123"));

    assert!(remove_auth_file_if_exists(&auth_path).expect("remove should succeed"));
    assert!(!auth_path.exists());
    assert!(!remove_auth_file_if_exists(&auth_path).expect("second call should be false"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_auth_file_path_propagates_create_error() {
    let dir = temp_test_dir("auth_path_err");
    let app_data = dir.join("app_data");
    let err =
        ensure_auth_file_path(&app_data, &create_dir_all_err).expect_err("expected creation error");
    assert_eq!(err, "create failed");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_register_nxm_protocol_with_propagates_xdg_error() {
    let dir = temp_test_dir("register_err");
    let home = dir.to_string_lossy().to_string();
    let err = linux_register_nxm_protocol_with(
        &home,
        "desktop",
        &create_dir_all_fs,
        &write_file_fs,
        &xdg_err,
    )
    .expect_err("expected xdg failure");
    assert_eq!(err, "xdg failed");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_register_nxm_protocol_with_propagates_create_and_write_errors() {
    let dir = temp_test_dir("register_create_write_err");
    let home = dir.to_string_lossy().to_string();

    let err = linux_register_nxm_protocol_with(
        &home,
        "desktop",
        &create_dir_all_err,
        &write_file_fs,
        &xdg_ok,
    )
    .expect_err("create_dir_all failure should bubble");
    assert_eq!(err, "create failed");

    let err = linux_register_nxm_protocol_with(
        &home,
        "desktop",
        &create_dir_all_fs,
        &write_file_err,
        &xdg_ok,
    )
    .expect_err("write failure should bubble");
    assert_eq!(err, "write failed");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn linux_unregister_nxm_protocol_with_ignores_missing_file() {
    let dir = temp_test_dir("unregister_missing");
    let home = dir.to_string_lossy().to_string();
    linux_unregister_nxm_protocol_with(&home, &remove_file_should_not_be_called)
        .expect("missing desktop file should be okay");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn remove_auth_file_if_exists_propagates_remove_error() {
    let dir = temp_test_dir("remove_err");
    let auth_path = dir.join("auth.json");
    fs::create_dir_all(&auth_path).unwrap();
    let result = remove_auth_file_if_exists(&auth_path);
    assert!(result.is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_api_key_to_auth_path_propagates_write_error() {
    let dir = temp_test_dir("save_auth_err");
    let auth_path = dir.join("auth.json");
    fs::create_dir_all(&auth_path).unwrap();

    let err = save_api_key_to_auth_path(&auth_path, "abc123")
        .expect_err("directory target should fail auth save");
    assert!(err.contains("Failed to save auth file"));

    fs::remove_dir_all(dir).unwrap();
}
