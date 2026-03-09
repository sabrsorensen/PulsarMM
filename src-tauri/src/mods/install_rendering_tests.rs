use super::*;
use crate::models::{ModEntry, ModProperty, SettingsData, TopLevelProperty};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("pulsarmm_rendering_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn mk_mod_entry(name: &str, enabled: &str, priority: &str) -> ModEntry {
    ModEntry {
        entry_name: "Mod".to_string(),
        entry_value: "".to_string(),
        index: "0".to_string(),
        properties: vec![
            ModProperty {
                name: "Name".to_string(),
                value: Some(name.to_string()),
            },
            ModProperty {
                name: "Enabled".to_string(),
                value: Some(enabled.to_string()),
            },
            ModProperty {
                name: "ModPriority".to_string(),
                value: Some(priority.to_string()),
            },
        ],
    }
}

#[test]
fn read_real_folders_builds_uppercase_lookup() {
    let dir = temp_test_dir("real_folders");
    fs::create_dir_all(dir.join("SomeMod")).unwrap();
    fs::create_dir_all(dir.join("another")).unwrap();
    fs::write(dir.join("file.txt"), "x").unwrap();

    let (map, set) = read_real_folders(&dir);
    assert_eq!(map.get("SOMEMOD"), Some(&"SomeMod".to_string()));
    assert_eq!(map.get("ANOTHER"), Some(&"another".to_string()));
    assert!(set.contains("SOMEMOD"));
    assert!(set.contains("ANOTHER"));
    assert!(!set.contains("FILE.TXT"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_real_folders_returns_empty_for_non_directory_root() {
    let dir = temp_test_dir("real_folders_file");
    let file = dir.join("mods.txt");
    fs::write(&file, "not a directory").unwrap();

    let (map, set) = read_real_folders(&file);
    assert!(map.is_empty());
    assert!(set.is_empty());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clean_orphaned_entries_removes_missing_and_reindexes() {
    let mut root = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![TopLevelProperty {
            name: "Data".to_string(),
            value: None,
            mods: vec![
                mk_mod_entry("KeepMe", "true", "9"),
                mk_mod_entry("DropMe", "true", "10"),
            ],
        }],
    };
    let set = HashSet::from(["KEEPME".to_string()]);

    let dirty = clean_orphaned_entries(&mut root, &set);
    assert!(dirty);
    let mods = &root.properties[0].mods;
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].index, "0");
    assert_eq!(
        mods[0]
            .properties
            .iter()
            .find(|p| p.name == "ModPriority")
            .and_then(|p| p.value.clone())
            .as_deref(),
        Some("0")
    );
}

#[test]
fn clean_orphaned_entries_returns_false_when_data_missing_or_unchanged() {
    let mut no_data = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![],
    };
    assert!(!clean_orphaned_entries(
        &mut no_data,
        &HashSet::from(["ANY".to_string()])
    ));

    let mut unchanged = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![TopLevelProperty {
            name: "Data".to_string(),
            value: None,
            mods: vec![mk_mod_entry("KeepMe", "true", "3")],
        }],
    };
    assert!(!clean_orphaned_entries(
        &mut unchanged,
        &HashSet::from(["KEEPME".to_string()])
    ));
}

#[test]
fn build_mods_to_render_sorts_and_reads_local_info() {
    let dir = temp_test_dir("render");
    fs::create_dir_all(dir.join("RealName")).unwrap();
    fs::write(
        dir.join("RealName/mod_info.json"),
        r#"{"id":"fallback-id","fileId":"22","version":"1.2.3","installSource":"src.zip"}"#,
    )
    .unwrap();

    let root = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![TopLevelProperty {
            name: "Data".to_string(),
            value: None,
            mods: vec![
                mk_mod_entry("UnmappedName", "false", "5"),
                mk_mod_entry("REALNAME", "true", "1"),
            ],
        }],
    };

    let map = HashMap::from([("REALNAME".to_string(), "RealName".to_string())]);
    let rendered = build_mods_to_render(&root, &map, &dir);

    assert_eq!(rendered.len(), 2);
    assert_eq!(rendered[0].folder_name, "RealName");
    assert!(rendered[0].enabled);
    assert_eq!(
        rendered[0]
            .local_info
            .as_ref()
            .and_then(|i| i.mod_id.clone())
            .as_deref(),
        Some("fallback-id")
    );
    assert_eq!(rendered[1].folder_name, "UnmappedName");
    assert!(!rendered[1].enabled);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn build_mods_to_render_skips_nameless_entries_and_handles_missing_data() {
    let dir = temp_test_dir("render_missing");
    let root = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![TopLevelProperty {
            name: "Data".to_string(),
            value: None,
            mods: vec![
                ModEntry {
                    entry_name: "Mod".to_string(),
                    entry_value: "".to_string(),
                    index: "0".to_string(),
                    properties: vec![ModProperty {
                        name: "Enabled".to_string(),
                        value: Some("true".to_string()),
                    }],
                },
                mk_mod_entry("Named", "false", "7"),
            ],
        }],
    };

    let rendered = build_mods_to_render(&root, &HashMap::new(), &dir);
    assert_eq!(rendered.len(), 1);
    assert_eq!(rendered[0].folder_name, "Named");

    let empty = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![],
    };
    assert!(build_mods_to_render(&empty, &HashMap::new(), &dir).is_empty());

    fs::remove_dir_all(dir).unwrap();
}
