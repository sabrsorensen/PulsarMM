use super::{
    copy_dir_recursive, deploy_structure_recursive, ensure_parent_dir_exists, find_folder_in_tree,
    smart_deploy_file,
};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_fsops_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[cfg(unix)]
fn shm_test_dir(prefix: &str) -> PathBuf {
    let dir =
        PathBuf::from("/dev/shm").join(format!("pulsarmm_fsops_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create shm temp dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent");
    }
    fs::write(path, content).expect("failed to write file");
}

#[test]
fn copy_dir_recursive_copies_nested_tree() {
    let root = temp_test_dir("copy");
    let src = root.join("src");
    let dest = root.join("dest");

    write_file(&src.join("a.txt"), "a");
    write_file(&src.join("nested/b.txt"), "b");
    fs::create_dir_all(&dest).unwrap();

    copy_dir_recursive(&src, &dest).unwrap();
    assert_eq!(fs::read_to_string(dest.join("a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(dest.join("nested/b.txt")).unwrap(), "b");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deploy_structure_recursive_overwrites_existing_file() {
    let root = temp_test_dir("deploy");
    let src = root.join("src");
    let dest = root.join("dest");

    write_file(&src.join("same.txt"), "new");
    write_file(&dest.join("same.txt"), "old");

    deploy_structure_recursive(&src, &dest).unwrap();
    assert_eq!(fs::read_to_string(dest.join("same.txt")).unwrap(), "new");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deploy_structure_recursive_creates_missing_directories() {
    let root = temp_test_dir("deploy_mkdir");
    let src = root.join("src");
    let dest = root.join("dest");

    write_file(&src.join("nested/deeper/file.txt"), "x");
    deploy_structure_recursive(&src, &dest).unwrap();
    assert_eq!(
        fs::read_to_string(dest.join("nested/deeper/file.txt")).unwrap(),
        "x"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn find_folder_in_tree_matches_case_insensitive_nested() {
    let root = temp_test_dir("find");
    fs::create_dir_all(root.join("A/B/TargetFolder")).unwrap();

    let found = find_folder_in_tree(&root, "targetfolder").unwrap();
    assert_eq!(
        found.file_name().and_then(|n| n.to_str()),
        Some("TargetFolder")
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn find_folder_in_tree_returns_none_when_not_found() {
    let root = temp_test_dir("find_none");
    fs::create_dir_all(root.join("A/B/C")).unwrap();
    assert!(find_folder_in_tree(&root, "missing").is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn copy_dir_recursive_errors_when_source_missing() {
    let root = temp_test_dir("copy_missing");
    let missing_src = root.join("nope");
    let dest = root.join("dest");
    fs::create_dir_all(&dest).unwrap();

    assert!(copy_dir_recursive(&missing_src, &dest).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn copy_dir_recursive_errors_when_dest_root_missing_for_files() {
    let root = temp_test_dir("copy_dest_missing");
    let src = root.join("src");
    let dest = root.join("dest");
    write_file(&src.join("a.txt"), "a");

    assert!(copy_dir_recursive(&src, &dest).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deploy_structure_recursive_errors_when_source_missing() {
    let root = temp_test_dir("deploy_missing");
    let missing_src = root.join("no_src");
    let dest = root.join("dest");
    assert!(deploy_structure_recursive(&missing_src, &dest).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deploy_structure_recursive_errors_on_file_dir_conflict() {
    let root = temp_test_dir("deploy_conflict");
    let src = root.join("src");
    let dest = root.join("dest");
    write_file(&src.join("same"), "new-content");
    fs::create_dir_all(dest.join("same")).unwrap();

    assert!(deploy_structure_recursive(&src, &dest).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn find_folder_in_tree_returns_none_for_non_directory_root() {
    let root = temp_test_dir("find_file_root");
    let file_root = root.join("root.txt");
    write_file(&file_root, "not a directory");

    assert!(find_folder_in_tree(&file_root, "anything").is_none());

    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn find_folder_in_tree_recurses_through_non_utf8_directory_names() {
    let root = temp_test_dir("find_non_utf8");
    let invalid = OsString::from_vec(vec![0x66, 0x6f, 0x80]);
    let nested = root.join(PathBuf::from(invalid)).join("TargetFolder");
    fs::create_dir_all(&nested).unwrap();

    let found = find_folder_in_tree(&root, "targetfolder").expect("target should be found");
    assert_eq!(
        found.file_name().and_then(|name| name.to_str()),
        Some("TargetFolder")
    );

    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn smart_deploy_file_falls_back_to_copy_across_filesystems_and_creates_parent() {
    let source_root = shm_test_dir("smart_deploy_src");
    let dest_root = temp_test_dir("smart_deploy_dest");
    let source = source_root.join("source.txt");
    let dest = dest_root.join("nested").join("copied.txt");
    write_file(&source, "cross-device");

    smart_deploy_file(&source, &dest).expect("smart deploy should fall back to copy");

    assert_eq!(fs::read_to_string(&dest).unwrap(), "cross-device");

    fs::remove_dir_all(source_root).unwrap();
    fs::remove_dir_all(dest_root).unwrap();
}

#[test]
fn ensure_parent_dir_exists_creates_missing_parent() {
    let root = temp_test_dir("ensure_parent");
    let dest = root.join("nested").join("file.txt");

    ensure_parent_dir_exists(&dest).unwrap();

    assert!(root.join("nested").is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ensure_parent_dir_exists_allows_paths_without_parent() {
    assert!(ensure_parent_dir_exists(Path::new("")).is_ok());
    assert!(ensure_parent_dir_exists(Path::new("/")).is_ok());
}

#[test]
fn ensure_parent_dir_exists_errors_when_ancestor_is_a_file() {
    let root = temp_test_dir("ensure_parent_error");
    let file_parent = root.join("file-parent");
    fs::write(&file_parent, "not a directory").unwrap();

    let err = ensure_parent_dir_exists(&file_parent.join("nested").join("file.txt"))
        .expect_err("file ancestor should make create_dir_all fail");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn smart_deploy_file_errors_when_existing_destination_is_a_directory() {
    let root = temp_test_dir("smart_deploy_dir_dest");
    let source = root.join("source.txt");
    let dest = root.join("dest");
    write_file(&source, "payload");
    fs::create_dir_all(&dest).unwrap();

    let err = smart_deploy_file(&source, &dest)
        .expect_err("directory destination should fail remove_file");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).unwrap();
}
