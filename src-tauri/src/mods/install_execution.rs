use crate::fs_ops::deploy_structure_recursive;
use crate::models::{ModConflictInfo, ModInstallInfo};
use crate::mods::install_planning::PlannedInstallAction;
use crate::read_mod_info;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn collect_installed_mods_by_id(mods_path: &Path) -> HashMap<String, String> {
    let mut installed_mods_by_id = HashMap::new();
    if let Ok(entries) = fs::read_dir(mods_path) {
        for entry in entries.filter_map(Result::ok) {
            if let Some(info) = read_mod_info(&entry.path()) {
                if let (Some(mod_id), Some(folder_name)) = (
                    info.mod_id,
                    entry.path().file_name().and_then(|n| n.to_str()),
                ) {
                    installed_mods_by_id.insert(mod_id, folder_name.to_string());
                }
            }
        }
    }
    installed_mods_by_id
}

pub fn apply_planned_install_actions(
    actions: Vec<PlannedInstallAction>,
    conflict_staging_path: &Path,
) -> Result<(Vec<ModInstallInfo>, Vec<ModConflictInfo>), String> {
    let mut successes = Vec::new();
    let mut conflicts = Vec::new();

    for action in actions {
        match action {
            PlannedInstallAction::StageConflict {
                source,
                staged_path,
                new_mod_name,
                old_mod_folder_name,
            } => {
                if !conflict_staging_path.exists() {
                    fs::create_dir_all(conflict_staging_path).map_err(|e| e.to_string())?;
                }

                let _ = deploy_structure_recursive(&source, &staged_path);
                conflicts.push(ModConflictInfo {
                    new_mod_name,
                    temp_path: staged_path.to_string_lossy().into_owned(),
                    old_mod_folder_name,
                });
            }
            PlannedInstallAction::DeployDirect {
                source,
                final_dest_path,
                dest_name,
            } => {
                if let Err(e) = deploy_structure_recursive(&source, &final_dest_path) {
                    return Err(format!("Failed to deploy {}: {}", dest_name, e));
                }

                successes.push(ModInstallInfo {
                    name: dest_name,
                    temp_path: final_dest_path.to_string_lossy().into_owned(),
                });
            }
        }
    }

    Ok((successes, conflicts))
}

#[cfg(test)]
#[path = "install_execution_tests.rs"]
mod tests;
