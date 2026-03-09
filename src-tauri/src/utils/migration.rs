use crate::models::ModProfileData;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(crate) type LegacyLookup = HashMap<(String, String), String>;

pub(crate) fn build_legacy_lookup<I>(profiles: I) -> LegacyLookup
where
    I: IntoIterator<Item = ModProfileData>,
{
    let mut lookup = HashMap::new();
    for profile in profiles {
        for mod_entry in profile.mods {
            if let (Some(mid), Some(fid)) = (mod_entry.mod_id, mod_entry.file_id) {
                lookup.insert((mid, fid), mod_entry.filename);
            }
        }
    }
    lookup
}

fn json_value_as_string(json: &Value, key: &str) -> Option<String> {
    match json.get(key) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

pub(crate) fn needs_install_source_heal(json: &Value) -> bool {
    json.get("installSource")
        .and_then(|s| s.as_str())
        .map(|s| s.is_empty())
        .unwrap_or(true)
}

pub(crate) fn heal_mod_info_json(json: &mut Value, lookup: &LegacyLookup) -> Option<String> {
    if !needs_install_source_heal(json) {
        return None;
    }

    let mod_id = json_value_as_string(json, "modId").or_else(|| json_value_as_string(json, "id"));
    let file_id = json_value_as_string(json, "fileId");
    let (Some(mid), Some(fid)) = (mod_id, file_id) else {
        return None;
    };

    let filename = lookup.get(&(mid, fid))?.clone();
    let obj = json.as_object_mut()?;
    obj.insert("installSource".to_string(), Value::String(filename.clone()));
    Some(filename)
}

pub(crate) fn load_profiles_from_dir(profiles_dir: &Path) -> Vec<ModProfileData> {
    let mut all_profiles = Vec::new();
    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(profile_data) = serde_json::from_str::<ModProfileData>(&content) {
                        all_profiles.push(profile_data);
                    }
                }
            }
        }
    }
    all_profiles
}

pub(crate) fn heal_mod_infos_in_dir(mods_dir: &Path, lookup: &LegacyLookup) -> usize {
    let Ok(entries) = fs::read_dir(mods_dir) else {
        return 0;
    };

    let mut healed_count = 0usize;

    for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
        let info_path = entry.path().join("mod_info.json");
        if !info_path.exists() {
            continue;
        }

        let Some(mut json) = fs::read_to_string(&info_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
        else {
            continue;
        };

        if heal_mod_info_json(&mut json, lookup).is_none() {
            continue;
        }

        let new_content =
            serde_json::to_string_pretty(&json).expect("serializing JSON value should not fail");

        if fs::write(&info_path, new_content).is_ok() {
            healed_count += 1;
        }
    }

    healed_count
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod tests;
