use super::{collect_nodes, target_path_from_relative};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_stage_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn target_path_rejects_parent_escape() {
    let root = PathBuf::from("/tmp/pulsar");
    assert!(target_path_from_relative(&root, "../outside").is_err());
    assert!(target_path_from_relative(&root, "/tmp/outside").is_err());
}

#[test]
fn target_path_uses_root_for_empty_relative() {
    let root = PathBuf::from("/tmp/pulsar");
    assert_eq!(
        target_path_from_relative(&root, "").expect("empty relative path should resolve"),
        root
    );
}

#[test]
fn collect_nodes_sorts_dirs_first_then_names() {
    let root = temp_test_dir("nodes");
    fs::write(root.join("z.txt"), "z").expect("write file should succeed");
    fs::write(root.join("a.txt"), "a").expect("write file should succeed");
    fs::create_dir_all(root.join("DirB")).expect("create dir should succeed");
    fs::create_dir_all(root.join("dira")).expect("create dir should succeed");

    let names: Vec<(String, bool)> = collect_nodes(&root, "")
        .expect("collect should succeed")
        .into_iter()
        .map(|n| (n.name, n.is_dir))
        .collect();
    assert_eq!(
        names,
        vec![
            ("dira".to_string(), true),
            ("DirB".to_string(), true),
            ("a.txt".to_string(), false),
            ("z.txt".to_string(), false),
        ]
    );

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn collect_nodes_returns_empty_for_non_directory_target() {
    let root = temp_test_dir("file_target");
    fs::write(root.join("entry.txt"), "x").expect("write file should succeed");

    let nodes = collect_nodes(&root, "entry.txt").expect("collect should succeed");
    assert!(nodes.is_empty());

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn collect_nodes_returns_empty_for_missing_directory() {
    let root = temp_test_dir("missing");
    let nodes = collect_nodes(&root, "does/not/exist").expect("collect should succeed");
    assert!(nodes.is_empty());
    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn collect_nodes_works_for_nested_relative_path() {
    let root = temp_test_dir("nested");
    fs::create_dir_all(root.join("inner")).expect("create dir should succeed");
    fs::write(root.join("inner/item.txt"), "x").expect("write file should succeed");

    let nodes = collect_nodes(&root, "inner").expect("collect should succeed");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].name, "item.txt");
    assert!(!nodes[0].is_dir);

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[cfg(unix)]
#[test]
fn collect_nodes_errors_when_directory_cannot_be_read() {
    let root = temp_test_dir("unreadable");
    let blocked = root.join("blocked");
    fs::create_dir_all(&blocked).expect("create dir should succeed");
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000))
        .expect("set permissions should succeed");

    let err = match collect_nodes(&root, "blocked") {
        Ok(_) => panic!("unreadable dir should fail"),
        Err(err) => err,
    };
    assert!(!err.is_empty(), "expected non-empty read_dir error");

    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o755))
        .expect("restore permissions should succeed");
    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[cfg(unix)]
#[test]
fn collect_nodes_rejects_invalid_relative_path() {
    let root = temp_test_dir("invalid_relative");

    let err = match collect_nodes(&root, "../outside") {
        Ok(_) => panic!("invalid relative path should fail"),
        Err(err) => err,
    };
    assert_eq!(err, "Invalid path access");

    fs::remove_dir_all(root).expect("cleanup should succeed");
}
