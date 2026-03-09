use super::apply_logic::{
    build_profile_data_from_entries, library_folder_name_for_profile_entry, profile_paths,
    profile_progress_payload, should_extract_archive,
};
use super::engine;
use crate::models::{ModProfileData, ProfileModEntry, ProfileSwitchProgress};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type WriteProfileJson<'a> = dyn Fn(&Path, &str) -> Result<(), String> + 'a;
type CopyFile<'a> = dyn Fn(&Path, &Path) -> Result<(), String> + 'a;
type ReadProfileJson<'a> = dyn Fn(&Path) -> Result<String, String> + 'a;
type EmitProgress<'a> = dyn FnMut(ProfileSwitchProgress) + 'a;
type ExtractArchiveWithProgress<'a> =
    dyn FnMut(&Path, &Path, &mut dyn FnMut(u64)) -> Result<(), String> + 'a;
type DeployProfileEntry<'a> = dyn FnMut(&ProfileModEntry, &Path, &Path) -> Result<(), String> + 'a;
type GetProfilesDir<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type FindGamePath<'a> = dyn Fn() -> Option<PathBuf> + 'a;
type CollectProfileMapAndMetadata<'a> = dyn Fn(
        &Path,
    ) -> (
        HashMap<String, Vec<String>>,
        HashMap<String, engine::ModMetadata>,
    ) + 'a;
type ModSettingsFile<'a> = dyn Fn(&Path) -> PathBuf + 'a;
type BackupLiveMxml<'a> = dyn Fn(&Path, &Path) -> Result<(), String> + 'a;
type BuildProfileEntries<'a> = dyn Fn(HashMap<String, Vec<String>>, &HashMap<String, engine::ModMetadata>) -> Vec<ProfileModEntry>
    + 'a;
type WriteProfileSnapshot<'a> =
    dyn Fn(&str, &Path, Vec<ProfileModEntry>) -> Result<(), String> + 'a;
type LoadProfileForApply<'a> =
    dyn Fn(&str, bool, Option<&str>) -> Result<ModProfileData, String> + 'a;
type ClearModsDir<'a> = dyn Fn(&Path) -> Result<(), String> + 'a;
type RestoreOrCreateLiveMxml<'a> = dyn Fn(&Path, &Path) -> Result<(), String> + 'a;
type GetDownloadsDir<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type GetLibraryDir<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type ApplyProfileEntries<'a> =
    dyn FnMut(&ModProfileData, &Path, &Path, &Path) -> Result<(), String> + 'a;

pub(crate) struct SaveActiveProfileDeps<'a> {
    pub get_profiles_dir: Box<GetProfilesDir<'a>>,
    pub find_game_path: Box<FindGamePath<'a>>,
    pub collect_profile_map_and_metadata: Box<CollectProfileMapAndMetadata<'a>>,
    pub mod_settings_file: Box<ModSettingsFile<'a>>,
    pub maybe_backup_live_mxml: Box<BackupLiveMxml<'a>>,
    pub build_profile_entries: Box<BuildProfileEntries<'a>>,
    pub write_profile_snapshot: Box<WriteProfileSnapshot<'a>>,
}

pub(crate) struct ApplyProfileDeps<'a> {
    pub get_profiles_dir: Box<GetProfilesDir<'a>>,
    pub load_profile_json_content: Box<dyn Fn(&Path) -> Result<Option<String>, String> + 'a>,
    pub load_profile_for_apply: Box<LoadProfileForApply<'a>>,
    pub find_game_path: Box<FindGamePath<'a>>,
    pub clear_mods_dir: Box<ClearModsDir<'a>>,
    pub mod_settings_file: Box<ModSettingsFile<'a>>,
    pub restore_or_create_live_mxml: Box<RestoreOrCreateLiveMxml<'a>>,
    pub get_downloads_dir: Box<GetDownloadsDir<'a>>,
    pub get_library_dir: Box<GetLibraryDir<'a>>,
    pub apply_profile_entries: Box<ApplyProfileEntries<'a>>,
}

fn serialize_profile_data(data: &ModProfileData) -> String {
    serde_json::to_string_pretty(data).expect("serializing ModProfileData should not fail")
}

fn write_profile_json_with(
    json_path: &Path,
    json_str: &str,
    write_file: &WriteProfileJson<'_>,
) -> Result<(), String> {
    write_file(json_path, json_str)
}

pub(crate) fn maybe_backup_live_mxml_with(
    current_mxml: &Path,
    mxml_backup_path: &Path,
    copy_file: &CopyFile<'_>,
) -> Result<(), String> {
    if current_mxml.exists() {
        copy_file(current_mxml, mxml_backup_path)?;
    }
    Ok(())
}

pub(crate) fn load_profile_json_content_with(
    json_path: &Path,
    read_file: &ReadProfileJson<'_>,
) -> Result<Option<String>, String> {
    if json_path.exists() {
        Ok(Some(read_file(json_path)?))
    } else {
        Ok(None)
    }
}

pub(crate) fn write_profile_snapshot_with(
    profile_name: &str,
    json_path: &Path,
    profile_entries: Vec<ProfileModEntry>,
    write_profile_json: &WriteProfileJson<'_>,
) -> Result<(), String> {
    let data = build_profile_data_from_entries(profile_name, profile_entries);
    let json_str = serialize_profile_data(&data);
    write_profile_json_with(json_path, &json_str, write_profile_json)
}

