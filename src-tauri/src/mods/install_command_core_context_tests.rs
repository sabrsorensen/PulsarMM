use super::{archive_path_from_input, build_install_archive_context};
use std::path::PathBuf;

#[test]
fn archive_path_from_input_roundtrips() {
    assert_eq!(
        archive_path_from_input("/tmp/archive.zip"),
        PathBuf::from("/tmp/archive.zip")
    );
}

#[test]
fn archive_path_from_input_decodes_file_urls() {
    assert_eq!(
        archive_path_from_input("file:///tmp/Ship%20Pack.zip"),
        PathBuf::from("/tmp/Ship Pack.zip")
    );
}

#[test]
fn build_install_archive_context_derives_expected_paths() {
    let out = build_install_archive_context(
        PathBuf::from("/downloads/Example.zip").as_path(),
        PathBuf::from("/library").as_path(),
    )
    .expect("expected install archive context");
    assert_eq!(out.final_archive_path_str, "/downloads/Example.zip");
    assert_eq!(out.library_id, "Example.zip_unpacked");
    assert_eq!(
        out.library_mod_path,
        PathBuf::from("/library/Example.zip_unpacked")
    );
}

#[test]
fn build_install_archive_context_errors_when_archive_name_missing() {
    let out = build_install_archive_context(
        PathBuf::from("/").as_path(),
        PathBuf::from("/library").as_path(),
    );
    assert!(out.is_err());
}
