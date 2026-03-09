use super::{
    finalize_installation_command_with, finalize_installation_with_provider,
    get_all_mods_for_render_command_with, get_all_mods_for_render_with_provider,
    get_staging_dir_command_with, get_staging_dir_with_provider,
    install_mod_from_archive_command_with, install_mod_from_archive_with_provider,
    resolve_conflict_command_with, resolve_conflict_with_provider,
};
use crate::models::{InstallProgressPayload, InstallationAnalysis};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[test]
fn resolve_conflict_with_provider_forwards_arguments() {
    let mut called = false;
    resolve_conflict_with_provider(
        "New",
        "Old",
        "/tmp/staging/New",
        true,
        || Some(PathBuf::from("/game")),
        |game_path, new_name, old_name, temp_path, replace| {
            called = true;
            assert_eq!(game_path, std::path::Path::new("/game/GAMEDATA/MODS"));
            assert_eq!(new_name, "New");
            assert_eq!(old_name, "Old");
            assert_eq!(temp_path, std::path::Path::new("/tmp/staging/New"));
            assert!(replace);
            Ok(())
        },
    )
    .expect("expected resolve forwarding");
    assert!(called);
}

#[test]
fn resolve_conflict_with_provider_propagates_game_path_and_resolver_errors() {
    let missing = resolve_conflict_with_provider(
        "New",
        "Old",
        "/tmp/staging/New",
        false,
        || None,
        |_game_path, _new_name, _old_name, _temp_path, _replace| Ok(()),
    );
    assert_eq!(
        missing.unwrap_err(),
        "Could not find game path.".to_string()
    );

    let resolver_err = resolve_conflict_with_provider(
        "New",
        "Old",
        "/tmp/staging/New",
        false,
        || Some(PathBuf::from("/game")),
        |_game_path, _new_name, _old_name, _temp_path, _replace| Err("resolver failed".to_string()),
    );
    assert_eq!(resolver_err.unwrap_err(), "resolver failed");
}

#[test]
fn install_wrapper_sync_helpers_forward_success_and_error() {
    let staging_ok =
        get_staging_dir_with_provider(|| Ok(PathBuf::from("/tmp/staging"))).expect("staging");
    assert_eq!(staging_ok, PathBuf::from("/tmp/staging"));
    assert_eq!(
        get_staging_dir_with_provider(|| Err("staging err".to_string())).unwrap_err(),
        "staging err"
    );

    let render_ok = get_all_mods_for_render_with_provider(|| Ok(Vec::new())).expect("render list");
    assert!(render_ok.is_empty());
    match get_all_mods_for_render_with_provider(|| Err("render err".to_string())) {
        Ok(_) => panic!("expected render error"),
        Err(err) => assert_eq!(err, "render err"),
    }

    let staging_ok =
        get_staging_dir_command_with(|| Ok(PathBuf::from("/tmp/staging2"))).expect("staging");
    assert_eq!(staging_ok, PathBuf::from("/tmp/staging2"));
    assert_eq!(
        get_staging_dir_command_with(|| Err("staging cmd err".to_string())).unwrap_err(),
        "staging cmd err"
    );

    let render_ok = get_all_mods_for_render_command_with(|| Ok(Vec::new())).expect("render list");
    assert!(render_ok.is_empty());
    match get_all_mods_for_render_command_with(|| Err("render cmd err".to_string())) {
        Ok(_) => panic!("expected render error"),
        Err(err) => assert_eq!(err, "render cmd err"),
    }
}

#[test]
fn finalize_installation_with_provider_forwards_inputs_and_results() {
    let result = finalize_installation_with_provider(
        &"runtime",
        "lib-1".to_string(),
        vec!["A".to_string(), "B".to_string()],
        true,
        |runtime, library_id, selected_folders, flatten_paths| {
            assert_eq!(*runtime, "runtime");
            assert_eq!(library_id, "lib-1");
            assert_eq!(selected_folders, vec!["A".to_string(), "B".to_string()]);
            assert!(flatten_paths);
            Ok(InstallationAnalysis {
                successes: vec![],
                conflicts: vec![],
                messy_archive_path: None,
                active_archive_path: None,
                selection_needed: false,
                temp_id: None,
                available_folders: None,
            })
        },
    )
    .expect("expected finalize success");
    assert!(!result.selection_needed);

    let err = finalize_installation_with_provider(
        &(),
        "lib-2".to_string(),
        vec![],
        false,
        |_runtime, _library_id, _selected_folders, _flatten_paths| Err("finalize err".to_string()),
    );
    match err {
        Ok(_) => panic!("expected finalize error"),
        Err(msg) => assert_eq!(msg, "finalize err"),
    }
}

