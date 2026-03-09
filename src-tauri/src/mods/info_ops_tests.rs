use super::{
    apply_mod_info_input, ensure_mod_info_file, read_mod_info_file, set_mod_id_field,
    update_mod_id_in_json_file, EnsureModInfoInput,
};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_mod_info_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn set_mod_id_field_updates_object_and_rejects_non_object() {
    let mut value = json!({"modId":"old"});
    set_mod_id_field(&mut value, "new").expect("object should accept id update");
    assert_eq!(value.get("id").and_then(|v| v.as_str()), Some("new"));

    let mut invalid = json!(["not", "an", "object"]);
    assert!(set_mod_id_field(&mut invalid, "new").is_err());
}

#[test]
fn apply_mod_info_input_sets_only_non_empty_fields_and_requires_object() {
    let mut value = json!({
        "modId": "old",
        "fileId": "keep",
        "version": "keep",
        "installSource": "old.zip"
    });
    let input = EnsureModInfoInput {
        mod_id: "".to_string(),
        file_id: "2".to_string(),
        version: "".to_string(),
        install_source: "archive.zip".to_string(),
    };

    apply_mod_info_input(&mut value, &input).expect("object should accept updates");
    assert_eq!(value.get("modId").and_then(|v| v.as_str()), Some("old"));
    assert_eq!(value.get("fileId").and_then(|v| v.as_str()), Some("2"));
    assert_eq!(value.get("version").and_then(|v| v.as_str()), Some("keep"));
    assert_eq!(
        value.get("installSource").and_then(|v| v.as_str()),
        Some("archive.zip")
    );

    let mut invalid = json!(null);
    assert!(apply_mod_info_input(&mut invalid, &input).is_err());
}

#[test]
fn update_and_read_mod_info_file_helpers_cover_roundtrip_and_errors() {
    let dir = temp_test_dir("roundtrip");
    let mod_info = dir.join("mod_info.json");
    fs::write(
        &mod_info,
        r#"{"modId":"old","fileId":"2","version":"1.0","installSource":"archive.zip"}"#,
    )
    .expect("write should succeed");

    update_mod_id_in_json_file(&mod_info, "new").expect("update should succeed");
    let parsed =
        serde_json::from_str::<Value>(&fs::read_to_string(&mod_info).expect("read should succeed"))
            .expect("json should parse");
    assert_eq!(parsed.get("id").and_then(|v| v.as_str()), Some("new"));

    let mod_dir = dir.join("folder");
    fs::create_dir_all(&mod_dir).expect("create mod dir");
    fs::write(
        mod_dir.join("mod_info.json"),
        r#"{"modId":"1","fileId":"2","version":"1.0","installSource":"archive.zip"}"#,
    )
    .expect("write should succeed");
    assert!(read_mod_info_file(&mod_dir).is_some());

    let missing_parent = dir.join("missing").join("mod_info.json");
    let input = EnsureModInfoInput {
        mod_id: "1".to_string(),
        file_id: "2".to_string(),
        version: "1.0".to_string(),
        install_source: "archive.zip".to_string(),
    };
    assert!(ensure_mod_info_file(&missing_parent, &input).is_err());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn update_mod_id_reports_missing_invalid_and_read_errors() {
    let dir = temp_test_dir("update_errors");

    let missing = dir.join("missing.json");
    let missing_err = update_mod_id_in_json_file(&missing, "new").expect_err("missing should fail");
    assert!(missing_err.contains("mod_info.json not found"));

    let invalid = dir.join("invalid.json");
    fs::write(&invalid, "{not-json").expect("write should succeed");
    let invalid_err =
        update_mod_id_in_json_file(&invalid, "new").expect_err("invalid json should fail");
    assert!(invalid_err.contains("Failed to parse mod_info.json"));

    let directory = dir.join("directory.json");
    fs::create_dir_all(&directory).expect("create directory target");
    let read_err =
        update_mod_id_in_json_file(&directory, "new").expect_err("directory read should fail");
    assert!(read_err.contains("Failed to read mod_info.json"));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn ensure_mod_info_file_creates_updates_and_rejects_invalid_existing_json() {
    let dir = temp_test_dir("ensure_create_update");
    let mod_info = dir.join("mod_info.json");
    let input = EnsureModInfoInput {
        mod_id: "10".to_string(),
        file_id: "20".to_string(),
        version: "1.2.3".to_string(),
        install_source: "archive.zip".to_string(),
    };

    ensure_mod_info_file(&mod_info, &input).expect("new file should be created");
    let created =
        serde_json::from_str::<Value>(&fs::read_to_string(&mod_info).expect("read should succeed"))
            .expect("created json should parse");
    assert_eq!(created.get("modId").and_then(|v| v.as_str()), Some("10"));
    assert_eq!(created.get("fileId").and_then(|v| v.as_str()), Some("20"));
    assert_eq!(
        created.get("version").and_then(|v| v.as_str()),
        Some("1.2.3")
    );
    assert_eq!(
        created.get("installSource").and_then(|v| v.as_str()),
        Some("archive.zip")
    );

    fs::write(&mod_info, "{broken").expect("write broken json");
    let err =
        ensure_mod_info_file(&mod_info, &input).expect_err("invalid existing json should fail");
    assert!(!err.is_empty());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn read_mod_info_file_returns_none_for_missing_or_invalid_json() {
    let dir = temp_test_dir("read_invalid");
    let missing_mod = dir.join("missing_mod");
    fs::create_dir_all(&missing_mod).expect("create missing mod dir");
    assert!(read_mod_info_file(&missing_mod).is_none());

    let invalid_mod = dir.join("invalid_mod");
    fs::create_dir_all(&invalid_mod).expect("create invalid mod dir");
    fs::write(invalid_mod.join("mod_info.json"), "{broken").expect("write broken json");
    assert!(read_mod_info_file(&invalid_mod).is_none());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
