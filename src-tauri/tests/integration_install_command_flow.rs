use pulsar::models::{InstallProgressPayload, InstallationAnalysis};
use pulsar::mods::install_command_flow::install_mod_from_archive_with;
use pulsar::mods::install_command_runtime::install_mod_from_archive_with_progress_events;
use pulsar::mods::install_command_runtime::{
    copy_archive_to_downloads_async, extract_archive_if_needed_async, scan_library_mod_path_async,
};
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_it_install_command_flow_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
    dir
}

fn create_zip_with_file(zip_path: &std::path::Path, internal_path: &str, content: &[u8]) {
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

#[test]
fn install_mod_from_archive_with_invokes_finalize_for_detected_default_installable() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("selection");
        let downloads = root.join("downloads");
        let library = root.join("library");
        fs::create_dir_all(&downloads).expect("failed to create downloads dir");
        fs::create_dir_all(&library).expect("failed to create library dir");

        let archive = downloads.join("Example.zip");
        fs::write(&archive, "zip-bytes").expect("failed to create archive");
        fs::create_dir_all(library.join("Example.zip_unpacked"))
            .expect("failed to create unpacked dir");
        let finalize_called = RefCell::new(false);

        let result = install_mod_from_archive_with(
            archive.to_string_lossy().into_owned(),
            || Ok(downloads.clone()),
            || Ok(library.clone()),
            |_step| {},
            |_pct| {},
            |library_id, selected_folders, flatten_paths| -> Result<InstallationAnalysis, String> {
                *finalize_called.borrow_mut() = true;
                assert_eq!(library_id, "Example.zip_unpacked");
                assert!(selected_folders.is_empty());
                assert!(!flatten_paths);
                Ok(InstallationAnalysis {
                    successes: Vec::new(),
                    conflicts: Vec::new(),
                    messy_archive_path: None,
                    active_archive_path: None,
                    selection_needed: false,
                    temp_id: None,
                    available_folders: None,
                })
            },
        )
        .await
        .expect("install should return finalized analysis");

        assert!(*finalize_called.borrow());
        assert!(!result.selection_needed);
        assert_eq!(
            result.active_archive_path.as_deref(),
            Some(archive.to_string_lossy().as_ref())
        );

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn install_mod_from_archive_with_propagates_downloads_dir_error() {
    tauri::async_runtime::block_on(async {
        let seen_steps = RefCell::new(Vec::<String>::new());
        let out = install_mod_from_archive_with(
            "/tmp/anything.zip".to_string(),
            || Err("downloads unavailable".to_string()),
            || Ok(PathBuf::from("/library")),
            |step| seen_steps.borrow_mut().push(step.to_string()),
            |_pct| {},
            |_library_id,
             _selected_folders,
             _flatten_paths|
             -> Result<InstallationAnalysis, String> {
                Err("should not finalize".to_string())
            },
        )
        .await;

        match out {
            Err(err) => assert_eq!(err, "downloads unavailable"),
            Ok(_) => panic!("expected downloads-dir error"),
        }
        assert_eq!(seen_steps.into_inner(), vec!["Initializing...".to_string()]);
    });
}

#[test]
fn install_mod_from_archive_with_progress_events_emits_structured_payloads() {
    tauri::async_runtime::block_on(async {
        let emitted = Arc::new(Mutex::new(Vec::<InstallProgressPayload>::new()));
        let emitted_out = emitted.clone();
        let out = install_mod_from_archive_with_progress_events(
            "/tmp/anything.zip".to_string(),
            "download-1".to_string(),
            || Err("downloads unavailable".to_string()),
            || Ok(PathBuf::from("/library")),
            move |payload| {
                emitted_out
                    .lock()
                    .expect("lock should succeed")
                    .push(payload)
            },
            |_library_id,
             _selected_folders,
             _flatten_paths|
             -> Result<InstallationAnalysis, String> {
                Err("should not finalize".to_string())
            },
        )
        .await;

        match out {
            Err(err) => assert_eq!(err, "downloads unavailable"),
            Ok(_) => panic!("expected downloads-dir error"),
        }

        let events = emitted.lock().expect("lock should succeed");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "download-1");
        assert_eq!(events[0].step, "Initializing...");
        assert_eq!(events[0].progress, None);
    });
}

#[test]
fn install_mod_from_archive_with_propagates_library_dir_error() {
    tauri::async_runtime::block_on(async {
        let seen_steps = RefCell::new(Vec::<String>::new());
        let out = install_mod_from_archive_with(
            "/tmp/anything.zip".to_string(),
            || Ok(PathBuf::from("/downloads")),
            || Err("library unavailable".to_string()),
            |step| seen_steps.borrow_mut().push(step.to_string()),
            |_pct| {},
            |_library_id,
             _selected_folders,
             _flatten_paths|
             -> Result<InstallationAnalysis, String> {
                Err("should not finalize".to_string())
            },
        )
        .await;

        match out {
            Err(err) => assert_eq!(err, "library unavailable"),
            Ok(_) => panic!("expected library-dir error"),
        }
        assert_eq!(seen_steps.into_inner(), vec!["Initializing...".to_string()]);
    });
}

