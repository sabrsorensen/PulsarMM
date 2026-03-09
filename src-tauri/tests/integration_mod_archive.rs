use pulsar::mods::archive::extract_archive;
use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

fn current_dir() -> PathBuf {
    std::env::current_dir().expect("cwd should be readable")
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_it_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
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
        .expect("failed to write zip entry content");
    writer.finish().expect("failed to finalize zip");
}

enum ZipEntry<'a> {
    Dir(&'a str),
    File(&'a str, &'a [u8]),
}

fn create_zip_with_entries(zip_path: &Path, entries: &[ZipEntry<'_>]) {
    let file = fs::File::create(zip_path).expect("failed to create zip");
    let mut writer = zip::ZipWriter::new(file);
    for entry in entries {
        match entry {
            ZipEntry::Dir(path) => writer
                .add_directory(*path, SimpleFileOptions::default())
                .expect("failed to add zip directory"),
            ZipEntry::File(path, content) => {
                writer
                    .start_file(*path, SimpleFileOptions::default())
                    .expect("failed to start zip entry");
                writer
                    .write_all(content)
                    .expect("failed to write zip entry content");
            }
        }
    }
    writer.finish().expect("failed to finalize zip");
}

fn create_7z_with_file(archive_path: &Path, source_path: &Path, entry_name: &str) {
    let mut writer = SevenZWriter::create(archive_path).expect("failed to create 7z");
    let entry = SevenZArchiveEntry::from_path(source_path, entry_name.to_string());
    let file = fs::File::open(source_path).expect("failed to open 7z source");
    writer
        .push_archive_entry(entry, Some(file))
        .expect("failed to add 7z entry");
    writer.finish().expect("failed to finalize 7z");
}

fn ignore_progress(_: u64) {}

#[test]
fn extract_archive_rejects_unsupported_extension() {
    let root = temp_test_dir("extract_unsupported");
    let src = root.join("mod.txt");
    let dest = root.join("out");
    fs::write(&src, "nope").expect("failed to write source");

    let mut on_progress = ignore_progress;
    let err =
        extract_archive(&src, &dest, &mut on_progress).expect_err("expected unsupported extension");
    assert!(err.contains("Unsupported file type"));

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_unpacks_zip_with_progress() {
    let root = temp_test_dir("extract_zip");
    let zip_path = root.join("mod.zip");
    let dest = root.join("out");
    create_zip_with_file(&zip_path, "MyMod/file.mbin", b"abc123");

    let mut progress_updates = Vec::new();
    let mut on_progress = |pct| progress_updates.push(pct);
    extract_archive(&zip_path, &dest, &mut on_progress).expect("zip extraction should succeed");

    let extracted = dest.join("MyMod").join("file.mbin");
    assert!(extracted.exists(), "extracted file should exist");
    assert_eq!(
        fs::read_to_string(extracted).expect("failed to read extracted file"),
        "abc123"
    );
    assert!(
        progress_updates.last().is_some_and(|v| *v == 100),
        "last progress update should be 100"
    );

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_rejects_invalid_zip_content() {
    let root = temp_test_dir("extract_bad_zip");
    let src = root.join("mod.zip");
    let dest = root.join("out");
    fs::write(&src, "not-a-zip").expect("failed to write fake zip");

    let mut on_progress = ignore_progress;
    let err = extract_archive(&src, &dest, &mut on_progress).expect_err("expected zip parse error");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_creates_zip_dirs_and_skips_unsafe_entries() {
    let root = temp_test_dir("extract_zip_dirs");
    let zip_path = root.join("mod.zip");
    let dest = root.join("out");
    create_zip_with_entries(
        &zip_path,
        &[
            ZipEntry::File("../escape.mbin", b"nope"),
            ZipEntry::Dir("Safe/"),
            ZipEntry::File("Safe/file.mbin", b"abc123"),
        ],
    );

    let mut progress_updates = Vec::new();
    let mut on_progress = |pct| progress_updates.push(pct);
    extract_archive(&zip_path, &dest, &mut on_progress).expect("zip extraction should succeed");

    assert_eq!(
        fs::read_to_string(dest.join("Safe").join("file.mbin"))
            .expect("safe extracted file should exist"),
        "abc123"
    );
    assert!(
        !dest.join("escape.mbin").exists(),
        "unsafe zip path should be skipped"
    );
    assert_eq!(progress_updates.last().copied(), Some(100));

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_reports_destination_create_errors() {
    let root = temp_test_dir("extract_dest_create_err");
    let zip_path = root.join("mod.zip");
    let blocking = root.join("blocking");
    fs::write(&blocking, "file").expect("failed to write blocking file");
    create_zip_with_file(&zip_path, "MyMod/file.mbin", b"abc123");

    let mut on_progress = ignore_progress;
    let err = extract_archive(&zip_path, &blocking.join("out"), &mut on_progress)
        .expect_err("expected destination create error");
    assert!(err.contains("Could not create dest dir"));

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_reports_zip_output_create_errors_when_destination_is_file() {
    let root = temp_test_dir("extract_dest_file_conflict");
    let zip_path = root.join("mod.zip");
    let dest = root.join("out");
    fs::write(&dest, "blocking file").expect("failed to create blocking destination");
    create_zip_with_file(&zip_path, "MyMod/file.mbin", b"abc123");

    let mut on_progress = ignore_progress;
    let err = extract_archive(&zip_path, &dest, &mut on_progress)
        .expect_err("expected zip output create error for file destination");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_reports_zip_output_path_conflicts() {
    let root = temp_test_dir("extract_zip_conflict");
    let zip_path = root.join("mod.zip");
    let dest = root.join("out");
    create_zip_with_entries(
        &zip_path,
        &[
            ZipEntry::File("dir", b"abc"),
            ZipEntry::File("dir/file.mbin", b"def"),
        ],
    );

    let mut on_progress = ignore_progress;
    let err = extract_archive(&zip_path, &dest, &mut on_progress)
        .expect_err("expected file/directory conflict during extraction");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_reports_invalid_archive_path() {
    let root = temp_test_dir("extract_missing_zip");
    let src = root.join("missing.zip");
    let dest = root.join("out");

    let mut on_progress = ignore_progress;
    let err = extract_archive(&src, &dest, &mut on_progress)
        .expect_err("expected invalid archive path error");
    assert!(err.contains("Invalid archive path"));

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_reports_zip_open_errors_for_directory_archives() {
    let root = temp_test_dir("extract_zip_open_err");
    let src = root.join("mod.zip");
    let dest = root.join("out");
    fs::create_dir_all(&src).expect("failed to create directory archive placeholder");

    let mut on_progress = ignore_progress;
    let err = extract_archive(&src, &dest, &mut on_progress)
        .expect_err("directory archive path should fail to open as zip");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_7z_reports_decompression_errors() {
    let root = temp_test_dir("extract_bad_7z");
    let src = root.join("mod.7z");
    let dest = root.join("out");
    fs::write(&src, "not-a-7z").expect("failed to write fake 7z");

    let mut progress_updates = Vec::new();
    let mut on_progress = |pct| progress_updates.push(pct);
    let err = extract_archive(&src, &dest, &mut on_progress).expect_err("expected 7z error");
    assert!(!err.is_empty());
    assert_eq!(progress_updates.first().copied(), Some(50));

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_7z_unpacks_with_progress() {
    let root = temp_test_dir("extract_good_7z");
    let src = root.join("source.txt");
    let archive = root.join("mod.7z");
    let dest = root.join("out");
    fs::write(&src, "abc123").expect("failed to write 7z source");
    create_7z_with_file(&archive, &src, "MyMod/file.mbin");

    let mut progress_updates = Vec::new();
    let mut on_progress = |pct| progress_updates.push(pct);
    extract_archive(&archive, &dest, &mut on_progress).expect("7z extraction should succeed");

    assert_eq!(
        fs::read_to_string(dest.join("MyMod").join("file.mbin"))
            .expect("extracted 7z file should exist"),
        "abc123"
    );
    assert_eq!(progress_updates, vec![50, 100]);

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_rar_reports_destination_change_errors_when_destination_is_file() {
    let root = temp_test_dir("extract_bad_rar_dest");
    let src = root.join("mod.rar");
    let dest = root.join("out");
    fs::write(&src, "not-a-rar").expect("failed to write fake rar");
    fs::write(&dest, "blocking file").expect("failed to create blocking destination");

    let mut on_progress = ignore_progress;
    let err = extract_archive(&src, &dest, &mut on_progress)
        .expect_err("file destination should fail before rar extraction");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn extract_archive_rar_restores_working_dir_on_error() {
    let root = temp_test_dir("extract_bad_rar");
    let src = root.join("mod.rar");
    let dest = root.join("out");
    fs::write(&src, "not-a-rar").expect("failed to write fake rar");

    let cwd_before = current_dir();
    let mut on_progress = ignore_progress;
    let err = extract_archive(&src, &dest, &mut on_progress).expect_err("expected rar error");
    assert!(!err.is_empty());
    let cwd_after = current_dir();
    assert_eq!(
        cwd_after, cwd_before,
        "working directory should be restored"
    );

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}
