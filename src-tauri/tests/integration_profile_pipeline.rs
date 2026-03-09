use pulsar::models::ProfileModEntry;
use pulsar::profiles::{apply_logic, apply_ops, engine};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_it_profile_pipeline_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn profile_pipeline_collects_entries_and_deploys_selected_options() {
    let root = temp_test_dir("selected_options");
    let game_mods = root.join("game/GAMEDATA/MODS");
    let library_mod = root.join("library/example.zip_unpacked");
    let deploy_mods = root.join("deploy/GAMEDATA/MODS");

    fs::create_dir_all(game_mods.join("FolderA")).unwrap();
    write_file(
        &game_mods.join("FolderA/mod_info.json"),
        r#"{"modId":"id-a","fileId":"file-a","version":"1.2.3","installSource":"example.zip"}"#,
    );

    fs::create_dir_all(library_mod.join("FolderA")).unwrap();
    write_file(&library_mod.join("FolderA/content.pak"), "pak");
    fs::create_dir_all(&deploy_mods).unwrap();

    let (profile_map, metadata_by_folder) =
        apply_logic::collect_profile_map_and_metadata(&game_mods);
    let entries = engine::build_profile_entries(profile_map, &metadata_by_folder);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].filename, "example.zip");
    assert_eq!(
        entries[0].installed_options,
        Some(vec!["FolderA".to_string()])
    );

    apply_ops::deploy_profile_entry(&entries[0], &library_mod, &deploy_mods).unwrap();
    assert!(deploy_mods.join("FolderA/content.pak").exists());
    assert!(deploy_mods.join("FolderA/mod_info.json").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn profile_pipeline_rewrites_copied_profile_and_restores_mxml() {
    let root = temp_test_dir("copy_restore");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).unwrap();

    write_file(
        &profiles.join("Source.json"),
        r#"{"name":"Source","mods":[{"filename":"a.zip","mod_id":"1","file_id":"2","version":"3","installed_options":["A"]}]}"#,
    );
    write_file(&profiles.join("Source.mxml"), "<Data template=\"false\"/>");

    apply_ops::copy_profile_from_dir(&profiles, "Source", "Target").unwrap();
    let copied = fs::read_to_string(profiles.join("Target.json")).unwrap();
    let copied_json: serde_json::Value = serde_json::from_str(&copied).unwrap();
    assert_eq!(copied_json["name"], "Target");
    assert_eq!(
        fs::read_to_string(profiles.join("Target.mxml")).unwrap(),
        "<Data template=\"false\"/>"
    );

    let backup_mxml = profiles.join("Target.mxml");
    let live_mxml = root.join("live/GCMODSETTINGS.MXML");
    if let Some(parent) = live_mxml.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    apply_ops::restore_or_create_live_mxml(&backup_mxml, &live_mxml).unwrap();
    assert_eq!(
        fs::read_to_string(&live_mxml).unwrap(),
        "<Data template=\"false\"/>"
    );

    fs::remove_file(&backup_mxml).unwrap();
    apply_ops::restore_or_create_live_mxml(&backup_mxml, &live_mxml).unwrap();
    assert!(fs::read_to_string(&live_mxml)
        .unwrap()
        .contains("<Data template=\"GcModSettings\">"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn profile_pipeline_build_profile_data_contract_holds() {
    let entries = vec![ProfileModEntry {
        filename: "mod-a.zip".to_string(),
        mod_id: Some("id-a".to_string()),
        file_id: Some("file-a".to_string()),
        version: Some("1.0.0".to_string()),
        installed_options: Some(vec!["FolderA".to_string()]),
    }];
    let data = apply_logic::build_profile_data_from_entries("Deck", entries.clone());
    assert_eq!(data.name, "Deck");
    assert_eq!(data.mods.len(), 1);
    assert_eq!(data.mods[0].filename, "mod-a.zip");

    let parsed = engine::parse_mod_metadata_from_mod_info(
        r#"{"modId":"id-a","fileId":"file-a","version":"1.0.0","installSource":"mod-a.zip"}"#,
    );
    assert_eq!(parsed.mod_id.as_deref(), Some("id-a"));

    let source = engine::parse_install_source_from_mod_info(
        r#"{"modId":"id-a","fileId":"file-a","version":"1.0.0","installSource":"mod-a.zip"}"#,
    );
    assert_eq!(source.as_deref(), Some("mod-a.zip"));
}