#[test]
fn install_mod_from_archive_with_returns_selection_analysis_without_finalizing() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("selection_needed");
        let downloads = root.join("downloads");
        let library = root.join("library");
        fs::create_dir_all(&downloads).expect("failed to create downloads dir");
        fs::create_dir_all(&library).expect("failed to create library dir");

        let archive = downloads.join("Bundle.zip");
        fs::write(&archive, "zip-bytes").expect("failed to create archive");
        fs::create_dir_all(library.join("Bundle.zip_unpacked/OptionA/UI"))
            .expect("failed to create OptionA");
        fs::create_dir_all(library.join("Bundle.zip_unpacked/OptionB/METADATA"))
            .expect("failed to create OptionB");

        let finalize_called = RefCell::new(false);
        let seen_steps = RefCell::new(Vec::<String>::new());
        let result = install_mod_from_archive_with(
            archive.to_string_lossy().into_owned(),
            || Ok(downloads.clone()),
            || Ok(library.clone()),
            |step| seen_steps.borrow_mut().push(step.to_string()),
            |_pct| {},
            |_library_id,
             _selected_folders,
             _flatten_paths|
             -> Result<InstallationAnalysis, String> {
                *finalize_called.borrow_mut() = true;
                Err("finalize should not run".to_string())
            },
        )
        .await
        .expect("selection-needed analysis should be returned");

        assert!(!*finalize_called.borrow());
        assert!(result.selection_needed);
        assert_eq!(result.temp_id.as_deref(), Some("Bundle.zip_unpacked"));
        let mut available = result
            .available_folders
            .expect("selection-needed analysis should include folder list");
        available.sort();
        assert_eq!(
            available,
            vec!["OptionA".to_string(), "OptionB".to_string()]
        );
        assert_eq!(
            seen_steps.into_inner(),
            vec![
                "Initializing...".to_string(),
                "Copying to library...".to_string(),
                "Analyzing structure...".to_string(),
                "Waiting for selection...".to_string()
            ]
        );

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn copy_archive_to_downloads_async_copies_file() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_copy_archive");
        let src_dir = root.join("src");
        let dst_dir = root.join("downloads");
        fs::create_dir_all(&src_dir).expect("failed to create src dir");
        fs::create_dir_all(&dst_dir).expect("failed to create downloads dir");
        let archive = src_dir.join("test.zip");
        fs::write(&archive, "zip-bytes").expect("failed to write archive");

        let out = copy_archive_to_downloads_async(archive.clone(), dst_dir.clone())
            .await
            .expect("copy should succeed");
        assert_eq!(out, dst_dir.join("test.zip"));
        assert!(out.exists());

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn extract_archive_if_needed_async_skips_when_library_already_exists() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_extract_skip");
        let archive = root.join("missing.zip");
        let library_mod_path = root.join("library").join("existing");
        fs::create_dir_all(&library_mod_path).expect("failed to create existing library path");

        extract_archive_if_needed_async(archive, library_mod_path, Box::new(|_pct| {}))
            .await
            .expect("existing library folder should skip extraction");

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn extract_archive_if_needed_async_extracts_archive_and_reports_progress() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_extract_success");
        let archive = root.join("mod.zip");
        let library_mod_path = root.join("library").join("Example");
        create_zip_with_file(&archive, "Example/file.pak", b"zip-bytes");

        let progress = Arc::new(Mutex::new(Vec::<u64>::new()));
        let progress_out = progress.clone();
        extract_archive_if_needed_async(
            archive,
            library_mod_path.clone(),
            Box::new(move |pct| {
                progress_out
                    .lock()
                    .expect("progress lock should succeed")
                    .push(pct);
            }),
        )
        .await
        .expect("archive extraction should succeed");

        assert_eq!(
            fs::read(library_mod_path.join("Example").join("file.pak"))
                .expect("extracted file should exist"),
            b"zip-bytes"
        );
        assert_eq!(
            progress
                .lock()
                .expect("progress lock should succeed")
                .last(),
            Some(&100)
        );

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn scan_library_mod_path_async_handles_empty_library_folder() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_scan_empty");
        fs::create_dir_all(&root).expect("failed to create library root");

        let (folders, installables) = scan_library_mod_path_async(root.clone())
            .await
            .expect("scan should succeed");
        assert!(folders.is_empty());
        assert!(installables.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn copy_archive_to_downloads_async_propagates_copy_error() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_copy_err");
        let downloads = root.join("downloads");
        fs::create_dir_all(&downloads).expect("failed to create downloads dir");

        let err = copy_archive_to_downloads_async(root.join("missing.zip"), downloads)
            .await
            .expect_err("missing archive should fail");
        assert!(!err.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn extract_archive_if_needed_async_propagates_extract_error() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_extract_err");
        let archive = root.join("missing.zip");
        let library_mod_path = root.join("library").join("new-mod");

        let err = extract_archive_if_needed_async(archive, library_mod_path, Box::new(|_pct| {}))
            .await
            .expect_err("missing archive should fail extraction");
        assert!(!err.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}

#[test]
fn scan_library_mod_path_async_propagates_read_dir_error() {
    tauri::async_runtime::block_on(async {
        let root = temp_test_dir("runtime_scan_err");
        let missing = root.join("missing");

        let err = scan_library_mod_path_async(missing)
            .await
            .expect_err("missing library root should fail");
        assert!(!err.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup");
    });
}
