use super::*;
use crate::mods::install_planning::PlannedInstallAction;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_install_execution_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn collect_installed_mods_reads_mod_id_mappings() {
    let dir = temp_test_dir("collect_ids");
    fs::create_dir_all(dir.join("ModA")).unwrap();
    fs::create_dir_all(dir.join("ModB")).unwrap();
    fs::write(
        dir.join("ModA/mod_info.json"),
        r#"{"modId":"100","fileId":"1","installSource":"a.zip"}"#,
    )
    .unwrap();
    fs::write(dir.join("ModB/mod_info.json"), r#"{"fileId":"2"}"#).unwrap();

    let map = collect_installed_mods_by_id(&dir);
    assert_eq!(map.get("100").map(String::as_str), Some("ModA"));
    assert_eq!(map.len(), 1);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn collect_installed_mods_skips_entries_without_readable_mod_info() {
    let dir = temp_test_dir("collect_ids_unreadable");
    fs::create_dir_all(dir.join("ValidMod")).unwrap();
    fs::create_dir_all(dir.join("InvalidMod")).unwrap();
    fs::write(
        dir.join("ValidMod/mod_info.json"),
        r#"{"modId":"200","fileId":"1","installSource":"a.zip"}"#,
    )
    .unwrap();
    fs::write(dir.join("InvalidMod/mod_info.json"), "{not-json").unwrap();

    let map = collect_installed_mods_by_id(&dir);
    assert_eq!(map.get("200").map(String::as_str), Some("ValidMod"));
    assert_eq!(map.len(), 1);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn collect_installed_mods_returns_empty_for_missing_directory() {
    let dir = temp_test_dir("collect_ids_missing");
    let missing = dir.join("missing");

    let map = collect_installed_mods_by_id(&missing);
    assert!(map.is_empty());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn apply_actions_stages_conflicts_and_deploys_direct() {
    let dir = temp_test_dir("apply_actions");
    let src_conflict = dir.join("src_conflict");
    let src_direct = dir.join("src_direct");
    let staging = dir.join("staging");
    let dest = dir.join("mods/DirectMod");

    fs::create_dir_all(&src_conflict).unwrap();
    fs::create_dir_all(&src_direct).unwrap();
    fs::write(src_conflict.join("a.pak"), "a").unwrap();
    fs::write(src_direct.join("b.pak"), "b").unwrap();

    let actions = vec![
        PlannedInstallAction::StageConflict {
            source: src_conflict.clone(),
            staged_path: staging.join("ConflictMod"),
            new_mod_name: "ConflictMod".to_string(),
            old_mod_folder_name: "OldMod".to_string(),
        },
        PlannedInstallAction::DeployDirect {
            source: src_direct.clone(),
            final_dest_path: dest.clone(),
            dest_name: "DirectMod".to_string(),
        },
    ];

    let (successes, conflicts) = apply_planned_install_actions(actions, &staging).unwrap();
    assert_eq!(successes.len(), 1);
    assert_eq!(conflicts.len(), 1);
    assert!(dest.join("b.pak").exists());
    assert!(staging.join("ConflictMod/a.pak").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn apply_actions_stages_conflict_when_staging_root_already_exists() {
    let dir = temp_test_dir("apply_actions_existing_staging");
    let src_conflict = dir.join("src_conflict");
    let staging = dir.join("staging");

    fs::create_dir_all(&src_conflict).unwrap();
    fs::create_dir_all(&staging).unwrap();
    fs::write(src_conflict.join("a.pak"), "a").unwrap();

    let actions = vec![PlannedInstallAction::StageConflict {
        source: src_conflict,
        staged_path: staging.join("ConflictMod"),
        new_mod_name: "ConflictMod".to_string(),
        old_mod_folder_name: "OldMod".to_string(),
    }];

    let (successes, conflicts) = apply_planned_install_actions(actions, &staging).unwrap();
    assert!(successes.is_empty());
    assert_eq!(conflicts.len(), 1);
    assert!(staging.join("ConflictMod/a.pak").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn apply_actions_returns_deploy_error_for_direct_failure() {
    let dir = temp_test_dir("apply_actions_direct_error");
    let src_direct = dir.join("src_direct");
    let staging = dir.join("staging");
    let dest = dir.join("mods").join("DirectMod");

    fs::create_dir_all(&src_direct).unwrap();
    fs::write(src_direct.join("b.pak"), "b").unwrap();
    fs::create_dir_all(dir.join("mods")).unwrap();
    fs::write(&dest, "blocking-file").unwrap();

    let actions = vec![PlannedInstallAction::DeployDirect {
        source: src_direct,
        final_dest_path: dest,
        dest_name: "DirectMod".to_string(),
    }];

    let err = match apply_planned_install_actions(actions, &staging) {
        Ok(_) => panic!("direct deploy should fail"),
        Err(err) => err,
    };
    assert!(err.contains("Failed to deploy DirectMod"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn apply_actions_reports_staging_root_creation_error() {
    let dir = temp_test_dir("apply_actions_staging_error");
    let src_conflict = dir.join("src_conflict");
    let blocked_parent = dir.join("blocked-parent");
    let staging = blocked_parent.join("staging");

    fs::create_dir_all(&src_conflict).unwrap();
    fs::write(src_conflict.join("a.pak"), "a").unwrap();
    fs::write(&blocked_parent, "not-a-directory").unwrap();

    let actions = vec![PlannedInstallAction::StageConflict {
        source: src_conflict,
        staged_path: staging.join("ConflictMod"),
        new_mod_name: "ConflictMod".to_string(),
        old_mod_folder_name: "OldMod".to_string(),
    }];

    let err = match apply_planned_install_actions(actions, &staging) {
        Ok(_) => panic!("staging root creation should fail when parent is a file"),
        Err(err) => err,
    };
    assert!(!err.is_empty());

    fs::remove_dir_all(dir).unwrap();
}
