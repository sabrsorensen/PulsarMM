use super::{
    append_log_entry, ensure_dir, format_log_entry, get_log_file_path_with, log_internal_with,
    log_paths, rotate_logs_in_dir,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_logging_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn log_paths_build_expected_filenames() {
    let base = PathBuf::from("/tmp/pulsar");
    let (current, previous, older) = log_paths(&base);
    assert_eq!(current, base.join("pulsar.log"));
    assert_eq!(previous, base.join("pulsar-previous.log"));
    assert_eq!(older, base.join("pulsar-older.log"));
}

#[test]
fn ensure_dir_creates_missing_directory() {
    let base = temp_test_dir("ensure").join("nested");
    assert!(!base.exists());
    ensure_dir(&base).expect("dir should be created");
    assert!(base.exists());
    fs::remove_dir_all(base.parent().expect("parent must exist")).expect("cleanup");
}

#[test]
fn ensure_dir_is_noop_for_existing_directory() {
    let base = temp_test_dir("ensure_existing");
    ensure_dir(&base).expect("existing dir should be accepted");
    assert!(base.exists());
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn format_log_entry_includes_timestamp_level_and_message() {
    let entry = format_log_entry("2026-01-01 00:00:00", "INFO", "hello");
    assert_eq!(entry, "[2026-01-01 00:00:00] [INFO] hello\n");
}

#[test]
fn rotate_logs_in_dir_rolls_current_and_previous() {
    let dir = temp_test_dir("rotate");
    fs::write(dir.join("pulsar.log"), "current").expect("write current");
    fs::write(dir.join("pulsar-previous.log"), "previous").expect("write previous");

    rotate_logs_in_dir(&dir);

    assert!(!dir.join("pulsar.log").exists());
    assert!(dir.join("pulsar-previous.log").exists());
    assert!(dir.join("pulsar-older.log").exists());

    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn get_log_file_path_with_creates_directory_and_resolves_log_path() {
    let base_parent = temp_test_dir("path_with");
    let app_data = base_parent.join("app-data");
    let out = get_log_file_path_with(&|| Ok(app_data.clone())).expect("log path should resolve");
    assert_eq!(out, app_data.join("pulsar.log"));
    assert!(app_data.exists());
    fs::remove_dir_all(base_parent).expect("cleanup");
}

#[test]
fn append_log_entry_writes_and_errors_for_non_file_target() {
    let root = temp_test_dir("append");
    let log_path = root.join("pulsar.log");
    append_log_entry(&log_path, "line1\n").expect("append should succeed");
    append_log_entry(&log_path, "line2\n").expect("append should succeed");
    let content = fs::read_to_string(&log_path).expect("log should be readable");
    assert_eq!(content, "line1\nline2\n");

    let err = append_log_entry(&root, "bad").expect_err("directory path should fail");
    assert!(!err.is_empty());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn log_internal_with_prints_even_when_path_or_append_fails() {
    let printed = Mutex::new(Vec::<String>::new());
    let appended = Mutex::new(Vec::<(PathBuf, String)>::new());

    log_internal_with(
        "INFO",
        "hello",
        &|| Ok(PathBuf::from("/tmp/pulsar.log")),
        &|| "2026-01-01 00:00:00".to_string(),
        &mut |path, entry| {
            appended
                .lock()
                .expect("appended lock")
                .push((path.to_path_buf(), entry.to_string()));
            Ok(())
        },
        &mut |line| printed.lock().expect("printed lock").push(line.to_string()),
    );
    assert_eq!(printed.lock().expect("printed lock")[0], "[INFO] hello");
    let appended_guard = appended.lock().expect("appended lock");
    assert_eq!(appended_guard.len(), 1);
    assert_eq!(appended_guard[0].0, PathBuf::from("/tmp/pulsar.log"));
    assert_eq!(appended_guard[0].1, "[2026-01-01 00:00:00] [INFO] hello\n");
    drop(appended_guard);

    printed.lock().expect("printed lock").clear();
    log_internal_with(
        "WARN",
        "no-path",
        &|| Err("no app data".to_string()),
        &|| "unused".to_string(),
        &mut |_path, _entry| Ok(()),
        &mut |line| printed.lock().expect("printed lock").push(line.to_string()),
    );
    assert_eq!(printed.lock().expect("printed lock")[0], "[WARN] no-path");

    printed.lock().expect("printed lock").clear();
    log_internal_with(
        "ERROR",
        "append-failed",
        &|| Ok(PathBuf::from("/tmp/pulsar.log")),
        &|| "2026-01-01 00:00:00".to_string(),
        &mut |_path, _entry| Err("append error".to_string()),
        &mut |line| printed.lock().expect("printed lock").push(line.to_string()),
    );
    assert_eq!(
        printed.lock().expect("printed lock")[0],
        "[ERROR] append-failed"
    );
}
