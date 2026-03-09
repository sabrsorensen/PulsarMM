use crate::models::{LocalModInfo, ModRenderData, SettingsData};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub fn read_real_folders(mods_path: &Path) -> (HashMap<String, String>, HashSet<String>) {
    let mut real_folders_map: HashMap<String, String> = HashMap::new();
    let mut real_folders_set: HashSet<String> = HashSet::new();

    if let Ok(entries) = fs::read_dir(mods_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let real_name = entry.file_name().to_string_lossy().into_owned();
                let upper_name = real_name.to_uppercase();
                real_folders_map.insert(upper_name.clone(), real_name);
                real_folders_set.insert(upper_name);
            }
        }
    }

    (real_folders_map, real_folders_set)
}

pub fn clean_orphaned_entries(root: &mut SettingsData, real_folders_set: &HashSet<String>) -> bool {
    let Some(prop) = root.properties.iter_mut().find(|p| p.name == "Data") else {
        return false;
    };

    let original_len = prop.mods.len();
    prop.mods.retain(|entry| {
        let xml_name = entry
            .properties
            .iter()
            .find(|p| p.name == "Name")
            .and_then(|p| p.value.as_ref())
            .map(|s| s.to_uppercase())
            .unwrap_or_default();

        real_folders_set.contains(&xml_name)
    });

    if prop.mods.len() == original_len {
        return false;
    }

    for (i, mod_entry) in prop.mods.iter_mut().enumerate() {
        mod_entry.index = i.to_string();
        if let Some(priority_prop) = mod_entry
            .properties
            .iter_mut()
            .find(|p| p.name == "ModPriority")
        {
            priority_prop.value = Some(i.to_string());
        }
    }

    true
}

fn read_local_info(mods_path: &Path, folder_name: &str) -> Option<LocalModInfo> {
    let parsed = super::info_ops::read_mod_info_file(&mods_path.join(folder_name))?;
    Some(LocalModInfo {
        folder_name: folder_name.to_string(),
        mod_id: parsed.mod_id,
        file_id: parsed.file_id,
        version: parsed.version,
        install_source: parsed.install_source,
    })
}

pub fn build_mods_to_render(
    root: &SettingsData,
    real_folders_map: &HashMap<String, String>,
    mods_path: &Path,
) -> Vec<ModRenderData> {
    let mut mods_to_render = Vec::new();

    if let Some(prop) = root.properties.iter().find(|p| p.name == "Data") {
        for mod_entry in &prop.mods {
            let xml_name_prop = mod_entry
                .properties
                .iter()
                .find(|p| p.name == "Name")
                .and_then(|p| p.value.as_ref());

            if let Some(xml_name) = xml_name_prop {
                let folder_name = real_folders_map
                    .get(&xml_name.to_uppercase())
                    .cloned()
                    .unwrap_or_else(|| xml_name.clone());

                let enabled = mod_entry
                    .properties
                    .iter()
                    .find(|p| p.name == "Enabled")
                    .and_then(|p| p.value.as_ref())
                    .map(|v| v == "true")
                    .unwrap_or(false);

                let priority = mod_entry
                    .properties
                    .iter()
                    .find(|p| p.name == "ModPriority")
                    .and_then(|p| p.value.as_ref())
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(u32::MAX);

                mods_to_render.push(ModRenderData {
                    folder_name: folder_name.clone(),
                    enabled,
                    priority,
                    local_info: read_local_info(mods_path, &folder_name),
                });
            }
        }
    }

    mods_to_render.sort_by_key(|m| m.priority);
    mods_to_render
}

#[cfg(test)]
#[path = "install_rendering_tests.rs"]
mod tests;
