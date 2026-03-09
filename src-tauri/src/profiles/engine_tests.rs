use super::*;

#[test]
fn parse_install_source_ignores_missing_or_empty() {
    assert_eq!(
        parse_install_source_from_mod_info(r#"{"installSource":"archive.zip"}"#),
        Some("archive.zip".to_string())
    );
    assert_eq!(
        parse_install_source_from_mod_info(r#"{"installSource":""}"#),
        None
    );
    assert_eq!(parse_install_source_from_mod_info(r#"{"other":"x"}"#), None);
    assert_eq!(parse_install_source_from_mod_info("not-json"), None);
}

#[test]
fn parse_mod_metadata_reads_known_fields() {
    let meta =
        parse_mod_metadata_from_mod_info(r#"{"modId":"10","fileId":"20","version":"1.2.3"}"#);
    assert_eq!(
        meta,
        ModMetadata {
            mod_id: Some("10".to_string()),
            file_id: Some("20".to_string()),
            version: Some("1.2.3".to_string()),
        }
    );
}

#[test]
fn parse_mod_metadata_defaults_on_invalid_json() {
    assert_eq!(
        parse_mod_metadata_from_mod_info("{broken"),
        ModMetadata::default()
    );
}

#[test]
fn build_profile_entries_uses_first_installed_folder_metadata() {
    let mut profile_map = HashMap::new();
    add_profile_map_entry(
        &mut profile_map,
        "archive.zip".to_string(),
        "OptionA".to_string(),
    );
    add_profile_map_entry(
        &mut profile_map,
        "archive.zip".to_string(),
        "OptionB".to_string(),
    );

    let mut metadata_by_folder = HashMap::new();
    metadata_by_folder.insert(
        "OptionA".to_string(),
        ModMetadata {
            mod_id: Some("m1".to_string()),
            file_id: Some("f1".to_string()),
            version: Some("v1".to_string()),
        },
    );

    let entries = build_profile_entries(profile_map, &metadata_by_folder);
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.filename, "archive.zip");
    assert_eq!(entry.mod_id.as_deref(), Some("m1"));
    assert_eq!(entry.file_id.as_deref(), Some("f1"));
    assert_eq!(entry.version.as_deref(), Some("v1"));
    assert_eq!(
        entry.installed_options.clone().unwrap_or_default(),
        vec!["OptionA".to_string(), "OptionB".to_string()]
    );
}

#[test]
fn build_profile_entries_defaults_when_metadata_is_missing() {
    let mut profile_map = HashMap::new();
    add_profile_map_entry(
        &mut profile_map,
        "archive.zip".to_string(),
        "MissingFolder".to_string(),
    );

    let entries = build_profile_entries(profile_map, &HashMap::new());
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.filename, "archive.zip");
    assert!(entry.mod_id.is_none());
    assert!(entry.file_id.is_none());
    assert!(entry.version.is_none());
    assert_eq!(
        entry.installed_options.clone().unwrap_or_default(),
        vec!["MissingFolder".to_string()]
    );
}

#[test]
fn load_profile_for_apply_handles_default_and_missing() {
    let default_profile =
        load_profile_for_apply("Default", false, None).expect("default should load");
    assert_eq!(default_profile.name, "Default");
    assert!(default_profile.mods.is_empty());

    let err =
        load_profile_for_apply("Custom", false, None).expect_err("missing profile should fail");
    assert_eq!(err, "Profile not found");
}

#[test]
fn load_profile_for_apply_parses_json() {
    let parsed = load_profile_for_apply(
        "Custom",
        true,
        Some(
            r#"{"name":"Custom","mods":[{"filename":"a.zip","mod_id":null,"file_id":null,"version":null,"installed_options":[]}]} "#,
        ),
    )
    .expect("profile JSON should parse");
    assert_eq!(parsed.name, "Custom");
    assert_eq!(parsed.mods.len(), 1);
    assert_eq!(parsed.mods[0].filename, "a.zip");
}

#[test]
fn load_profile_for_apply_reports_invalid_json() {
    let err = load_profile_for_apply("Custom", true, Some("{broken"))
        .expect_err("invalid profile JSON should fail");
    assert!(!err.is_empty());
}
