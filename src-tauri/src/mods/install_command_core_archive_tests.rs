use super::{
    copy_archive_to_downloads_blocking, extract_archive_if_needed, scan_library_mod_path_blocking,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_install_command_runtime_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn create_zip_with_file(zip_path: &Path, internal_path: &str, content: &[u8]) {
    let file = fs::File::create(zip_path).expect("failed to create zip");
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file(internal_path, SimpleFileOptions::default())
        .expect("failed to start zip entry");
    writer
        .write_all(content)
        .expect("failed to write zip contents");
    writer.finish().expect("failed to finalize zip");
}

#[test]
fn copy_archive_to_downloads_blocking_copies_and_errors_as_expected() {
    let root = temp_test_dir("copy_archive");
    let downloads = root.join("downloads");
    let outside = root.join("outside");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let archive = outside.join("mod.zip");
    fs::write(&archive, b"zip-bytes").unwrap();

    let copied = copy_archive_to_downloads_blocking(archive.clone(), downloads.clone())
        .expect("archive should be copied into downloads");
    assert!(copied.starts_with(&downloads));
    assert_eq!(fs::read(&copied).unwrap(), b"zip-bytes");

    let missing = outside.join("missing.zip");
    let err = copy_archive_to_downloads_blocking(missing, downloads)
        .expect_err("missing source archive should fail");
    assert!(!err.is_empty(), "expected non-empty copy error");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn extract_archive_if_needed_skips_existing_and_extracts_missing() {
    let root = temp_test_dir("extract_archive");
    let archive = root.join("mod.zip");
    let existing_library = root.join("existing");
    let extracted_library = root.join("extracted");
    fs::create_dir_all(&existing_library).unwrap();
    create_zip_with_file(&archive, "Sample/payload.pak", b"pak");

    let existing_progress = Arc::new(Mutex::new(Vec::<u64>::new()));
    let existing_progress_out = existing_progress.clone();
    let mut existing_cb = move |pct| {
        existing_progress_out
            .lock()
            .expect("progress lock")
            .push(pct);
    };
    extract_archive_if_needed(archive.clone(), existing_library, &mut existing_cb)
        .expect("existing library path should skip extraction");
    assert!(existing_progress.lock().expect("progress lock").is_empty());

    let extracted_progress = Arc::new(Mutex::new(Vec::<u64>::new()));
    let extracted_progress_out = extracted_progress.clone();
    let mut extract_cb = move |pct| {
        extracted_progress_out
            .lock()
            .expect("progress lock")
            .push(pct);
    };
    extract_archive_if_needed(archive.clone(), extracted_library.clone(), &mut extract_cb)
        .expect("missing library path should extract archive");
    assert!(extracted_library
        .join("Sample")
        .join("payload.pak")
        .exists());
    assert!(
        extracted_progress
            .lock()
            .expect("progress lock")
            .iter()
            .any(|pct| *pct == 100),
        "expected a 100% extraction progress event"
    );

    let invalid_archive = root.join("invalid.zip");
    fs::write(&invalid_archive, b"not a zip").unwrap();
    let err = extract_archive_if_needed(invalid_archive, root.join("invalid-out"), &mut |_pct| {})
        .expect_err("invalid archive should fail extraction");
    assert!(!err.is_empty(), "expected non-empty extract error");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scan_library_mod_path_blocking_returns_expected_scan_results() {
    let root = temp_test_dir("scan_library");
    let library_mod_path = root.join("library_mod");
    fs::create_dir_all(library_mod_path.join("FolderMod")).unwrap();
    fs::create_dir_all(library_mod_path.join("Other")).unwrap();
    fs::create_dir_all(library_mod_path.join("FolderMod").join("MODELS")).unwrap();
    fs::write(
        library_mod_path.join("FolderMod").join("mod_info.json"),
        "{}",
    )
    .unwrap();
    fs::write(
        library_mod_path.join("FolderMod").join("payload.pak"),
        "pak",
    )
    .unwrap();
    fs::write(library_mod_path.join("Other").join("readme.txt"), "txt").unwrap();

    let (folders, installables) =
        scan_library_mod_path_blocking(library_mod_path.clone()).expect("scan should succeed");
    assert!(folders.contains(&"FolderMod".to_string()));
    assert!(folders.contains(&"Other".to_string()));
    assert!(installables.contains(&"FolderMod".to_string()));

    let err = scan_library_mod_path_blocking(root.join("missing"))
        .expect_err("missing library root should fail");
    assert!(!err.is_empty(), "expected non-empty scan error");

    fs::remove_dir_all(root).unwrap();
}
