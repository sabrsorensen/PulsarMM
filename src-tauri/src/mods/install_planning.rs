use crate::models::InstallationAnalysis;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizeRequest {
    pub selected_folders: Vec<String>,
    pub flatten_paths: bool,
}

pub enum ArchiveDecision {
    WaitForSelection(InstallationAnalysis),
    Finalize(FinalizeRequest),
}

pub fn decide_archive_flow(
    installable_paths: &[String],
    folder_names: &[String],
    library_id: &str,
    active_archive_path: &str,
) -> ArchiveDecision {
    if installable_paths.len() > 1 {
        return ArchiveDecision::WaitForSelection(InstallationAnalysis {
            successes: vec![],
            conflicts: vec![],
            messy_archive_path: None,
            active_archive_path: Some(active_archive_path.to_string()),
            selection_needed: true,
            temp_id: Some(library_id.to_string()),
            available_folders: Some(installable_paths.to_vec()),
        });
    }

    if installable_paths.len() == 1 {
        return ArchiveDecision::Finalize(FinalizeRequest {
            selected_folders: vec![installable_paths[0].clone()],
            flatten_paths: true,
        });
    }

    if folder_names.len() > 1 {
        return ArchiveDecision::WaitForSelection(InstallationAnalysis {
            successes: vec![],
            conflicts: vec![],
            messy_archive_path: None,
            active_archive_path: Some(active_archive_path.to_string()),
            selection_needed: true,
            temp_id: Some(library_id.to_string()),
            available_folders: Some(folder_names.to_vec()),
        });
    }

    ArchiveDecision::Finalize(FinalizeRequest {
        selected_folders: vec![],
        flatten_paths: false,
    })
}

#[derive(Debug, Clone)]
pub struct DeployCandidate {
    pub source: PathBuf,
    pub dest_name: String,
    pub mod_id: Option<String>,
    pub dest_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedInstallAction {
    StageConflict {
        source: PathBuf,
        staged_path: PathBuf,
        new_mod_name: String,
        old_mod_folder_name: String,
    },
    DeployDirect {
        source: PathBuf,
        final_dest_path: PathBuf,
        dest_name: String,
    },
}

pub fn plan_install_actions(
    candidates: Vec<DeployCandidate>,
    installed_mods_by_id: &HashMap<String, String>,
    mods_path: &Path,
    conflict_staging_path: &Path,
) -> Vec<PlannedInstallAction> {
    let mut actions = Vec::new();

    for candidate in candidates {
        if let Some(mod_id) = &candidate.mod_id {
            if let Some(old_folder_name) = installed_mods_by_id.get(mod_id) {
                actions.push(PlannedInstallAction::StageConflict {
                    staged_path: conflict_staging_path.join(&candidate.dest_name),
                    source: candidate.source,
                    new_mod_name: candidate.dest_name,
                    old_mod_folder_name: old_folder_name.clone(),
                });
                continue;
            }
        }

        if candidate.dest_exists {
            actions.push(PlannedInstallAction::StageConflict {
                staged_path: conflict_staging_path.join(&candidate.dest_name),
                source: candidate.source,
                new_mod_name: candidate.dest_name.clone(),
                old_mod_folder_name: candidate.dest_name,
            });
        } else {
            actions.push(PlannedInstallAction::DeployDirect {
                final_dest_path: mods_path.join(&candidate.dest_name),
                source: candidate.source,
                dest_name: candidate.dest_name,
            });
        }
    }

    actions
}

#[cfg(test)]
#[path = "install_planning_tests.rs"]
mod tests;
