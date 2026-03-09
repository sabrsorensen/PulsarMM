use super::{
    clear_downloads_and_library, clear_downloads_and_library_with, open_special_folder_with,
    set_downloads_path_with, set_library_path_with,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_storage_command_flow_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn clear_downloads_and_library_with_calls_both_cleanup_functions() {
    let mut clear_files_called = false;
    let mut clear_dirs_called = false;
    let out = clear_downloads_and_library_with(
        PathBuf::from("/tmp/downloads").as_path(),
        PathBuf::from("/tmp/library").as_path(),
        |_p| {
            clear_files_called = true;
            Ok(1)
        },
        |_p| {
            clear_dirs_called = true;
            Ok(2)
        },
    );
    assert!(out.is_ok());
    assert!(clear_files_called);
    assert!(clear_dirs_called);
}

#[test]
fn clear_downloads_and_library_with_propagates_cleanup_errors() {
    let err = clear_downloads_and_library_with(
        PathBuf::from("/tmp/downloads").as_path(),
        PathBuf::from("/tmp/library").as_path(),
        |_p| Err("clear-files-failed".to_string()),
        |_p| Ok(0),
    )
    .expect_err("clear files error should bubble");
    assert_eq!(err, "clear-files-failed");

    let err = clear_downloads_and_library_with(
        PathBuf::from("/tmp/downloads").as_path(),
        PathBuf::from("/tmp/library").as_path(),
        |_p| Ok(0),
        |_p| Err("clear-dirs-failed".to_string()),
    )
    .expect_err("clear dirs error should bubble");
    assert_eq!(err, "clear-dirs-failed");
}

#[test]
fn clear_downloads_and_library_cleans_real_directories() {
    let dir = temp_test_dir("clear_real");
    let downloads = dir.join("downloads");
    let library = dir.join("library");
    fs::create_dir_all(&downloads).expect("create downloads dir should succeed");
    fs::create_dir_all(library.join("nested")).expect("create library dir should succeed");
    fs::write(downloads.join("a.zip"), "a").expect("write file should succeed");

    clear_downloads_and_library(&downloads, &library).expect("real cleanup should succeed");

    assert!(fs::read_dir(&downloads)
        .expect("downloads dir")
        .next()
        .is_none());
    assert!(fs::read_dir(&library)
        .expect("library dir")
        .next()
        .is_none());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn open_special_folder_with_resolves_and_opens_selected_path() {
    let mut opened: Option<PathBuf> = None;
    open_special_folder_with(
        "downloads",
        PathBuf::from("/tmp/downloads"),
        PathBuf::from("/tmp/profiles"),
        PathBuf::from("/tmp/library"),
        |p| {
            opened = Some(p);
            Ok(())
        },
    )
    .expect("expected special folder open success");
    assert_eq!(opened, Some(PathBuf::from("/tmp/downloads")));
}

#[test]
fn open_special_folder_with_propagates_open_and_selection_errors() {
    let err = open_special_folder_with(
        "downloads",
        PathBuf::from("/tmp/downloads"),
        PathBuf::from("/tmp/profiles"),
        PathBuf::from("/tmp/library"),
        |_p| Err("open-failed".to_string()),
    )
    .expect_err("open error should bubble");
    assert_eq!(err, "open-failed");

    let err = open_special_folder_with(
        "unknown",
        PathBuf::from("/tmp/downloads"),
        PathBuf::from("/tmp/profiles"),
        PathBuf::from("/tmp/library"),
        |_p| Ok(()),
    )
    .expect_err("unknown folder should fail");
    assert_eq!(err, "Unknown folder type");
}

#[test]
fn set_downloads_path_with_logs_and_propagates_errors() {
    let dir = temp_test_dir("downloads_err");
    let old = dir.join("old_downloads");
    let config = dir.join("config").join("config.json");
    fs::create_dir_all(&old).expect("create dir should succeed");
    let mut logs: Vec<(String, String)> = Vec::new();
    let out = set_downloads_path_with(
        old.as_path(),
        "/root/forbidden/path",
        config.as_path(),
        |lvl, msg| {
            logs.push((lvl.to_string(), msg.to_string()));
        },
    );
    assert!(out.is_err());
    assert!(logs.iter().any(|(lvl, _)| lvl == "INFO"));
    assert!(logs.iter().any(|(lvl, _)| lvl == "WARN"));
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn set_downloads_path_with_logs_success() {
    let dir = temp_test_dir("downloads_ok");
    let old = dir.join("old_downloads");
    let config = dir.join("config.json");
    let new_root = dir.join("new_root");
    fs::create_dir_all(&old).expect("create dir should succeed");
    fs::write(old.join("a.zip"), "a").expect("write file should succeed");

    let mut logs: Vec<(String, String)> = Vec::new();
    set_downloads_path_with(
        old.as_path(),
        new_root.to_string_lossy().as_ref(),
        config.as_path(),
        |lvl, msg| logs.push((lvl.to_string(), msg.to_string())),
    )
    .expect("downloads path change should succeed");

    assert!(logs
        .iter()
        .any(|(lvl, msg)| lvl == "INFO" && msg.contains("Changing Downloads Path")));
    assert!(logs
        .iter()
        .any(|(lvl, msg)| lvl == "INFO" && msg.contains("updated successfully")));
    assert!(new_root.join("downloads").exists());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn set_library_path_with_moves_when_paths_valid() {
    let dir = temp_test_dir("library_set");
    let old_root = dir.join("old");
    let old_library = old_root.join("Library");
    let new_root = dir.join("new");
    let config = dir.join("config.json");

    fs::create_dir_all(old_library.join("m1")).expect("create dir should succeed");
    fs::write(old_library.join("a.zip"), "a").expect("write file should succeed");

    let out = set_library_path_with(
        old_library.as_path(),
        new_root.to_string_lossy().as_ref(),
        config.as_path(),
    );
    assert!(out.is_ok());
    assert!(new_root.join("Library").exists());
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn set_library_path_with_propagates_errors() {
    let dir = temp_test_dir("library_set_err");
    let old_root = dir.join("old");
    let old_library = old_root.join("Library");
    let config = dir.join("config.json");
    fs::create_dir_all(old_library.join("m1")).expect("create dir should succeed");

    let err = set_library_path_with(
        old_library.as_path(),
        old_library.join("nested").to_string_lossy().as_ref(),
        config.as_path(),
    )
    .expect_err("moving library into itself should fail");
    assert!(!err.is_empty(), "expected non-empty library path error");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