fn profile_entry_paths(
    downloads_dir: &Path,
    library_dir: &Path,
    filename: &str,
) -> (PathBuf, PathBuf) {
    let archive_path = downloads_dir.join(filename);
    let library_folder_name = library_folder_name_for_profile_entry(filename);
    let library_mod_path = library_dir.join(&library_folder_name);
    (archive_path, library_mod_path)
}

fn emit_entry_progress(
    emit_progress: &mut EmitProgress<'_>,
    current_idx: usize,
    total_mods: usize,
    filename: &str,
    pct: u64,
) {
    emit_progress(profile_progress_payload(
        current_idx,
        total_mods,
        filename.to_string(),
        pct,
    ));
}

fn prepare_profile_entry_with(
    entry: &ProfileModEntry,
    archive_path: &Path,
    library_mod_path: &Path,
    current_idx: usize,
    total_mods: usize,
    emit_progress: &mut EmitProgress<'_>,
    extract_archive_with_progress: &mut ExtractArchiveWithProgress<'_>,
) -> bool {
    emit_entry_progress(emit_progress, current_idx, total_mods, &entry.filename, 0);

    if !should_extract_archive(library_mod_path.exists(), archive_path.exists()) {
        return library_mod_path.exists();
    }

    let mod_name = entry.filename.clone();
    let mut progress_cb = |pct: u64| {
        emit_entry_progress(emit_progress, current_idx, total_mods, &mod_name, pct);
    };

    if let Err(e) = extract_archive_with_progress(archive_path, library_mod_path, &mut progress_cb)
    {
        println!("Failed to extract {}: {}", entry.filename, e);
        return false;
    }

    library_mod_path.exists()
}

pub(crate) fn apply_profile_entries_with(
    profile_data: &ModProfileData,
    downloads_dir: &Path,
    library_dir: &Path,
    mods_dir: &Path,
    emit_progress: &mut EmitProgress<'_>,
    extract_archive_with_progress: &mut ExtractArchiveWithProgress<'_>,
    deploy_profile_entry: &mut DeployProfileEntry<'_>,
) -> Result<(), String> {
    let total_mods = profile_data.mods.len();

    for (i, entry) in profile_data.mods.iter().enumerate() {
        let (archive_path, library_mod_path) =
            profile_entry_paths(downloads_dir, library_dir, &entry.filename);
        let current_idx = i + 1;

        if prepare_profile_entry_with(
            entry,
            &archive_path,
            &library_mod_path,
            current_idx,
            total_mods,
            emit_progress,
            extract_archive_with_progress,
        ) {
            deploy_profile_entry(entry, &library_mod_path, mods_dir)?;
            emit_entry_progress(emit_progress, current_idx, total_mods, &entry.filename, 100);
        }
    }
    Ok(())
}

fn save_active_profile_impl_with(
    profile_name: &str,
    deps: &SaveActiveProfileDeps<'_>,
) -> Result<(), String> {
    let profiles_dir = (deps.get_profiles_dir)()?;
    let (json_path, mxml_backup_path) = profile_paths(&profiles_dir, profile_name);

    if let Some(game_path) = (deps.find_game_path)() {
        let mods_path = game_path.join("GAMEDATA").join("MODS");
        let (profile_map, metadata_by_folder) = (deps.collect_profile_map_and_metadata)(&mods_path);

        let current_mxml = (deps.mod_settings_file)(&game_path);
        (deps.maybe_backup_live_mxml)(&current_mxml, &mxml_backup_path)?;

        let profile_entries = (deps.build_profile_entries)(profile_map, &metadata_by_folder);
        (deps.write_profile_snapshot)(profile_name, &json_path, profile_entries)?;
        return Ok(());
    }

    let empty_metadata: HashMap<String, engine::ModMetadata> = HashMap::new();
    let profile_entries = (deps.build_profile_entries)(HashMap::new(), &empty_metadata);
    (deps.write_profile_snapshot)(profile_name, &json_path, profile_entries)?;
    Ok(())
}

pub(crate) fn save_active_profile_command_entry_with(
    profile_name: &str,
    deps: &SaveActiveProfileDeps<'_>,
) -> Result<(), String> {
    save_active_profile_impl_with(profile_name, deps)
}

fn apply_profile_impl_with(
    profile_name: &str,
    deps: &mut ApplyProfileDeps<'_>,
) -> Result<(), String> {
    let dir = (deps.get_profiles_dir)()?;
    let (json_path, mxml_backup_path) = profile_paths(&dir, profile_name);

    let profile_json_exists = json_path.exists();
    let profile_json_content = (deps.load_profile_json_content)(&json_path)?;
    let profile_data = (deps.load_profile_for_apply)(
        profile_name,
        profile_json_exists,
        profile_json_content.as_deref(),
    )?;

    let game_path = (deps.find_game_path)().ok_or_else(|| "Game path not found".to_string())?;
    let mods_dir = game_path.join("GAMEDATA/MODS");

    (deps.clear_mods_dir)(&mods_dir)?;

    let live_mxml = (deps.mod_settings_file)(&game_path);
    println!("Applying Profile: {}", profile_name);

    (deps.restore_or_create_live_mxml)(&mxml_backup_path, &live_mxml)?;

    let downloads_dir = (deps.get_downloads_dir)()?;
    let library_dir = (deps.get_library_dir)()?;
    (deps.apply_profile_entries)(&profile_data, &downloads_dir, &library_dir, &mods_dir)?;
    Ok(())
}

pub(crate) fn apply_profile_command_entry_with(
    profile_name: &str,
    deps: &mut ApplyProfileDeps<'_>,
) -> Result<(), String> {
    apply_profile_impl_with(profile_name, deps)
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
