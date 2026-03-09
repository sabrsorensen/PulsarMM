use super::*;
use crate::models::ModProperty;
use crate::models::{TopLevelProperty, CLEAN_MXML_TEMPLATE};
use crate::mods::settings_store as mod_settings_store;

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

fn name_of(entry: &ModEntry) -> String {
    entry
        .properties
        .iter()
        .find(|p| p.name == "Name")
        .and_then(|p| p.value.clone())
        .unwrap_or_default()
}

fn priority_of(entry: &ModEntry) -> String {
    entry
        .properties
        .iter()
        .find(|p| p.name == "ModPriority")
        .and_then(|p| p.value.clone())
        .unwrap_or_default()
}

fn data_mods(root: &SettingsData) -> Vec<ModEntry> {
    root.properties
        .iter()
        .find(|p| p.name == "Data")
        .map(|p| p.mods.clone())
        .unwrap_or_default()
}

fn root_with_mods(mods: Vec<ModEntry>) -> SettingsData {
    let mut root =
        mod_settings_store::parse_settings(CLEAN_MXML_TEMPLATE).expect("parse should work");
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

#[test]
fn delete_mod_removes_and_reindexes_sorted_by_priority() {
    let mut root = root_with_mods(vec![
        make_mod("BETA", "9", "0"),
        make_mod("ALPHA", "1", "1"),
        make_mod("GAMMA", "5", "2"),
    ]);
    delete_mod_and_reindex(&mut root, "gamma");

    let mods = data_mods(&root);
    assert_eq!(mods.len(), 2);
    assert_eq!(name_of(&mods[0]), "ALPHA");
    assert_eq!(name_of(&mods[1]), "BETA");
    assert_eq!(mods[0].index, "0");
    assert_eq!(mods[1].index, "1");
}

#[test]
fn reorder_mods_updates_index_and_priority_for_ordered_entries() {
    let mut root = root_with_mods(vec![
        make_mod("ALPHA", "0", "0"),
        make_mod("BETA", "1", "1"),
        make_mod("GAMMA", "2", "2"),
    ]);
    reorder_mods(&mut root, &["GAMMA".to_string(), "ALPHA".to_string()]);

    let mods = data_mods(&root);
    assert_eq!(name_of(&mods[0]), "GAMMA");
    assert_eq!(mods[0].index, "0");
    assert_eq!(priority_of(&mods[0]), "0");

    assert_eq!(name_of(&mods[1]), "ALPHA");
    assert_eq!(mods[1].index, "1");
    assert_eq!(priority_of(&mods[1]), "1");

    assert_eq!(name_of(&mods[2]), "BETA");
}

#[test]
fn rename_mod_updates_name_to_uppercase() {
    let mut root = root_with_mods(vec![make_mod("ALPHA", "0", "0")]);
    rename_mod_in_xml(&mut root, "alpha", "newName").expect("rename should succeed");
    let mods = data_mods(&root);
    assert_eq!(name_of(&mods[0]), "NEWNAME");
}

#[test]
fn rename_mod_returns_error_when_not_found() {
    let mut root = root_with_mods(vec![make_mod("ALPHA", "0", "0")]);
    let err = rename_mod_in_xml(&mut root, "missing", "new").expect_err("missing mod should fail");
    assert!(err.contains("Could not find a mod entry"));
}

#[test]
fn delete_and_reorder_are_noops_without_data_property() {
    let mut root = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![],
    };

    delete_mod_and_reindex(&mut root, "ALPHA");
    reorder_mods(&mut root, &["ALPHA".to_string()]);

    assert!(root.properties.is_empty());
}

#[test]
fn reorder_mods_leaves_unordered_and_missing_priority_entries_intact() {
    let mut unnamed_priority = make_mod("BETA", "1", "1");
    unnamed_priority
        .properties
        .retain(|p| p.name != "ModPriority");
    let mut root = root_with_mods(vec![
        make_mod("ALPHA", "0", "0"),
        unnamed_priority.clone(),
        make_mod("GAMMA", "2", "2"),
    ]);

    reorder_mods(&mut root, &["GAMMA".to_string(), "MISSING".to_string()]);

    let mods = data_mods(&root);
    assert_eq!(name_of(&mods[0]), "GAMMA");
    assert_eq!(mods[0].index, "0");
    assert_eq!(priority_of(&mods[0]), "0");
    let remaining_names: Vec<String> = mods[1..].iter().map(name_of).collect();
    assert!(remaining_names.contains(&"ALPHA".to_string()));
    assert!(remaining_names.contains(&"BETA".to_string()));
    let beta = mods
        .iter()
        .find(|entry| name_of(entry) == "BETA")
        .expect("BETA should remain present");
    assert_eq!(priority_of(beta), "");
}

#[test]
fn rename_mod_returns_error_without_data_or_name_property() {
    let mut no_data = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![],
    };
    let err =
        rename_mod_in_xml(&mut no_data, "alpha", "new").expect_err("missing data should fail");
    assert!(err.contains("Could not find a mod entry"));

    let mut missing_name = root_with_mods(vec![make_mod("ALPHA", "0", "0")]);
    missing_name
        .properties
        .iter_mut()
        .find(|property| property.name == "Data")
        .expect("Data property should exist")
        .mods[0]
        .properties
        .retain(|p| p.name != "Name");
    let err =
        rename_mod_in_xml(&mut missing_name, "alpha", "new").expect_err("missing name should fail");
    assert!(err.contains("Could not find a mod entry"));
}
