use super::{
    check_library_existence_map, downloads_target_from_root, library_target_from_root,
    sort_file_nodes, validate_move_target,
};
use crate::models::FileNode;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_pathops_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn target_path_builders_append_expected_folder_names() {
    let root = PathBuf::from("/tmp/pulsar");
    assert_eq!(
        downloads_target_from_root(&root),
        PathBuf::from("/tmp/pulsar/downloads")
    );
    assert_eq!(
        library_target_from_root(&root),
        PathBuf::from("/tmp/pulsar/Library")
    );
}

#[test]
fn validate_move_target_allows_same_or_different_and_rejects_nested() {
    let old = PathBuf::from("/tmp/a");
    let same = PathBuf::from("/tmp/a");
    let different = PathBuf::from("/tmp/b");
    let nested = PathBuf::from("/tmp/a/sub");

    assert!(validate_move_target(&old, &same).is_ok());
    assert!(validate_move_target(&old, &different).is_ok());
    assert!(validate_move_target(&old, &nested).is_err());
}

#[test]
fn sort_file_nodes_puts_dirs_first_then_name_case_insensitive() {
    let mut nodes = vec![
        FileNode {
            name: "z-file.txt".to_string(),
            is_dir: false,
        },
        FileNode {
            name: "Beta".to_string(),
            is_dir: true,
        },
        FileNode {
            name: "alpha".to_string(),
            is_dir: true,
        },
        FileNode {
            name: "A-file.txt".to_string(),
            is_dir: false,
        },
    ];

    sort_file_nodes(&mut nodes);
    let ordered: Vec<(String, bool)> = nodes.into_iter().map(|n| (n.name, n.is_dir)).collect();
    assert_eq!(
        ordered,
        vec![
            ("alpha".to_string(), true),
            ("Beta".to_string(), true),
            ("A-file.txt".to_string(), false),
            ("z-file.txt".to_string(), false)
        ]
    );
}

#[test]
fn check_library_existence_map_marks_existing_unpacked_dirs() {
    let dir = temp_test_dir("existence");
    fs::create_dir_all(dir.join("A.zip_unpacked")).expect("create unpacked dir should succeed");

    let map = check_library_existence_map(&dir, vec!["A.zip".to_string(), "B.zip".to_string()]);
    assert_eq!(map.get("A.zip"), Some(&true));
    assert_eq!(map.get("B.zip"), Some(&false));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
