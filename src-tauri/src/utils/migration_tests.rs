use super::{
    build_legacy_lookup, heal_mod_info_json, heal_mod_infos_in_dir, load_profiles_from_dir,
    needs_install_source_heal,
};
use crate::models::{ModProfileData, ProfileModEntry};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("pulsarmm_migration_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn build_lookup_collects_mod_and_file_pairs() {
    let profiles = vec![
        ModProfileData {
            name: "A".to_string(),
            mods: vec![
                ProfileModEntry {
                    filename: "a.zip".to_string(),
                    mod_id: Some("1".to_string()),
                    file_id: Some("10".to_string()),
                    version: None,
                    installed_options: None,
                },
                ProfileModEntry {
                    filename: "skip.zip".to_string(),
                    mod_id: Some("2".to_string()),
                    file_id: None,
                    version: None,
                    installed_options: None,
                },
            ],
        },
        ModProfileData {
            name: "B".to_string(),
            mods: vec![ProfileModEntry {
                filename: "b.zip".to_string(),
                mod_id: Some("3".to_string()),
                file_id: Some("30".to_string()),
                version: None,
                installed_options: None,
            }],
        },
    ];

    let lookup = build_legacy_lookup(profiles);
    assert_eq!(
        lookup.get(&("1".to_string(), "10".to_string())),
        Some(&"a.zip".to_string())
    );
    assert_eq!(
        lookup.get(&("3".to_string(), "30".to_string())),
        Some(&"b.zip".to_string())
    );
    assert!(!lookup.contains_key(&("2".to_string(), "0".to_string())));
}

#[test]
fn needs_heal_requires_missing_or_empty_install_source() {
    assert!(needs_install_source_heal(
        &json!({"modId":"1","fileId":"2"})
    ));
    assert!(needs_install_source_heal(&json!({"installSource":""})));
    assert!(!needs_install_source_heal(
        &json!({"installSource":"abc.zip"})
    ));
}

#[test]
fn heal_json_inserts_install_source_from_lookup() {
    let lookup = build_legacy_lookup(vec![ModProfileData {
        name: "A".to_string(),
        mods: vec![ProfileModEntry {
            filename: "archive.zip".to_string(),
            mod_id: Some("100".to_string()),
            file_id: Some("200".to_string()),
            version: None,
            installed_options: None,
        }],
    }]);

    let mut json = json!({"modId":"100","fileId":"200"});
    let healed = heal_mod_info_json(&mut json, &lookup);
    assert_eq!(healed.as_deref(), Some("archive.zip"));
    assert_eq!(
        json.get("installSource").and_then(|v| v.as_str()),
        Some("archive.zip")
    );
}

#[test]
fn heal_json_uses_id_fallback_and_numeric_values() {
    let lookup = build_legacy_lookup(vec![ModProfileData {
        name: "A".to_string(),
        mods: vec![ProfileModEntry {
            filename: "archive.zip".to_string(),
            mod_id: Some("7".to_string()),
            file_id: Some("8".to_string()),
            version: None,
            installed_options: None,
        }],
    }]);

    let mut json = json!({"id": 7, "fileId": 8, "installSource": ""});
    let healed = heal_mod_info_json(&mut json, &lookup);
    assert_eq!(healed.as_deref(), Some("archive.zip"));
}

#[test]
fn heal_json_skips_when_already_populated_or_missing_keys() {
    let lookup = build_legacy_lookup(vec![ModProfileData {
        name: "A".to_string(),
        mods: vec![ProfileModEntry {
            filename: "archive.zip".to_string(),
            mod_id: Some("1".to_string()),
            file_id: Some("2".to_string()),
            version: None,
            installed_options: None,
        }],
    }]);

    let mut already = json!({"modId":"1","fileId":"2","installSource":"present.zip"});
    assert!(heal_mod_info_json(&mut already, &lookup).is_none());

    let mut missing = json!({"modId":"1"});
    assert!(heal_mod_info_json(&mut missing, &lookup).is_none());

    let mut non_object = json!(["not", "an", "object"]);
    assert!(heal_mod_info_json(&mut non_object, &lookup).is_none());
}

