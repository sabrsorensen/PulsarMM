use super::{
    build_profile_data_from_entries, collect_profile_map_and_metadata,
    library_folder_name_for_profile_entry, profile_paths, profile_progress_payload,
    should_extract_archive,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_profiles_apply_logic_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn profile_paths_build_expected_targets() {
    let dir = PathBuf::from("/tmp/profiles");
    let (json, mxml) = profile_paths(&dir, "Deck");
    assert_eq!(json, dir.join("Deck.json"));
    assert_eq!(mxml, dir.join("Deck.mxml"));
}

#[test]
fn library_folder_name_for_profile_entry_appends_unpack_suffix() {
    assert_eq!(
        library_folder_name_for_profile_entry("MyMod.zip"),
        "MyMod.zip_unpacked"
    );
}

#[test]
fn profile_progress_payload_sets_fields() {
    let payload = profile_progress_payload(2, 7, "A.zip".to_string(), 45);
    assert_eq!(payload.current, 2);
    assert_eq!(payload.total, 7);
    assert_eq!(payload.current_mod, "A.zip");
    assert_eq!(payload.file_progress, 45);
}

#[test]
fn collect_profile_map_and_metadata_reads_only_valid_mod_info_files() {
    let dir = temp_test_dir("collect");
    fs::create_dir_all(dir.join("ModA")).expect("create dir should succeed");
    fs::create_dir_all(dir.join("ModB")).expect("create dir should succeed");
    fs::create_dir_all(dir.join("ModC")).expect("create dir should succeed");

    fs::write(
        dir.join("ModA/mod_info.json"),
        r#"{"modId":"1","fileId":"2","installSource":"a.zip","version":"1.0"}"#,
    )
    .expect("write file should succeed");
    fs::write(dir.join("ModB/mod_info.json"), r#"{"id":"3","fileId":"4"}"#)
        .expect("write file should succeed");
    fs::write(dir.join("ModC/mod_info.json"), "not json").expect("write file should succeed");

    let (profile_map, metadata) = collect_profile_map_and_metadata(&dir);

    let mapped = profile_map.get("a.zip").cloned().unwrap_or_default();
    assert_eq!(mapped, vec!["ModA".to_string()]);

    assert!(metadata.contains_key("ModA"));
    assert!(metadata.contains_key("ModB"));
    assert!(metadata.contains_key("ModC"));

    let mod_c = metadata.get("ModC").expect("ModC metadata should exist");
    assert!(mod_c.mod_id.is_none());
    assert!(mod_c.file_id.is_none());
    assert!(mod_c.version.is_none());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn build_profile_data_from_entries_sets_name_and_entries() {
    let data = build_profile_data_from_entries("Deck", vec![]);
    assert_eq!(data.name, "Deck");
    assert!(data.mods.is_empty());
}

#[test]
fn should_extract_archive_only_when_library_missing_and_archive_exists() {
    assert!(should_extract_archive(false, true));
    assert!(!should_extract_archive(true, true));
    assert!(!should_extract_archive(false, false));
}

#[test]
fn collect_profile_map_and_metadata_skips_unreadable_mod_info_and_missing_roots() {
    let dir = temp_test_dir("collect_unreadable");
    fs::create_dir_all(dir.join("Unreadable/mod_info.json"))
        .expect("create unreadable mod_info dir should succeed");
    fs::write(dir.join("plain-file.txt"), "not a mod dir").expect("write file should succeed");

    let (profile_map, metadata) = collect_profile_map_and_metadata(&dir);
    assert!(profile_map.is_empty());
    assert!(metadata.is_empty());

    let missing = dir.join("missing");
    let (profile_map, metadata) = collect_profile_map_and_metadata(&missing);
    assert!(profile_map.is_empty());
    assert!(metadata.is_empty());

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
