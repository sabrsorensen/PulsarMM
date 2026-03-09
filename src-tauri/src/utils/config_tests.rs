use super::{default_config, load_config_or_default, parse_config_or_default, save_config};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_cfg_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn default_config_sets_expected_fields() {
    let cfg = default_config(true);
    assert!(cfg.custom_download_path.is_none());
    assert!(cfg.custom_library_path.is_none());
    assert!(cfg.legacy_migration_done);
}

#[test]
fn parse_config_or_default_parses_valid_json_and_falls_back_on_invalid() {
    let parsed = parse_config_or_default(
        r#"{"custom_download_path":"/tmp/dl","custom_library_path":"/tmp/lib","legacy_migration_done":true}"#,
        false,
    );
    assert_eq!(parsed.custom_download_path.as_deref(), Some("/tmp/dl"));
    assert_eq!(parsed.custom_library_path.as_deref(), Some("/tmp/lib"));
    assert!(parsed.legacy_migration_done);

    let fallback = parse_config_or_default("not-json", true);
    assert!(fallback.legacy_migration_done);
    assert!(fallback.custom_download_path.is_none());
    assert!(fallback.custom_library_path.is_none());
}

#[test]
fn load_config_returns_default_when_file_missing_or_invalid() {
    let dir = temp_test_dir("load_default");
    let path = dir.join("settings.json");

    let missing = load_config_or_default(&path, false);
    assert!(!missing.legacy_migration_done);

    fs::write(&path, "invalid-json").expect("write should succeed");
    let invalid = load_config_or_default(&path, true);
    assert!(invalid.legacy_migration_done);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn load_config_returns_default_when_existing_path_is_directory() {
    let dir = temp_test_dir("load_dir_default");
    let path = dir.join("settings.json");
    fs::create_dir_all(&path).expect("create dir should succeed");

    let loaded = load_config_or_default(&path, true);
    assert!(loaded.legacy_migration_done);
    assert!(loaded.custom_download_path.is_none());
    assert!(loaded.custom_library_path.is_none());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn save_and_load_config_roundtrip() {
    let dir = temp_test_dir("roundtrip");
    let path = dir.join("settings.json");

    let mut cfg = default_config(true);
    cfg.custom_download_path = Some("/tmp/downloads".to_string());
    cfg.custom_library_path = Some("/tmp/library".to_string());

    save_config(&path, &cfg).expect("save should succeed");
    let loaded = load_config_or_default(&path, false);

    assert_eq!(
        loaded.custom_download_path.as_deref(),
        Some("/tmp/downloads")
    );
    assert_eq!(loaded.custom_library_path.as_deref(), Some("/tmp/library"));
    assert!(loaded.legacy_migration_done);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn save_config_returns_error_for_unwritable_parent() {
    let dir = temp_test_dir("save_error");
    let path = dir.join("as-directory");
    fs::create_dir_all(&path).expect("create dir should succeed");

    let cfg = default_config(true);
    assert!(save_config(&path, &cfg).is_err());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
