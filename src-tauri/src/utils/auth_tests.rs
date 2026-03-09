use super::{extract_api_key_from_content, load_api_key_from_file};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_auth_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn extract_api_key_accepts_valid_json_with_apikey() {
    let key = extract_api_key_from_content(r#"{"apikey":"abc123"}"#)
        .expect("valid auth json should parse");
    assert_eq!(key, "abc123");
}

#[test]
fn extract_api_key_rejects_invalid_or_missing_key() {
    assert!(extract_api_key_from_content("not-json").is_err());
    assert!(extract_api_key_from_content(r#"{"other":"x"}"#).is_err());
    assert!(extract_api_key_from_content(r#"{"apikey":123}"#).is_err());
}

#[test]
fn load_api_key_reads_from_existing_file() {
    let dir = temp_test_dir("load");
    let path = dir.join("auth.json");
    fs::write(&path, r#"{"apikey":"secret"}"#).expect("write should succeed");
    let key = load_api_key_from_file(&path).expect("auth file should load");
    assert_eq!(key, "secret");
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn load_api_key_handles_missing_and_bad_files() {
    let dir = temp_test_dir("load_err");
    let path = dir.join("auth.json");
    assert!(load_api_key_from_file(&path).is_err());

    fs::write(&path, "bad-json").expect("write should succeed");
    assert!(load_api_key_from_file(&path).is_err());
    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn load_api_key_propagates_read_error_for_directory_target() {
    let dir = temp_test_dir("load_dir_err");
    let path = dir.join("auth.json");
    fs::create_dir_all(&path).expect("create dir should succeed");

    let err = load_api_key_from_file(&path).expect_err("directory target should fail to read");
    assert!(!err.is_empty(), "expected propagated read error");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
