use super::*;
use crate::models::CLEAN_MXML_TEMPLATE;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parse_settings_rejects_invalid_xml() {
    let err = parse_settings("not xml").unwrap_err();
    assert!(err.contains("Failed to parse GCMODSETTINGS.MXML"));
}

#[test]
fn parse_settings_accepts_clean_template() {
    let root = parse_settings(CLEAN_MXML_TEMPLATE).unwrap();
    assert_eq!(root.template, "GcModSettings");
    assert!(root.properties.iter().any(|p| p.name == "Data"));
}

#[test]
fn to_formatted_xml_includes_xml_header() {
    let root = parse_settings(CLEAN_MXML_TEMPLATE).unwrap();
    let xml = to_formatted_xml(&root).unwrap();
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
}

#[test]
fn save_and_load_roundtrip() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("pulsarmm-mod-settings-test-{}", unique));
    fs::create_dir_all(&base).unwrap();
    let file_path = base.join("GCMODSETTINGS.MXML");

    let root = parse_settings(CLEAN_MXML_TEMPLATE).unwrap();
    save_settings_file(&file_path, &root).unwrap();
    let reloaded = load_settings_file(&file_path).unwrap();
    assert_eq!(reloaded.template, "GcModSettings");

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn load_settings_file_reports_read_errors() {
    let missing = Path::new("/definitely/missing/GCMODSETTINGS.MXML");
    let err = load_settings_file(missing).unwrap_err();
    assert!(err.contains("Failed to read GCMODSETTINGS.MXML"));
}

#[test]
fn load_settings_file_reports_parse_errors_for_existing_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("pulsarmm-mod-settings-invalid-{}", unique));
    fs::create_dir_all(&base).unwrap();
    let file_path = base.join("GCMODSETTINGS.MXML");
    fs::write(&file_path, "not xml").unwrap();

    let err = load_settings_file(&file_path).unwrap_err();
    assert!(err.contains("Failed to parse GCMODSETTINGS.MXML"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn save_settings_file_reports_write_errors() {
    let root = parse_settings(CLEAN_MXML_TEMPLATE).unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("pulsarmm-mod-settings-write-err-{}", unique));
    fs::create_dir_all(&base).unwrap();

    let err = save_settings_file(&base, &root).unwrap_err();
    assert!(err.contains("Failed to save updated GCMODSETTINGS.MXML"));

    let _ = fs::remove_dir_all(&base);
}
