use super::{copy_archive_to_downloads, library_folder_name_for_archive, scan_library_mod_path};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_install_archive_flow_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp test dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn library_folder_name_uses_archive_filename() {
    let path = Path::new("/tmp/mods/cool-mod.zip");
    let folder = library_folder_name_for_archive(path).expect("expected name");
    assert_eq!(folder, "cool-mod.zip_unpacked");
}

#[test]
fn copy_archive_to_downloads_copies_when_outside_downloads() {
    let root = temp_test_dir("copy_outside");
    let downloads = root.join("downloads");
    let archive = root.join("incoming/mod.zip");
    write_file(&archive, "zip-data");

    let (final_path, already_in_downloads) =
        copy_archive_to_downloads(&archive, &downloads).expect("copy should succeed");

    assert!(!already_in_downloads);
    assert_eq!(final_path, downloads.join("mod.zip"));
    assert!(final_path.exists());

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn copy_archive_to_downloads_returns_same_when_already_under_downloads() {
    let root = temp_test_dir("copy_inside");
    let downloads = root.join("downloads");
    let archive = downloads.join("mod.zip");
    write_file(&archive, "zip-data");

    let (final_path, already_in_downloads) =
        copy_archive_to_downloads(&archive, &downloads).expect("copy should succeed");

    assert!(already_in_downloads);
    assert_eq!(final_path, archive);

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn scan_library_mod_path_collects_folders_and_installables() {
    let root = temp_test_dir("scan");
    fs::create_dir_all(root.join("mod_a/UI")).expect("create mod_a should succeed");
    fs::create_dir_all(root.join("mod_b/docs")).expect("create mod_b should succeed");
    write_file(&root.join("mod_b/README.txt"), "hello");

    let (mut folders, mut installables) =
        scan_library_mod_path(&root).expect("scan should succeed");
    folders.sort();
    installables.sort();

    assert_eq!(folders, vec!["mod_a".to_string(), "mod_b".to_string()]);
    assert_eq!(installables, vec!["mod_a".to_string()]);

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn library_folder_name_for_archive_errors_without_filename() {
    let err = library_folder_name_for_archive(Path::new("/"))
        .expect_err("root path should not have an archive filename");
    assert_eq!(err, "Invalid archive filename");
}

#[test]
fn copy_archive_to_downloads_reports_missing_source_and_invalid_filename() {
    let root = temp_test_dir("copy_errors");
    let downloads = root.join("downloads");

    let err = copy_archive_to_downloads(&root.join("missing.zip"), &downloads)
        .expect_err("missing archive should fail copy");
    assert!(!err.is_empty());

    let err = copy_archive_to_downloads(Path::new("/"), &downloads)
        .expect_err("root path should not be treated as a valid archive file");
    assert_eq!(err, "Invalid filename");

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn copy_archive_to_downloads_reports_downloads_dir_creation_error() {
    let root = temp_test_dir("copy_downloads_dir_error");
    let blocked_parent = root.join("blocked-parent");
    let downloads = blocked_parent.join("downloads");
    let archive = root.join("incoming/mod.zip");

    fs::write(&blocked_parent, "not-a-directory").expect("write blocking file");
    write_file(&archive, "zip-data");

    let err = copy_archive_to_downloads(&archive, &downloads)
        .expect_err("downloads dir creation should fail when parent is a file");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn scan_library_mod_path_errors_when_root_is_missing() {
    let root = temp_test_dir("scan_missing");
    let missing = root.join("does_not_exist");

    let err = scan_library_mod_path(&missing).expect_err("missing scan root should error");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("cleanup should succeed");
}
