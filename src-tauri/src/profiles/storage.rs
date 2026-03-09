use crate::models::{ModProfileData, CLEAN_MXML_TEMPLATE};
use std::fs;
use std::path::{Path, PathBuf};

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

fn json_error_to_string(error: serde_json::Error) -> String {
    error.to_string()
}

fn empty_profile_json(profile_name: &str) -> String {
    let empty_data = ModProfileData {
        name: profile_name.to_string(),
        mods: Vec::new(),
    };
    serde_json::to_string_pretty(&empty_data)
        .expect("serializing empty ModProfileData should not fail")
}

pub(crate) fn get_profiles_dir_with(
    get_pulsar_root: impl FnOnce() -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    let root = get_pulsar_root()?;
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).map_err(io_error_to_string)?;
    Ok(profiles)
}

pub fn collect_profile_names_from_dir(dir: &Path) -> Vec<String> {
    let mut profiles = vec!["Default".to_string()];
    let mut others: Vec<String> = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(profile_name_from_entry)
        .collect();

    others.sort();
    profiles.extend(others);
    profiles
}

fn profile_name_from_entry(entry: fs::DirEntry) -> Option<String> {
    let path = entry.path();
    if path.extension().unwrap_or_default() != "json" {
        return None;
    }

    let name = path.file_stem()?.to_string_lossy().into_owned();
    (name != "Default").then_some(name)
}

pub fn delete_profile_files_in_dir(dir: &Path, profile_name: &str) -> Result<(), String> {
    let json_path = dir.join(format!("{}.json", profile_name));
    let mxml_path = dir.join(format!("{}.mxml", profile_name));
    if json_path.exists() {
        fs::remove_file(json_path).map_err(io_error_to_string)?;
    }
    if mxml_path.exists() {
        fs::remove_file(mxml_path).map_err(io_error_to_string)?;
    }
    Ok(())
}

pub fn rename_profile_files_in_dir(
    dir: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<(), String> {
    let old_json = dir.join(format!("{}.json", old_name));
    let old_mxml = dir.join(format!("{}.mxml", old_name));
    let new_json = dir.join(format!("{}.json", new_name));
    let new_mxml = dir.join(format!("{}.mxml", new_name));

    if old_json.exists() {
        fs::rename(old_json, new_json).map_err(io_error_to_string)?;
    }
    if old_mxml.exists() {
        fs::rename(old_mxml, new_mxml).map_err(io_error_to_string)?;
    }
    Ok(())
}

pub fn create_empty_profile_in_dir(dir: &Path, profile_name: &str) -> Result<(), String> {
    let json_path = dir.join(format!("{}.json", profile_name));
    let mxml_path = dir.join(format!("{}.mxml", profile_name));

    if json_path.exists() {
        return Err("Profile already exists".to_string());
    }

    let json_str = empty_profile_json(profile_name);
    fs::write(&json_path, json_str).map_err(io_error_to_string)?;
    fs::write(&mxml_path, CLEAN_MXML_TEMPLATE).map_err(io_error_to_string)?;
    Ok(())
}

pub fn read_profile_mod_list_from_json_path(json_path: &Path) -> Result<Vec<String>, String> {
    if !json_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(json_path).map_err(io_error_to_string)?;
    let data: ModProfileData = serde_json::from_str(&content).map_err(json_error_to_string)?;
    Ok(data.mods.iter().map(|m| m.filename.clone()).collect())
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
