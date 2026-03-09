use super::{
    detect_archive_kind, ensure_destination_exists, ensure_parent_dir_exists, ArchiveKind,
};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_archive_unit_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn detect_archive_kind_maps_supported_and_unsupported_extensions() {
    assert_eq!(
        detect_archive_kind(Path::new("mod.zip")).expect("zip should map"),
        ArchiveKind::Zip
    );
    assert_eq!(
        detect_archive_kind(Path::new("mod.rar")).expect("rar should map"),
        ArchiveKind::Rar
    );
    assert_eq!(
        detect_archive_kind(Path::new("mod.7z")).expect("7z should map"),
        ArchiveKind::SevenZ
    );

    let err = detect_archive_kind(Path::new("mod.txt")).expect_err("txt should fail");
    assert!(err.contains("Unsupported file type"));
}

#[test]
fn ensure_destination_exists_is_noop_or_creates_directory() {
    let root = temp_test_dir("destination_exists");
    let dest = root.join("out");

    ensure_destination_exists(&dest).expect("missing destination should be created");
    assert!(dest.is_dir());

    ensure_destination_exists(&dest).expect("existing destination should be allowed");

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn ensure_parent_dir_exists_handles_missing_and_parentless_paths() {
    let root = temp_test_dir("parent_dir");
    let nested = root.join("nested").join("file.txt");

    ensure_parent_dir_exists(&nested).expect("missing parent should be created");
    assert!(root.join("nested").is_dir());

    ensure_parent_dir_exists(Path::new("/")).expect("root path should be allowed");

    fs::remove_dir_all(root).expect("cleanup should succeed");
}
