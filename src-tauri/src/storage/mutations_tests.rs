use super::{apply_downloads_path_change, apply_library_path_change};
use crate::utils::config::load_config_or_default;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_storage_mutations_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn apply_downloads_path_change_moves_content_and_updates_config() {
    let dir = temp_test_dir("downloads");
    let old = dir.join("old_downloads");
    let new_root = dir.join("new_root");
    let config = dir.join("config.json");

    fs::create_dir_all(old.join("nested")).expect("create nested dir should succeed");
    fs::write(old.join("nested/file.zip"), "x").expect("write file should succeed");

    let target =
        apply_downloads_path_change(&old, &new_root, &config).expect("path change should succeed");
    assert!(target.ends_with("downloads"));
    assert!(target.join("nested/file.zip").exists());
    assert!(!old.exists());

    let cfg = load_config_or_default(&config, true);
    assert_eq!(
        cfg.custom_download_path.as_deref(),
        Some(target.to_string_lossy().as_ref())
    );

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn apply_library_path_change_moves_content_and_updates_config() {
    let dir = temp_test_dir("library");
    let old = dir.join("old_library");
    let new_root = dir.join("new_root");
    let config = dir.join("config.json");

    fs::create_dir_all(old.join("mod1")).expect("create dir should succeed");
    fs::write(old.join("mod1/a.pak"), "x").expect("write file should succeed");

    let target =
        apply_library_path_change(&old, &new_root, &config).expect("path change should succeed");
    assert!(target.ends_with("Library"));
    assert!(target.join("mod1/a.pak").exists());
    assert!(!old.exists());

    let cfg = load_config_or_default(&config, true);
    assert_eq!(
        cfg.custom_library_path.as_deref(),
        Some(target.to_string_lossy().as_ref())
    );

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn apply_path_change_rejects_nested_targets() {
    let dir = temp_test_dir("nested_target");
    let old_downloads = dir.join("downloads");
    let old_library = dir.join("Library");
    let config = dir.join("config.json");

    fs::create_dir_all(&old_downloads).expect("create downloads dir should succeed");
    fs::create_dir_all(&old_library).expect("create library dir should succeed");

    let downloads_err = apply_downloads_path_change(&old_downloads, &old_downloads, &config)
        .expect_err("downloads target nested under old path should fail");
    assert!(downloads_err.contains("Cannot move the folder inside itself"));

    let library_err = apply_library_path_change(&old_library, &old_library, &config)
        .expect_err("library target nested under old path should fail");
    assert!(library_err.contains("Cannot move the folder inside itself"));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn apply_path_change_propagates_target_create_errors() {
    let dir = temp_test_dir("target_create_error");
    let old_downloads = dir.join("old_downloads");
    let old_library = dir.join("old_library");
    let config = dir.join("config.json");
    let downloads_root_file = dir.join("downloads-root-file");
    let library_root_file = dir.join("library-root-file");

    fs::create_dir_all(&old_downloads).expect("create old downloads dir should succeed");
    fs::create_dir_all(&old_library).expect("create old library dir should succeed");
    fs::write(&downloads_root_file, "not a dir").expect("write file should succeed");
    fs::write(&library_root_file, "not a dir").expect("write file should succeed");

    let downloads_err = apply_downloads_path_change(&old_downloads, &downloads_root_file, &config)
        .expect_err("downloads target create_dir_all should fail");
    assert!(!downloads_err.is_empty());

    let library_err = apply_library_path_change(&old_library, &library_root_file, &config)
        .expect_err("library target create_dir_all should fail");
    assert!(!library_err.is_empty());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn apply_path_change_propagates_move_and_config_errors() {
    let dir = temp_test_dir("move_and_config_errors");
    let downloads_config_dir = dir.join("downloads-config-dir");
    let library_config_dir = dir.join("library-config-dir");
    let bad_downloads_old = dir.join("bad-downloads-old");
    let bad_library_old = dir.join("bad-library-old");
    let downloads_root = dir.join("downloads-root");
    let library_root = dir.join("library-root");

    fs::write(&bad_downloads_old, "not a directory").expect("write file should succeed");
    fs::write(&bad_library_old, "not a directory").expect("write file should succeed");
    let move_downloads_err =
        apply_downloads_path_change(&bad_downloads_old, &downloads_root, &dir.join("cfg.json"))
            .expect_err("file source should fail downloads move");
    assert!(!move_downloads_err.is_empty());

    let move_library_err =
        apply_library_path_change(&bad_library_old, &library_root, &dir.join("cfg2.json"))
            .expect_err("file source should fail library move");
    assert!(!move_library_err.is_empty());

    fs::create_dir_all(dir.join("old_downloads")).expect("create old downloads dir should succeed");
    fs::create_dir_all(dir.join("old_library")).expect("create old library dir should succeed");
    fs::create_dir_all(&downloads_config_dir).expect("create config dir should succeed");
    fs::create_dir_all(&library_config_dir).expect("create config dir should succeed");

    let config_downloads_err = apply_downloads_path_change(
        &dir.join("old_downloads"),
        &downloads_root,
        &downloads_config_dir,
    )
    .expect_err("directory config path should fail downloads config write");
    assert!(!config_downloads_err.is_empty());

    let config_library_err =
        apply_library_path_change(&dir.join("old_library"), &library_root, &library_config_dir)
            .expect_err("directory config path should fail library config write");
    assert!(!config_library_err.is_empty());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn apply_path_change_updates_config_when_old_path_is_missing() {
    let dir = temp_test_dir("missing_old_path");
    let config = dir.join("config.json");
    let downloads_root = dir.join("downloads-root");
    let library_root = dir.join("library-root");

    let downloads_target =
        apply_downloads_path_change(&dir.join("missing-downloads"), &downloads_root, &config)
            .expect("downloads path change should succeed without old dir");
    assert!(downloads_target.ends_with("downloads"));
    assert!(downloads_target.exists());

    let library_target =
        apply_library_path_change(&dir.join("missing-library"), &library_root, &config)
            .expect("library path change should succeed without old dir");
    assert!(library_target.ends_with("Library"));
    assert!(library_target.exists());

    let cfg = load_config_or_default(&config, true);
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
