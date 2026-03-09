use super::{build_mod_info_json, has_specific_options};
use crate::models::ProfileModEntry;

fn entry_with_options(options: Option<Vec<&str>>) -> ProfileModEntry {
    ProfileModEntry {
        filename: "mod.zip".to_string(),
        mod_id: Some("1".to_string()),
        file_id: Some("2".to_string()),
        version: Some("3".to_string()),
        installed_options: options.map(|v| v.into_iter().map(str::to_string).collect()),
    }
}

#[test]
fn has_specific_options_only_true_for_non_empty_vec() {
    assert!(!has_specific_options(&entry_with_options(None)));
    assert!(!has_specific_options(&entry_with_options(Some(vec![]))));
    assert!(has_specific_options(&entry_with_options(Some(vec!["A"]))));
}

#[test]
fn build_mod_info_json_includes_expected_fields() {
    let entry = entry_with_options(Some(vec!["A", "B"]));
    let value = build_mod_info_json(&entry);
    assert_eq!(value.get("modId").and_then(|v| v.as_str()), Some("1"));
    assert_eq!(value.get("fileId").and_then(|v| v.as_str()), Some("2"));
    assert_eq!(value.get("version").and_then(|v| v.as_str()), Some("3"));
    assert_eq!(
        value.get("installSource").and_then(|v| v.as_str()),
        Some("mod.zip")
    );
}
