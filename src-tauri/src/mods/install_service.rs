use crate::models::{InstallationAnalysis, ModRenderData, SettingsData};
use crate::mods::install_execution::{apply_planned_install_actions, collect_installed_mods_by_id};
use crate::mods::install_finalize_flow::{
    build_deploy_candidates_with, conflict_staging_path, is_scan_all_selection,
};
use crate::mods::install_planning::{plan_install_actions, DeployCandidate};
use crate::mods::install_rendering::{
    build_mods_to_render, clean_orphaned_entries, read_real_folders,
};
use crate::mods::install_scan::{build_deploy_ops, select_items_to_process};
use crate::mods::settings_store as mod_settings_store;
use crate::read_mod_info;
use crate::services::runtime::AppRuntime;
use crate::settings_paths;
use std::fs;
use std::path::{Path, PathBuf};

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

fn missing_game_installation_path_error() -> String {
    "Could not find game installation path.".to_string()
}

fn missing_game_path_error() -> String {
    "Could not find game path.".to_string()
}

fn create_staging_dir(root: &Path) -> Result<PathBuf, String> {
    let staging = root.join("staging");
    fs::create_dir_all(&staging).map_err(io_error_to_string)?;
    Ok(staging)
}

fn read_settings_xml(
    runtime: &dyn AppRuntime,
    settings_file_path: &Path,
) -> Result<String, String> {
    runtime.log(
        "DEBUG",
        &format!(
            "Attempting to read GCMODSETTINGS.MXML at: {}",
            settings_file_path.display()
        ),
    );

    match fs::read_to_string(settings_file_path) {
        Ok(content) => {
            runtime.log(
                "DEBUG",
                &format!(
                    "Read GCMODSETTINGS.MXML successfully. Content length: {} bytes",
                    content.len()
                ),
            );
            Ok(content)
        }
        Err(error) => {
            let err = format!("Failed to read GCMODSETTINGS.MXML: {}", error);
            runtime.log("ERROR", &err);
            Err(err)
        }
    }
}

fn parse_settings_xml(runtime: &dyn AppRuntime, xml_content: &str) -> Result<SettingsData, String> {
    runtime.log("DEBUG", "Parsing GCMODSETTINGS.MXML...");

    match mod_settings_store::parse_settings(xml_content) {
        Ok(parsed) => {
            runtime.log("DEBUG", "Parsed GCMODSETTINGS.MXML successfully.");
            Ok(parsed)
        }
        Err(error) => {
            let err = format!("Failed to parse GCMODSETTINGS.MXML: {}", error);
            runtime.log("ERROR", &err);
            Err(err)
        }
    }
}

fn ensure_existing_source_root(
    runtime: &dyn AppRuntime,
    library_dir: &Path,
    library_id: &str,
) -> Result<PathBuf, String> {
    let source_root = library_dir.join(library_id);
    if source_root.exists() {
        Ok(source_root)
    } else {
        let err = format!("Library folder missing: {:?}", source_root);
        runtime.log("ERROR", &err);
        Err(err)
    }
}

fn select_install_items(
    runtime: &dyn AppRuntime,
    source_root: &Path,
    selected_folders: &[String],
) -> Result<Vec<String>, String> {
    if is_scan_all_selection(selected_folders) {
        runtime.log(
            "INFO",
            "No specific folders selected. Scanning all top-level folders.",
        );
    } else {
        runtime.log(
            "INFO",
            &format!("Selected folders to install: {:?}", selected_folders),
        );
    }

    select_items_to_process(source_root, selected_folders)
}

fn read_mod_id_for_candidate(path: &Path) -> Option<String> {
    read_mod_info(path).and_then(|info| info.mod_id)
}

pub fn get_staging_dir_with(runtime: &dyn AppRuntime) -> Result<PathBuf, String> {
    let root = runtime.get_pulsar_root()?;
    create_staging_dir(&root)
}

pub fn get_all_mods_for_render_with(
    runtime: &dyn AppRuntime,
) -> Result<Vec<ModRenderData>, String> {
    let game_path = runtime
        .find_game_path()
        .ok_or_else(missing_game_installation_path_error)?;
    let mods_path = game_path.join("GAMEDATA").join("MODS");
    let settings_file_path = settings_paths::mod_settings_file(&game_path);

    if !settings_file_path.exists() {
        runtime.log(
            "DEBUG",
            &format!(
                "GCMODSETTINGS.MXML not found at: {}",
                settings_file_path.display()
            ),
        );
        return Ok(Vec::new());
    }

    let (real_folders_map, real_folders_set) = read_real_folders(&mods_path);
    let xml_content = read_settings_xml(runtime, &settings_file_path)?;
    let mut root = parse_settings_xml(runtime, &xml_content)?;

    let dirty = clean_orphaned_entries(&mut root, &real_folders_set);
    if dirty {
        let _ = mod_settings_store::save_settings_file(&settings_file_path, &root);
        runtime.log("INFO", "Cleaned orphaned mods from GCMODSETTINGS.MXML");
    }

    Ok(build_mods_to_render(&root, &real_folders_map, &mods_path))
}

pub fn finalize_installation_with(
    runtime: &dyn AppRuntime,
    library_id: String,
    selected_folders: Vec<String>,
    flatten_paths: bool,
) -> Result<InstallationAnalysis, String> {
    runtime.log(
        "INFO",
        &format!(
            "Finalizing installation. Source: {}, Flatten: {}",
            library_id, flatten_paths
        ),
    );

    let game_path = runtime
        .find_game_path()
        .ok_or_else(missing_game_path_error)?;
    let mods_path = game_path.join("GAMEDATA").join("MODS");
    fs::create_dir_all(&mods_path).map_err(io_error_to_string)?;

    let library_dir = runtime.get_library_dir()?;
    let source_root = ensure_existing_source_root(runtime, &library_dir, &library_id)?;

    let installed_mods_by_id = collect_installed_mods_by_id(&mods_path);
    let staging_dir = get_staging_dir_with(runtime)?;
    let conflict_staging_dir =
        conflict_staging_path(&staging_dir, chrono::Utc::now().timestamp_millis());

    let items_to_process = select_install_items(runtime, &source_root, &selected_folders)?;

    let ops = build_deploy_ops(&source_root, items_to_process, flatten_paths)?;
    let candidates: Vec<DeployCandidate> =
        build_deploy_candidates_with(ops, &mods_path, read_mod_id_for_candidate);

    let actions = plan_install_actions(
        candidates,
        &installed_mods_by_id,
        &mods_path,
        &conflict_staging_dir,
    );
    let (successes, conflicts) = apply_planned_install_actions(actions, &conflict_staging_dir)?;

    runtime.log("INFO", "Installation finalization complete.");
    Ok(InstallationAnalysis {
        successes,
        conflicts,
        messy_archive_path: None,
        active_archive_path: None,
        selection_needed: false,
        temp_id: None,
        available_folders: None,
    })
}

#[cfg(test)]
#[path = "install_service_tests.rs"]
mod tests;
