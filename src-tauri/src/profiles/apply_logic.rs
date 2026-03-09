use crate::models::{ModProfileData, ProfileSwitchProgress};
use crate::profiles::engine;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn profile_paths(profiles_dir: &Path, profile_name: &str) -> (PathBuf, PathBuf) {
    (
        profiles_dir.join(format!("{}.json", profile_name)),
        profiles_dir.join(format!("{}.mxml", profile_name)),
    )
}

pub fn library_folder_name_for_profile_entry(filename: &str) -> String {
    format!("{}_unpacked", filename)
}

pub fn profile_progress_payload(
    current: usize,
    total: usize,
    current_mod: String,
    file_progress: u64,
) -> ProfileSwitchProgress {
    ProfileSwitchProgress {
        current,
        total,
        current_mod,
        file_progress,
    }
}

pub fn collect_profile_map_and_metadata(
    mods_path: &Path,
) -> (
    HashMap<String, Vec<String>>,
    HashMap<String, engine::ModMetadata>,
) {
    let mut profile_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut metadata_by_folder: HashMap<String, engine::ModMetadata> = HashMap::new();

    if let Ok(entries) = fs::read_dir(mods_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let folder_name = entry.file_name().to_string_lossy().into_owned();
                let info_path = entry.path().join("mod_info.json");

                if let Ok(content) = fs::read_to_string(&info_path) {
                    if let Some(source) = engine::parse_install_source_from_mod_info(&content) {
                        engine::add_profile_map_entry(
                            &mut profile_map,
                            source,
                            folder_name.clone(),
                        );
                    }
                    metadata_by_folder.insert(
                        folder_name.clone(),
                        engine::parse_mod_metadata_from_mod_info(&content),
                    );
                }
            }
        }
    }

    (profile_map, metadata_by_folder)
}

pub fn build_profile_data_from_entries(
    profile_name: &str,
    entries: Vec<crate::models::ProfileModEntry>,
) -> ModProfileData {
    ModProfileData {
        name: profile_name.to_string(),
        mods: entries,
    }
}

pub fn should_extract_archive(library_mod_exists: bool, archive_exists: bool) -> bool {
    !library_mod_exists && archive_exists
}

#[cfg(test)]
#[path = "apply_logic_tests.rs"]
mod tests;