#[test]
fn install_mod_from_archive_with_provider_forwards_arguments() {
    tauri::async_runtime::block_on(async {
        let out = install_mod_from_archive_with_provider(
            "/tmp/mod.zip".to_string(),
            "download-1".to_string(),
            |archive, download| {
                Box::pin(async move {
                    assert_eq!(archive, "/tmp/mod.zip");
                    assert_eq!(download, "download-1");
                    Ok(InstallationAnalysis {
                        successes: vec![],
                        conflicts: vec![],
                        messy_archive_path: None,
                        active_archive_path: None,
                        selection_needed: true,
                        temp_id: Some("temp".to_string()),
                        available_folders: Some(vec!["A".to_string()]),
                    })
                })
            },
        )
        .await
        .expect("install provider should succeed");
        assert!(out.selection_needed);

        let err = install_mod_from_archive_with_provider(
            "/tmp/mod.zip".to_string(),
            "download-2".to_string(),
            |_archive, _download| Box::pin(async { Err("install err".to_string()) }),
        )
        .await;
        match err {
            Ok(_) => panic!("expected install error"),
            Err(msg) => assert_eq!(msg, "install err"),
        }
    });
}

#[test]
fn install_mod_from_archive_command_with_emits_progress_and_forwards_errors() {
    tauri::async_runtime::block_on(async {
        let emitted = Arc::new(Mutex::new(Vec::<InstallProgressPayload>::new()));
        let emitted_out = emitted.clone();

        let out = install_mod_from_archive_command_with(
            "/tmp/anything.zip".to_string(),
            "download-xyz".to_string(),
            || Err("downloads unavailable".to_string()),
            || Ok(PathBuf::from("/tmp/library")),
            move |payload| {
                emitted_out
                    .lock()
                    .expect("lock should succeed")
                    .push(payload)
            },
            |_library_id, _selected_folders, _flatten_paths| {
                Err("finalize should not be called".to_string())
            },
        )
        .await;

        let err = match out {
            Ok(_) => panic!("expected install error"),
            Err(err) => err,
        };
        assert_eq!(err, "downloads unavailable");

        let events = emitted.lock().expect("lock should succeed");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "download-xyz");
        assert_eq!(events[0].step, "Initializing...");
        assert_eq!(events[0].progress, None);
    });
}

#[test]
fn install_mod_from_archive_command_with_propagates_library_dir_error() {
    tauri::async_runtime::block_on(async {
        let emitted = Arc::new(Mutex::new(Vec::<InstallProgressPayload>::new()));
        let emitted_out = emitted.clone();

        let out = install_mod_from_archive_command_with(
            "/tmp/archive.zip".to_string(),
            "download-library-fail".to_string(),
            || Ok(PathBuf::from("/tmp/downloads")),
            || Err("library unavailable".to_string()),
            move |payload| {
                emitted_out
                    .lock()
                    .expect("lock should succeed")
                    .push(payload)
            },
            |_library_id, _selected_folders, _flatten_paths| {
                Ok(InstallationAnalysis {
                    successes: vec![],
                    conflicts: vec![],
                    messy_archive_path: None,
                    active_archive_path: None,
                    selection_needed: false,
                    temp_id: None,
                    available_folders: None,
                })
            },
        )
        .await;
        let err = match out {
            Ok(_) => panic!("expected library error"),
            Err(err) => err,
        };
        assert_eq!(err, "library unavailable");

        let events = emitted.lock().expect("lock should succeed");
        assert!(!events.is_empty());
        assert_eq!(events[0].id, "download-library-fail");
        assert_eq!(events[0].step, "Initializing...");
    });
}

#[test]
fn install_command_wrappers_forward_arguments_and_results() {
    let out = finalize_installation_command_with(
        &"runtime",
        "lib-9".to_string(),
        vec!["A".to_string()],
        false,
        |runtime, library_id, selected_folders, flatten_paths| {
            assert_eq!(*runtime, "runtime");
            assert_eq!(library_id, "lib-9");
            assert_eq!(selected_folders, vec!["A".to_string()]);
            assert!(!flatten_paths);
            Ok(InstallationAnalysis {
                successes: vec![],
                conflicts: vec![],
                messy_archive_path: None,
                active_archive_path: None,
                selection_needed: false,
                temp_id: None,
                available_folders: None,
            })
        },
    )
    .expect("finalize command wrapper");
    assert!(!out.selection_needed);

    resolve_conflict_command_with(
        "New",
        "Old",
        "/tmp/staging/New",
        true,
        || Some(PathBuf::from("/game")),
        |game_path, new_name, old_name, temp_path, replace| {
            assert_eq!(game_path, std::path::Path::new("/game/GAMEDATA/MODS"));
            assert_eq!(new_name, "New");
            assert_eq!(old_name, "Old");
            assert_eq!(temp_path, std::path::Path::new("/tmp/staging/New"));
            assert!(replace);
            Ok(())
        },
    )
    .expect("resolve command wrapper");
}
