use super::{build_deploy_candidates_with, conflict_staging_path, is_scan_all_selection};
use crate::mods::install_scan::DeployOp;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_install_finalize_flow_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp test dir");
    dir
}

#[test]
fn scan_all_selection_detects_empty_and_dot() {
    assert!(is_scan_all_selection(&[]));
    assert!(is_scan_all_selection(&[".".to_string()]));
    assert!(!is_scan_all_selection(&["mod_a".to_string()]));
}

#[test]
fn conflict_staging_path_formats_with_timestamp() {
    let base = Path::new("/tmp/staging");
    let path = conflict_staging_path(base, 12345);
    assert_eq!(path, PathBuf::from("/tmp/staging/conflict_12345"));
}

#[test]
fn build_deploy_candidates_maps_fields_and_dest_existence() {
    let root = temp_test_dir("candidates");
    let mods_path = root.join("mods");
    fs::create_dir_all(&mods_path).expect("create mods dir should succeed");
    fs::create_dir_all(mods_path.join("mod_a")).expect("create mod dir should succeed");

    let ops = vec![
        DeployOp {
            source: root.join("src/mod_a"),
            dest_name: "mod_a".to_string(),
        },
        DeployOp {
            source: root.join("src/mod_b"),
            dest_name: "mod_b".to_string(),
        },
    ];

    let candidates = build_deploy_candidates_with(ops, &mods_path, |path| {
        path.file_name()
            .map(|n| format!("id_{}", n.to_string_lossy()))
    });

    assert_eq!(candidates.len(), 2);
    assert!(candidates[0].dest_exists);
    assert!(!candidates[1].dest_exists);
    assert_eq!(candidates[0].mod_id.as_deref(), Some("id_mod_a"));
    assert_eq!(candidates[1].mod_id.as_deref(), Some("id_mod_b"));

    fs::remove_dir_all(root).expect("cleanup should succeed");
}
