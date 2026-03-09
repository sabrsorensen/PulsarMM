use super::{
    finalize_installation_command_entry_with, finalize_installation_runtime_with,
    get_all_mods_for_render_command_entry_with, get_all_mods_for_render_runtime_with,
    get_staging_dir_command_entry_with, get_staging_dir_runtime_with,
    install_mod_from_archive_command_entry_with, install_mod_from_archive_runtime_with,
    resolve_conflict_command_entry_with, resolve_conflict_runtime_with,
};
use crate::models::{InstallProgressPayload, InstallationAnalysis, LocalModInfo, ModRenderData};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[test]
fn install_commands_runtime_sync_helpers_forward_and_propagate() {
    let runtime = "runtime";
    let staging = get_staging_dir_runtime_with(&runtime, |rt| {
        assert_eq!(*rt, "runtime");
        Ok(PathBuf::from("/tmp/staging"))
    })
    .expect("staging should succeed");
    assert_eq!(staging, PathBuf::from("/tmp/staging"));

    let mods = get_all_mods_for_render_runtime_with(&runtime, |rt| {
        assert_eq!(*rt, "runtime");
        Ok(vec![ModRenderData {
            folder_name: "A".to_string(),
            enabled: true,
            priority: 0,
            local_info: Some(LocalModInfo {
                folder_name: "A".to_string(),
                mod_id: Some("1".to_string()),
                file_id: Some("2".to_string()),
                version: Some("1.0".to_string()),
                install_source: Some("nexus".to_string()),
            }),
        }])
    })
    .expect("mods should succeed");
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].folder_name, "A");

    let finalized = finalize_installation_runtime_with(
        &runtime,
        "lib".to_string(),
        vec!["A".to_string()],
        false,
        |rt, lib, folders, flatten| {
            assert_eq!(*rt, "runtime");
            assert_eq!(lib, "lib");
            assert_eq!(folders, vec!["A".to_string()]);
            assert!(!flatten);
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
    .expect("finalize should succeed");
    assert!(!finalized.selection_needed);

    resolve_conflict_runtime_with(
        "New",
        "Old",
        "/tmp/New",
        true,
        || Some(PathBuf::from("/game")),
        |mods_path, new_name, old_name, temp_path, replace| {
            assert_eq!(mods_path, Path::new("/game/GAMEDATA/MODS"));
            assert_eq!(new_name, "New");
            assert_eq!(old_name, "Old");
            assert_eq!(temp_path, Path::new("/tmp/New"));
            assert!(replace);
            Ok(())
        },
    )
    .expect("resolve should succeed");
}

#[test]
fn install_commands_runtime_sync_helpers_propagate_errors() {
    let runtime = ();
    let err = get_staging_dir_runtime_with(&runtime, |_rt| Err("staging err".to_string()))
        .expect_err("staging error should bubble");
    assert_eq!(err, "staging err");

    let err =
        match get_all_mods_for_render_runtime_with(&runtime, |_rt| Err("mods err".to_string())) {
            Ok(_) => panic!("expected mods error"),
            Err(err) => err,
        };
    assert_eq!(err, "mods err");

    let err = match finalize_installation_runtime_with(
        &runtime,
        "lib".to_string(),
        vec![],
        true,
        |_rt, _lib, _folders, _flatten| Err("finalize err".to_string()),
    ) {
        Ok(_) => panic!("expected finalize error"),
        Err(err) => err,
    };
    assert_eq!(err, "finalize err");

    let err = resolve_conflict_runtime_with(
        "New",
        "Old",
        "/tmp/New",
        false,
        || None,
        |_mods_path, _new_name, _old_name, _temp_path, _replace| Ok(()),
    )
    .expect_err("missing game path should bubble");
    assert_eq!(err, "Could not find game path.");
}

#[test]
fn install_mod_from_archive_runtime_with_emits_progress_and_propagates_errors() {
    tauri::async_runtime::block_on(async {
        let emitted = Arc::new(Mutex::new(Vec::<InstallProgressPayload>::new()));
        let emitted_out = emitted.clone();
        let err = match install_mod_from_archive_runtime_with(
            "/tmp/archive.zip".to_string(),
            "dl-1".to_string(),
            || Err("downloads err".to_string()),
            || Ok(PathBuf::from("/tmp/library")),
            move |payload| emitted_out.lock().expect("emitted lock").push(payload),
            |_lib_id, _folders, _flatten| Err("finalize should not be called".to_string()),
        )
        .await
        {
            Ok(_) => panic!("expected downloads error"),
            Err(err) => err,
        };
        assert_eq!(err, "downloads err");

        let events = emitted.lock().expect("emitted lock");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "dl-1");
        assert_eq!(events[0].step, "Initializing...");
    });
}

#[test]
fn install_command_entries_forward_to_runtime_helpers() {
    let runtime = "runtime";
    let staging = get_staging_dir_command_entry_with(&runtime, |rt| {
        assert_eq!(*rt, "runtime");
        Ok(PathBuf::from("/tmp/staging-entry"))
    })
    .expect("staging entry should succeed");
    assert_eq!(staging, PathBuf::from("/tmp/staging-entry"));

    let mods = get_all_mods_for_render_command_entry_with(&runtime, |rt| {
        assert_eq!(*rt, "runtime");
        Ok(Vec::<ModRenderData>::new())
    })
    .expect("mods entry should succeed");
    assert!(mods.is_empty());

    let finalized = finalize_installation_command_entry_with(
        &runtime,
        "lib-entry".to_string(),
        vec!["A".to_string()],
        true,
        |rt, lib, folders, flatten| {
            assert_eq!(*rt, "runtime");
            assert_eq!(lib, "lib-entry");
            assert_eq!(folders, vec!["A".to_string()]);
            assert!(flatten);
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
    .expect("finalize entry should succeed");
    assert!(!finalized.selection_needed);

    resolve_conflict_command_entry_with(
        "New",
        "Old",
        "/tmp/New",
        true,
        || Some(PathBuf::from("/game")),
        |mods_path, new_name, old_name, temp_path, replace| {
            assert_eq!(mods_path, Path::new("/game/GAMEDATA/MODS"));
            assert_eq!(new_name, "New");
            assert_eq!(old_name, "Old");
            assert_eq!(temp_path, Path::new("/tmp/New"));
            assert!(replace);
            Ok(())
        },
    )
    .expect("resolve entry should succeed");
}

#[test]
fn install_mod_from_archive_command_entry_with_propagates_download_error() {
    tauri::async_runtime::block_on(async {
        let emitted = Arc::new(Mutex::new(Vec::<InstallProgressPayload>::new()));
        let emitted_out = emitted.clone();
        let err = match install_mod_from_archive_command_entry_with(
            "/tmp/archive.zip".to_string(),
            "dl-entry".to_string(),
            || Err("downloads entry err".to_string()),
            || Ok(PathBuf::from("/tmp/library")),
            move |payload| emitted_out.lock().expect("emitted lock").push(payload),
            |_lib_id, _folders, _flatten| Err("finalize should not be called".to_string()),
        )
        .await
        {
            Ok(_) => panic!("expected downloads entry error"),
            Err(err) => err,
        };
        assert_eq!(err, "downloads entry err");
        let events = emitted.lock().expect("emitted lock");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "dl-entry");
    });
}
