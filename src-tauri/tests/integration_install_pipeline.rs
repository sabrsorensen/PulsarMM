use pulsar::mods::install_archive_flow::{library_folder_name_for_archive, scan_library_mod_path};
use pulsar::mods::install_finalize_flow::{
    build_deploy_candidates_with, conflict_staging_path, is_scan_all_selection,
};
use pulsar::mods::install_planning::{
    decide_archive_flow, plan_install_actions, ArchiveDecision, PlannedInstallAction,
};
use pulsar::mods::install_scan::build_deploy_ops;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_it_install_pipeline_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn install_pipeline_detects_selection_then_plans_conflicts_and_deploys() {
    let root = temp_test_dir("pipeline");
    let unpacked = root.join("archive_unpacked");
    let mods_path = root.join("mods");
    fs::create_dir_all(&mods_path).unwrap();

    fs::create_dir_all(unpacked.join("mod_a/UI")).unwrap();
    fs::create_dir_all(unpacked.join("mod_b/METADATA")).unwrap();
    write_file(&unpacked.join("mod_a/mod_info.json"), r#"{"modId":"id-a"}"#);
    write_file(&unpacked.join("mod_b/mod_info.json"), r#"{"modId":"id-b"}"#);

    let (mut folder_names, mut installables) = scan_library_mod_path(&unpacked).unwrap();
    folder_names.sort();
    installables.sort();
    assert_eq!(folder_names, vec!["mod_a".to_string(), "mod_b".to_string()]);
    assert_eq!(installables, vec!["mod_a".to_string(), "mod_b".to_string()]);

    let decision = decide_archive_flow(&installables, &folder_names, "tmp-id", "/tmp/mod.zip");
    match decision {
        ArchiveDecision::WaitForSelection(analysis) => {
            assert!(analysis.selection_needed);
            assert_eq!(analysis.available_folders, Some(installables.clone()));
        }
        _ => panic!("expected selection-needed decision"),
    }

    fs::create_dir_all(mods_path.join("mod_a")).unwrap();
    let ops = build_deploy_ops(&unpacked, installables.clone(), true).unwrap();
    let candidates = build_deploy_candidates_with(ops, &mods_path, |source| {
        fs::read_to_string(source.join("mod_info.json"))
            .ok()
            .and_then(|json| {
                serde_json::from_str::<serde_json::Value>(&json)
                    .ok()
                    .and_then(|v| {
                        v.get("modId")
                            .and_then(|m| m.as_str())
                            .map(ToString::to_string)
                    })
            })
    });

    let installed_by_id = HashMap::from([("id-a".to_string(), "existing_mod_a".to_string())]);
    let staging = conflict_staging_path(&root.join("staging"), 42);
    let actions = plan_install_actions(candidates, &installed_by_id, &mods_path, &staging);

    assert_eq!(actions.len(), 2);
    assert!(
        actions.iter().any(|a| matches!(
            a,
            PlannedInstallAction::StageConflict {
                new_mod_name, old_mod_folder_name, ..
            } if new_mod_name == "mod_a" && old_mod_folder_name == "existing_mod_a"
        )),
        "expected mod_id conflict to be staged"
    );
    assert!(
        actions.iter().any(|a| matches!(
            a,
            PlannedInstallAction::DeployDirect {
                dest_name, ..
            } if dest_name == "mod_b"
        )),
        "expected non-conflicting mod to deploy directly"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn helpers_resolve_expected_folder_names_and_selection_rules() {
    let folder = library_folder_name_for_archive(Path::new("/tmp/ShipPack.7z"))
        .expect("expected folder name");
    assert_eq!(folder, "ShipPack.7z_unpacked");
    assert!(is_scan_all_selection(&[]));
    assert!(is_scan_all_selection(&[".".to_string()]));
    assert!(!is_scan_all_selection(&["mod_x".to_string()]));
}
