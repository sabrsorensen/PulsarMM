use super::{delete_mod_and_save_settings, rename_mod_in_settings, reorder_mods_from_settings};
use crate::models::{ModEntry, ModProperty, TopLevelProperty, CLEAN_MXML_TEMPLATE};
use crate::mods::settings_store;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_mod_command_mutations_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn make_mod(name: &str, priority: &str, index: &str) -> ModEntry {
    ModEntry {
        entry_name: "Data".to_string(),
        entry_value: "GcModData.xml".to_string(),
        index: index.to_string(),
        properties: vec![
            ModProperty {
                name: "Name".to_string(),
                value: Some(name.to_string()),
            },
            ModProperty {
                name: "ModPriority".to_string(),
                value: Some(priority.to_string()),
            },
        ],
    }
}

fn root_with_mods(mods: Vec<ModEntry>) -> crate::models::SettingsData {
    let mut root = settings_store::parse_settings(CLEAN_MXML_TEMPLATE).expect("parse should work");
    if let Some(data) = root.properties.iter_mut().find(|p| p.name == "Data") {
        data.mods = mods;
    } else {
        root.properties.push(TopLevelProperty {
            name: "Data".to_string(),
            value: None,
            mods,
        });
    }
    root
}

fn names_from_xml(xml: &str) -> Vec<String> {
    let parsed = settings_store::parse_settings(xml).expect("parse should succeed");
    parsed
        .properties
        .iter()
        .find(|p| p.name == "Data")
        .map(|p| {
            p.mods
                .iter()
                .map(|m| {
                    m.properties
                        .iter()
                        .find(|p| p.name == "Name")
                        .and_then(|p| p.value.clone())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn reorder_mods_from_settings_reorders_and_formats() {
    let dir = temp_test_dir("reorder");
    let file = dir.join("GCMODSETTINGS.MXML");

    let root = root_with_mods(vec![
        make_mod("ALPHA", "0", "0"),
        make_mod("BETA", "1", "1"),
        make_mod("GAMMA", "2", "2"),
    ]);
    settings_store::save_settings_file(&file, &root).expect("save should succeed");

    let xml = reorder_mods_from_settings(&file, &["GAMMA".to_string(), "ALPHA".to_string()])
        .expect("reorder should succeed");
    let names = names_from_xml(&xml);
    assert_eq!(names[0], "GAMMA");
    assert_eq!(names[1], "ALPHA");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn rename_mod_in_settings_updates_name() {
    let dir = temp_test_dir("rename");
    let file = dir.join("GCMODSETTINGS.MXML");

    let root = root_with_mods(vec![make_mod("ALPHA", "0", "0")]);
    settings_store::save_settings_file(&file, &root).expect("save should succeed");

    let xml = rename_mod_in_settings(&file, "alpha", "new_name").expect("rename should succeed");
    let names = names_from_xml(&xml);
    assert_eq!(names[0], "NEW_NAME");

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn delete_mod_and_save_settings_removes_target() {
    let dir = temp_test_dir("delete");
    let file = dir.join("GCMODSETTINGS.MXML");

    let root = root_with_mods(vec![
        make_mod("ALPHA", "0", "0"),
        make_mod("BETA", "1", "1"),
    ]);
    settings_store::save_settings_file(&file, &root).expect("save should succeed");

    delete_mod_and_save_settings(&file, "alpha").expect("delete should succeed");
    let after = settings_store::load_settings_file(&file).expect("load should succeed");
    let names = after
        .properties
        .iter()
        .find(|p| p.name == "Data")
        .map(|p| {
            p.mods
                .iter()
                .map(|m| {
                    m.properties
                        .iter()
                        .find(|p| p.name == "Name")
                        .and_then(|p| p.value.clone())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(names, vec!["BETA".to_string()]);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
