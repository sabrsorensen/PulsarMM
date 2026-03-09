use super::*;
use std::collections::HashMap;

#[test]
fn decide_archive_flow_prefers_installable_selection_when_multiple() {
    let installable = vec!["A".to_string(), "B".to_string()];
    let folders = vec!["X".to_string()];
    let decision = decide_archive_flow(&installable, &folders, "lib-id", "/tmp/a.zip");

    match decision {
        ArchiveDecision::WaitForSelection(analysis) => {
            assert!(analysis.selection_needed);
            assert_eq!(
                analysis.available_folders,
                Some(vec!["A".to_string(), "B".to_string()])
            );
        }
        _ => panic!("expected selection"),
    }
}

#[test]
fn decide_archive_flow_finalizes_single_installable_as_flattened() {
    let installable = vec!["OnlyOne".to_string()];
    let decision = decide_archive_flow(&installable, &[], "lib-id", "/tmp/a.zip");
    match decision {
        ArchiveDecision::Finalize(req) => {
            assert_eq!(req.selected_folders, vec!["OnlyOne".to_string()]);
            assert!(req.flatten_paths);
        }
        _ => panic!("expected finalize"),
    }
}

#[test]
fn decide_archive_flow_uses_folder_selection_when_no_installables() {
    let folders = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let decision = decide_archive_flow(&[], &folders, "lib-id", "/tmp/a.zip");
    match decision {
        ArchiveDecision::WaitForSelection(analysis) => {
            assert_eq!(analysis.available_folders, Some(folders));
            assert_eq!(analysis.temp_id.as_deref(), Some("lib-id"));
        }
        _ => panic!("expected selection"),
    }
}

#[test]
fn decide_archive_flow_defaults_to_non_flattened_finalize() {
    let decision = decide_archive_flow(&[], &[], "lib-id", "/tmp/a.zip");
    match decision {
        ArchiveDecision::Finalize(req) => {
            assert!(req.selected_folders.is_empty());
            assert!(!req.flatten_paths);
        }
        _ => panic!("expected finalize"),
    }
}

#[test]
fn plan_install_actions_prioritizes_mod_id_conflicts_then_overwrite_then_direct() {
    let candidates = vec![
        DeployCandidate {
            source: PathBuf::from("/src/one"),
            dest_name: "one".to_string(),
            mod_id: Some("42".to_string()),
            dest_exists: false,
        },
        DeployCandidate {
            source: PathBuf::from("/src/two"),
            dest_name: "two".to_string(),
            mod_id: None,
            dest_exists: true,
        },
        DeployCandidate {
            source: PathBuf::from("/src/three"),
            dest_name: "three".to_string(),
            mod_id: None,
            dest_exists: false,
        },
    ];
    let installed = HashMap::from([("42".to_string(), "existing-folder".to_string())]);
    let actions = plan_install_actions(
        candidates,
        &installed,
        Path::new("/mods"),
        Path::new("/staging/conflict_1"),
    );

    assert!(matches!(
        &actions[0],
        PlannedInstallAction::StageConflict {
            old_mod_folder_name,
            new_mod_name,
            ..
        } if old_mod_folder_name == "existing-folder" && new_mod_name == "one"
    ));
    assert!(matches!(
        &actions[1],
        PlannedInstallAction::StageConflict {
            old_mod_folder_name,
            new_mod_name,
            ..
        } if old_mod_folder_name == "two" && new_mod_name == "two"
    ));
    assert!(matches!(
        &actions[2],
        PlannedInstallAction::DeployDirect {
            dest_name,
            final_dest_path,
            ..
        } if dest_name == "three" && final_dest_path == &PathBuf::from("/mods/three")
    ));
}
