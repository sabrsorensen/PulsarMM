use crate::installation_detection::find_game_path;
use crate::mods::archive::extract_archive;
use crate::profiles::apply::{
    apply_profile_command_entry_with, apply_profile_entries_with, load_profile_json_content_with,
    maybe_backup_live_mxml_with, save_active_profile_command_entry_with,
    write_profile_snapshot_with, ApplyProfileDeps, SaveActiveProfileDeps,
};
use crate::profiles::apply_logic::collect_profile_map_and_metadata;
use crate::profiles::engine;
use crate::profiles::storage::{
    collect_profile_names_from_dir, create_empty_profile_in_dir, delete_profile_files_in_dir,
    get_profiles_dir_with, read_profile_mod_list_from_json_path, rename_profile_files_in_dir,
};
use crate::profiles::{
    apply_ops, copy_profile_command_with, create_empty_profile_command_with,
    delete_profile_command_with, get_profile_mod_list_command_with, list_profiles_command_with,
    list_profiles_with, rename_profile_command_with,
};
use crate::{get_downloads_dir, get_library_dir, get_pulsar_root, settings_paths};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub fn get_profiles_dir(app: &AppHandle) -> Result<PathBuf, String> {
    get_profiles_dir_with(|| get_pulsar_root(app))
}

pub fn save_active_profile_impl(app: &AppHandle, profile_name: &str) -> Result<(), String> {
    let copy_file = |src: &std::path::Path, dst: &std::path::Path| {
        fs::copy(src, dst).map(|_| ()).map_err(|e| e.to_string())
    };
    let write_profile_json =
        |path: &std::path::Path, content: &str| fs::write(path, content).map_err(|e| e.to_string());

    let deps = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| get_profiles_dir(app)),
        find_game_path: Box::new(find_game_path),
        collect_profile_map_and_metadata: Box::new(collect_profile_map_and_metadata),
        mod_settings_file: Box::new(settings_paths::mod_settings_file),
        maybe_backup_live_mxml: Box::new(|current_mxml, mxml_backup_path| {
            maybe_backup_live_mxml_with(current_mxml, mxml_backup_path, &copy_file)
        }),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(|name, json_path, entries| {
            write_profile_snapshot_with(name, json_path, entries, &write_profile_json)
        }),
    };

    save_active_profile_command_entry_with(profile_name, &deps)
}

pub async fn apply_profile_impl(app: &AppHandle, profile_name: &str) -> Result<(), String> {
    let read_profile_json = |path: &std::path::Path| {
        fs::read_to_string(path).map_err(|_| "Profile not found".to_string())
    };
    let mut emit_progress = |payload| {
        app.emit("profile-progress", payload).ok();
    };
    let mut extract_archive_with_progress =
        |archive_path: &std::path::Path,
         library_mod_path: &std::path::Path,
         progress_cb: &mut dyn FnMut(u64)| {
            extract_archive(archive_path, library_mod_path, progress_cb)
        };
    let mut deploy_profile_entry = apply_ops::deploy_profile_entry;

    let mut deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| get_profiles_dir(app)),
        load_profile_json_content: Box::new(|json_path| {
            load_profile_json_content_with(json_path, &read_profile_json)
        }),
        load_profile_for_apply: Box::new(engine::load_profile_for_apply),
        find_game_path: Box::new(find_game_path),
        clear_mods_dir: Box::new(apply_ops::clear_mods_dir),
        mod_settings_file: Box::new(settings_paths::mod_settings_file),
        restore_or_create_live_mxml: Box::new(apply_ops::restore_or_create_live_mxml),
        get_downloads_dir: Box::new(|| get_downloads_dir(app)),
        get_library_dir: Box::new(|| get_library_dir(app)),
        apply_profile_entries: Box::new(|profile_data, downloads_dir, library_dir, mods_dir| {
            apply_profile_entries_with(
                profile_data,
                downloads_dir,
                library_dir,
                mods_dir,
                &mut emit_progress,
                &mut extract_archive_with_progress,
                &mut deploy_profile_entry,
            )
        }),
    };

    apply_profile_command_entry_with(profile_name, &mut deps)?;
    Ok(())
}

#[tauri::command]
pub fn list_profiles(app: AppHandle) -> Result<Vec<String>, String> {
    list_profiles_command_with(
        || get_profiles_dir(&app),
        |dir| list_profiles_with(dir, collect_profile_names_from_dir),
    )
}

#[tauri::command]
pub fn save_active_profile(app: AppHandle, profile_name: String) -> Result<(), String> {
    save_active_profile_impl(&app, &profile_name)
}

#[tauri::command]
pub async fn apply_profile(app: AppHandle, profile_name: String) -> Result<(), String> {
    apply_profile_impl(&app, &profile_name).await
}

#[tauri::command]
pub fn delete_profile(app: AppHandle, profile_name: String) -> Result<(), String> {
    delete_profile_command_with(
        &profile_name,
        || get_profiles_dir(&app),
        delete_profile_files_in_dir,
    )
}

#[tauri::command]
pub fn rename_profile(app: AppHandle, old_name: String, new_name: String) -> Result<(), String> {
    rename_profile_command_with(
        &old_name,
        &new_name,
        || get_profiles_dir(&app),
        rename_profile_files_in_dir,
    )
}

#[tauri::command]
pub fn create_empty_profile(app: AppHandle, profile_name: String) -> Result<(), String> {
    create_empty_profile_command_with(
        &profile_name,
        || get_profiles_dir(&app),
        create_empty_profile_in_dir,
    )
}

#[tauri::command]
pub fn get_profile_mod_list(app: AppHandle, profile_name: String) -> Result<Vec<String>, String> {
    get_profile_mod_list_command_with(
        &profile_name,
        || get_profiles_dir(&app),
        read_profile_mod_list_from_json_path,
    )
}

#[tauri::command]
pub fn copy_profile(app: AppHandle, source_name: String, new_name: String) -> Result<(), String> {
    copy_profile_command_with(
        &source_name,
        &new_name,
        || get_profiles_dir(&app),
        apply_ops::copy_profile_from_dir,
    )
}
