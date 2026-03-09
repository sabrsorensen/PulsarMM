use crate::models::{ModProfileData, ProfileModEntry};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModMetadata {
    pub mod_id: Option<String>,
    pub file_id: Option<String>,
    pub version: Option<String>,
}

pub fn parse_install_source_from_mod_info(content: &str) -> Option<String> {
    let json = serde_json::from_str::<Value>(content).ok()?;
    let source = json.get("installSource").and_then(|s| s.as_str())?;
    if source.is_empty() {
        None
    } else {
        Some(source.to_string())
    }
}

pub fn parse_mod_metadata_from_mod_info(content: &str) -> ModMetadata {
    let Ok(json) = serde_json::from_str::<Value>(content) else {
        return ModMetadata::default();
    };
    ModMetadata {
        mod_id: json.get("modId").and_then(|s| s.as_str()).map(String::from),
        file_id: json
            .get("fileId")
            .and_then(|s| s.as_str())
            .map(String::from),
        version: json
            .get("version")
            .and_then(|s| s.as_str())
            .map(String::from),
    }
}

pub fn add_profile_map_entry(
    profile_map: &mut HashMap<String, Vec<String>>,
    install_source: String,
    folder_name: String,
) {
    profile_map
        .entry(install_source)
        .or_default()
        .push(folder_name);
}

pub fn build_profile_entries(
    profile_map: HashMap<String, Vec<String>>,
    metadata_by_folder: &HashMap<String, ModMetadata>,
) -> Vec<ProfileModEntry> {
    let mut profile_entries = Vec::new();

    for (filename, installed_folders) in profile_map {
        let metadata = installed_folders
            .first()
            .and_then(|folder| metadata_by_folder.get(folder))
            .cloned()
            .unwrap_or_default();

        profile_entries.push(ProfileModEntry {
            filename,
            mod_id: metadata.mod_id,
            file_id: metadata.file_id,
            version: metadata.version,
            installed_options: Some(installed_folders),
        });
    }

    profile_entries
}

pub fn load_profile_for_apply(
    profile_name: &str,
    profile_json_exists: bool,
    profile_json_content: Option<&str>,
) -> Result<ModProfileData, String> {
    if profile_name == "Default" && !profile_json_exists {
        return Ok(ModProfileData {
            name: "Default".to_string(),
            mods: vec![],
        });
    }

    let content = profile_json_content.ok_or_else(|| "Profile not found".to_string())?;
    serde_json::from_str(content).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