#[test]
fn load_profiles_from_dir_reads_only_valid_json_profiles() {
    let dir = temp_test_dir("load_profiles");
    fs::write(
        dir.join("p1.json"),
        r#"{"name":"p1","mods":[{"filename":"a.zip","mod_id":"1","file_id":"2","version":null,"installed_options":null}]}"#,
    )
    .expect("write profile should succeed");
    fs::write(dir.join("bad.json"), "not json").expect("write bad json should succeed");
    fs::write(dir.join("note.txt"), "x").expect("write text note should succeed");

    let profiles = load_profiles_from_dir(&dir);
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].name, "p1");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn load_profiles_from_dir_ignores_missing_and_unreadable_entries() {
    let dir = temp_test_dir("load_profiles_missing");
    fs::create_dir_all(dir.join("dir.json")).expect("create dir should succeed");

    let profiles = load_profiles_from_dir(&dir.join("missing"));
    assert!(profiles.is_empty());

    let profiles = load_profiles_from_dir(&dir);
    assert!(profiles.is_empty());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn heal_mod_infos_in_dir_updates_expected_files() {
    let dir = temp_test_dir("heal_dir");
    fs::create_dir_all(dir.join("ModA")).expect("create ModA dir should succeed");
    fs::create_dir_all(dir.join("ModB")).expect("create ModB dir should succeed");
    fs::write(
        dir.join("ModA/mod_info.json"),
        r#"{"modId":"1","fileId":"2"}"#,
    )
    .expect("write mod info should succeed");
    fs::write(
        dir.join("ModB/mod_info.json"),
        r#"{"modId":"9","fileId":"9","installSource":"keep.zip"}"#,
    )
    .expect("write mod info should succeed");

    let lookup = build_legacy_lookup(vec![ModProfileData {
        name: "A".to_string(),
        mods: vec![ProfileModEntry {
            filename: "archive.zip".to_string(),
            mod_id: Some("1".to_string()),
            file_id: Some("2".to_string()),
            version: None,
            installed_options: None,
        }],
    }]);

    let healed = heal_mod_infos_in_dir(&dir, &lookup);
    assert_eq!(healed, 1);
    let updated =
        fs::read_to_string(dir.join("ModA/mod_info.json")).expect("updated mod info should read");
    assert!(updated.contains("\"installSource\": \"archive.zip\""));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn heal_mod_infos_in_dir_ignores_missing_invalid_and_unwritable_files() {
    let dir = temp_test_dir("heal_dir_ignored");
    fs::create_dir_all(dir.join("NoInfo")).expect("create NoInfo dir should succeed");
    fs::create_dir_all(dir.join("BadJson")).expect("create BadJson dir should succeed");
    fs::create_dir_all(dir.join("DirInfo/mod_info.json"))
        .expect("create DirInfo dir should succeed");
    fs::create_dir_all(dir.join("Unwritable")).expect("create Unwritable dir should succeed");
    fs::write(dir.join("BadJson/mod_info.json"), "not json").expect("write bad mod info");
    fs::write(
        dir.join("Unwritable/mod_info.json"),
        r#"{"modId":"1","fileId":"2"}"#,
    )
    .expect("write mod info should succeed");
    fs::create_dir_all(dir.join("Unwritable/mod_info.json"))
        .expect_err("mod_info.json should remain a file");

    let lookup = build_legacy_lookup(vec![ModProfileData {
        name: "A".to_string(),
        mods: vec![ProfileModEntry {
            filename: "archive.zip".to_string(),
            mod_id: Some("1".to_string()),
            file_id: Some("2".to_string()),
            version: None,
            installed_options: None,
        }],
    }]);

    let healed = heal_mod_infos_in_dir(&dir, &lookup);
    assert_eq!(healed, 1);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn heal_mod_infos_in_dir_returns_zero_for_missing_root() {
    let dir = temp_test_dir("heal_dir_missing");
    let lookup = build_legacy_lookup(Vec::<ModProfileData>::new());

    assert_eq!(heal_mod_infos_in_dir(&dir.join("missing"), &lookup), 0);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
