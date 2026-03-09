use pulsar::models::{DownloadResult, ModInfo, ModRenderData};
use pulsar::mods::command_flow::{
    delete_mod_flow, download_mod_archive_flow, ensure_mod_info_flow,
    maybe_persist_renamed_mod_settings, maybe_sync_library_rename_for_mod, rename_mod_folder_flow,
    reorder_mods_flow, update_mod_id_in_json_flow, update_mod_name_in_xml_flow,
};
use pulsar::mods::command_ops::LibraryRenameSync;
use pulsar::mods::info_ops::{
    ensure_mod_info_file, read_mod_info_file, update_mod_id_in_json_file, EnsureModInfoInput,
};
use serde_json::Value;
use std::cell::RefCell;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_it_mod_command_flow_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create integration temp dir");
    dir
}

#[test]
fn maybe_sync_library_rename_logs_success_and_warning() {
    let logs = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
    let logs_ok = logs.clone();
    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| {
            Some(ModInfo {
                mod_id: None,
                file_id: None,
                version: None,
                install_source: Some("archive.zip".to_string()),
            })
        },
        || Ok(PathBuf::from("/lib")),
        |_, _, _, _| Ok(LibraryRenameSync::Renamed),
        |level, msg| {
            logs_ok
                .borrow_mut()
                .push((level.to_string(), msg.to_string()))
        },
    );
    assert_eq!(logs.borrow().len(), 1);
    assert_eq!(logs.borrow()[0].0, "INFO");

    logs.borrow_mut().clear();
    let logs_warn = logs.clone();
    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| {
            Some(ModInfo {
                mod_id: None,
                file_id: None,
                version: None,
                install_source: Some("archive.zip".to_string()),
            })
        },
        || Ok(PathBuf::from("/lib")),
        |_, _, _, _| Err("sync failed".to_string()),
        |level, msg| {
            logs_warn
                .borrow_mut()
                .push((level.to_string(), msg.to_string()))
        },
    );
    assert_eq!(logs.borrow().len(), 1);
    assert_eq!(logs.borrow()[0].0, "WARN");
    assert_eq!(logs.borrow()[0].1, "sync failed");
}

#[test]
fn maybe_sync_library_rename_short_circuits_on_missing_data() {
    let called = Rc::new(RefCell::new(false));
    let called_sync = called.clone();
    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| None,
        || Ok(PathBuf::from("/lib")),
        |_, _, _, _| {
            *called_sync.borrow_mut() = true;
            Ok(LibraryRenameSync::Renamed)
        },
        |_, _| {},
    );
    assert!(!*called.borrow());
}

#[test]
fn maybe_sync_library_rename_covers_noop_and_library_error_paths() {
    let logs = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
    let sync_called = Rc::new(RefCell::new(0usize));

    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| {
            Some(ModInfo {
                mod_id: None,
                file_id: None,
                version: None,
                install_source: None,
            })
        },
        || Ok(PathBuf::from("/lib")),
        |_, _, _, _| {
            *sync_called.borrow_mut() += 1;
            Ok(LibraryRenameSync::Renamed)
        },
        |level, msg| logs.borrow_mut().push((level.to_string(), msg.to_string())),
    );
    assert_eq!(*sync_called.borrow(), 0);
    assert!(logs.borrow().is_empty());

    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| {
            Some(ModInfo {
                mod_id: None,
                file_id: None,
                version: None,
                install_source: Some("archive.zip".to_string()),
            })
        },
        || Err("library missing".to_string()),
        |_, _, _, _| {
            *sync_called.borrow_mut() += 1;
            Ok(LibraryRenameSync::Renamed)
        },
        |level, msg| logs.borrow_mut().push((level.to_string(), msg.to_string())),
    );
    assert_eq!(*sync_called.borrow(), 0);
    assert!(logs.borrow().is_empty());

    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| {
            Some(ModInfo {
                mod_id: None,
                file_id: None,
                version: None,
                install_source: Some("archive.zip".to_string()),
            })
        },
        || Ok(PathBuf::from("/lib")),
        |_, _, _, _| Ok(LibraryRenameSync::SourceMissing),
        |level, msg| logs.borrow_mut().push((level.to_string(), msg.to_string())),
    );
    maybe_sync_library_rename_for_mod(
        Path::new("/tmp/Old"),
        "Old",
        "New",
        |_| {
            Some(ModInfo {
                mod_id: None,
                file_id: None,
                version: None,
                install_source: Some("archive.zip".to_string()),
            })
        },
        || Ok(PathBuf::from("/lib")),
        |_, _, _, _| Ok(LibraryRenameSync::TargetExists),
        |level, msg| logs.borrow_mut().push((level.to_string(), msg.to_string())),
    );
    assert!(
        logs.borrow().is_empty(),
        "source-missing and target-exists should be silent no-ops"
    );
}

#[test]
fn maybe_persist_renamed_mod_settings_updates_when_present() {
    let dir = temp_test_dir("persist");
    let settings = dir.join("GCMODSETTINGS.MXML");
    fs::write(&settings, "<xml/>").unwrap();

    let saved = Rc::new(RefCell::new(None::<(String, String)>));
    let saved_out = saved.clone();
    maybe_persist_renamed_mod_settings(
        &settings,
        "Old".to_string(),
        "New".to_string(),
        |old, new| Ok(format!("{}->{}", old, new)),
        |path, xml| {
            *saved_out.borrow_mut() = Some((path, xml));
            Ok(())
        },
        |_, _| {},
    );
    let saved_value = saved.borrow().clone().expect("expected save call");
    assert!(saved_value.0.ends_with("GCMODSETTINGS.MXML"));
    assert_eq!(saved_value.1, "Old->New");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn maybe_persist_renamed_mod_settings_covers_missing_update_error_and_save_error() {
    let dir = temp_test_dir("persist_edges");
    let missing = dir.join("MISSING.MXML");
    let update_called = Rc::new(RefCell::new(false));
    let update_called_out = update_called.clone();
    maybe_persist_renamed_mod_settings(
        &missing,
        "Old".to_string(),
        "New".to_string(),
        |_, _| {
            *update_called_out.borrow_mut() = true;
            Ok("<xml/>".to_string())
        },
        |_, _| Ok(()),
        |_, _| {},
    );
    assert!(
        !*update_called.borrow(),
        "missing settings file should short-circuit"
    );

    let settings = dir.join("GCMODSETTINGS.MXML");
    fs::write(&settings, "<xml/>").unwrap();
    let logs = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
    let logs_out = logs.clone();
    maybe_persist_renamed_mod_settings(
        &settings,
        "Old".to_string(),
        "New".to_string(),
        |_, _| Err("update failed".to_string()),
        |_, _| Ok(()),
        |level, msg| {
            logs_out
                .borrow_mut()
                .push((level.to_string(), msg.to_string()))
        },
    );
    assert_eq!(logs.borrow().len(), 1);
    assert_eq!(logs.borrow()[0].0, "WARN");
    assert!(logs.borrow()[0]
        .1
        .contains("Folder renamed, but XML update failed"));

    logs.borrow_mut().clear();
    let save_called = Rc::new(RefCell::new(false));
    let save_called_out = save_called.clone();
    maybe_persist_renamed_mod_settings(
        &settings,
        "Old".to_string(),
        "New".to_string(),
        |old, new| Ok(format!("{}->{}", old, new)),
        |_, _| {
            *save_called_out.borrow_mut() = true;
            Err("save failed".to_string())
        },
        |level, msg| logs.borrow_mut().push((level.to_string(), msg.to_string())),
    );
    assert!(
        *save_called.borrow(),
        "save callback should still be invoked when update succeeds"
    );
    assert!(
        logs.borrow().is_empty(),
        "save failures are intentionally ignored here"
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rename_mod_folder_flow_covers_success_and_error() {
    let dir = temp_test_dir("rename");
    let game = dir.join("game");
    let mods = game.join("GAMEDATA").join("MODS");
    fs::create_dir_all(&mods).unwrap();
    let old = mods.join("Old");
    fs::create_dir_all(&old).unwrap();

    let renamed = Rc::new(RefCell::new(None::<(PathBuf, PathBuf)>));
    let renamed_out = renamed.clone();
    let result = rename_mod_folder_flow(
        "Old".to_string(),
        "New".to_string(),
        || Some(game.clone()),
        |base, name| base.join("GAMEDATA").join("MODS").join(name),
        |old_exists, new_exists| {
            if !old_exists {
                Err("missing old".to_string())
            } else if new_exists {
                Err("new exists".to_string())
            } else {
                Ok(())
            }
        },
        |_, _, _| {},
        |old_path, new_path| {
            *renamed_out.borrow_mut() = Some((old_path.to_path_buf(), new_path.to_path_buf()));
            Ok(())
        },
        |base| {
            base.join("Binaries")
                .join("SETTINGS")
                .join("GCMODSETTINGS.MXML")
        },
        |_, _, _| {},
        || {
            Ok(vec![ModRenderData {
                folder_name: "New".to_string(),
                enabled: true,
                priority: 0,
                local_info: None,
            }])
        },
    )
    .expect("rename flow should succeed");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].folder_name, "New");
    assert!(result[0].enabled);
    assert_eq!(result[0].priority, 0);
    assert!(result[0].local_info.is_none());
    let renamed_value = renamed.borrow().clone().expect("rename call expected");
    assert!(renamed_value.0.ends_with("GAMEDATA/MODS/Old"));
    assert!(renamed_value.1.ends_with("GAMEDATA/MODS/New"));

    let err = rename_mod_folder_flow(
        "Old".to_string(),
        "New".to_string(),
        || None,
        |_, _| PathBuf::new(),
        |_, _| Ok(()),
        |_, _, _| {},
        |_, _| Ok(()),
        |_| PathBuf::new(),
        |_, _, _| {},
        || Ok(vec![]),
    );
    let err = match err {
        Ok(_) => panic!("missing game path should fail"),
        Err(e) => e,
    };
    assert_eq!(err, "Could not find game path.");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rename_mod_folder_flow_propagates_validation_rename_and_render_errors() {
    let err = rename_mod_folder_flow(
        "Old".to_string(),
        "New".to_string(),
        || Some(PathBuf::from("/game")),
        |base, name| base.join("GAMEDATA").join("MODS").join(name),
        |_old_exists, _new_exists| Err("validation failed".to_string()),
        |_, _, _| {},
        |_, _| Ok(()),
        |_| PathBuf::from("/settings.mxml"),
        |_, _, _| {},
        || Ok(vec![]),
    );
    let err = match err {
        Ok(_) => panic!("validation error should bubble"),
        Err(err) => err,
    };
    assert_eq!(err, "validation failed");

    let sync_called = Rc::new(RefCell::new(false));
    let sync_called_out = sync_called.clone();
    let err = rename_mod_folder_flow(
        "Old".to_string(),
        "New".to_string(),
        || Some(PathBuf::from("/game")),
        |base, name| base.join("GAMEDATA").join("MODS").join(name),
        |_, _| Ok(()),
        |_, _, _| *sync_called_out.borrow_mut() = true,
        |_, _| Err("rename failed".to_string()),
        |_| PathBuf::from("/settings.mxml"),
        |_, _, _| panic!("persist should not run when rename fails"),
        || Ok(vec![]),
    );
    let err = match err {
        Ok(_) => panic!("rename error should bubble"),
        Err(err) => err,
    };
    assert_eq!(err, "rename failed");
    assert!(*sync_called.borrow());

    let persisted = Rc::new(RefCell::new(false));
    let persisted_out = persisted.clone();
    let err = rename_mod_folder_flow(
        "Old".to_string(),
        "New".to_string(),
        || Some(PathBuf::from("/game")),
        |base, name| base.join("GAMEDATA").join("MODS").join(name),
        |_, _| Ok(()),
        |_, _, _| {},
        |_, _| Ok(()),
        |_| PathBuf::from("/settings.mxml"),
        |_, _, _| *persisted_out.borrow_mut() = true,
        || Err("render failed".to_string()),
    );
    let err = match err {
        Ok(_) => panic!("render error should bubble"),
        Err(err) => err,
    };
    assert_eq!(err, "render failed");
    assert!(*persisted.borrow());
}

#[test]
fn delete_mod_flow_covers_delete_and_missing_game_path() {
    let logs = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
    let logs_out = logs.clone();
    let deleted = delete_mod_flow(
        "MyMod".to_string(),
        || Some(PathBuf::from("/game")),
        |p| {
            p.join("Binaries")
                .join("SETTINGS")
                .join("GCMODSETTINGS.MXML")
        },
        |p, n| p.join("GAMEDATA").join("MODS").join(n),
        |_, _| Ok(true),
        |_, _| Ok(()),
        |level, msg| {
            logs_out
                .borrow_mut()
                .push((level.to_string(), msg.to_string()))
        },
        || Ok(vec![]),
    )
    .expect("delete should succeed");
    assert!(deleted.is_empty());
    assert_eq!(logs.borrow()[0].0, "INFO");

    let err = delete_mod_flow(
        "MyMod".to_string(),
        || None,
        |_| PathBuf::new(),
        |_, _| PathBuf::new(),
        |_, _| Ok(false),
        |_, _| Ok(()),
        |_, _| {},
        || Ok(vec![]),
    );
    let err = match err {
        Ok(_) => panic!("missing game path should error"),
        Err(e) => e,
    };
    assert_eq!(err, "Could not find game installation path.");
}

#[test]
fn delete_mod_flow_covers_missing_folder_and_error_paths() {
    let logs = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
    let logs_out = logs.clone();
    let deleted = delete_mod_flow(
        "MyMod".to_string(),
        || Some(PathBuf::from("/game")),
        |p| p.join("settings.mxml"),
        |p, n| p.join("GAMEDATA").join("MODS").join(n),
        |_, _| Ok(false),
        |_, _| Ok(()),
        |level, msg| {
            logs_out
                .borrow_mut()
                .push((level.to_string(), msg.to_string()))
        },
        || Ok(vec![]),
    )
    .expect("missing folder should still allow settings deletion");
    assert!(deleted.is_empty());
    assert_eq!(logs.borrow()[0].0, "WARN");

    let err = delete_mod_flow(
        "MyMod".to_string(),
        || Some(PathBuf::from("/game")),
        |p| p.join("settings.mxml"),
        |p, n| p.join("GAMEDATA").join("MODS").join(n),
        |_, _| Err("remove failed".to_string()),
        |_, _| Ok(()),
        |_, _| {},
        || Ok(vec![]),
    );
    let err = match err {
        Ok(_) => panic!("remove failure should bubble"),
        Err(err) => err,
    };
    assert_eq!(err, "remove failed");

    let err = delete_mod_flow(
        "MyMod".to_string(),
        || Some(PathBuf::from("/game")),
        |p| p.join("settings.mxml"),
        |p, n| p.join("GAMEDATA").join("MODS").join(n),
        |_, _| Ok(true),
        |_, _| Err("settings delete failed".to_string()),
        |_, _| {},
        || Ok(vec![]),
    );
    let err = match err {
        Ok(_) => panic!("settings delete failure should bubble"),
        Err(err) => err,
    };
    assert_eq!(err, "settings delete failed");

    let err = delete_mod_flow(
        "MyMod".to_string(),
        || Some(PathBuf::from("/game")),
        |p| p.join("settings.mxml"),
        |p, n| p.join("GAMEDATA").join("MODS").join(n),
        |_, _| Ok(true),
        |_, _| Ok(()),
        |_, _| {},
        || Err("render failed".to_string()),
    );
    let err = match err {
        Ok(_) => panic!("render failure should bubble"),
        Err(err) => err,
    };
    assert_eq!(err, "render failed");
}

#[test]
fn reorder_and_update_name_flows_use_settings_path() {
    let reordered = reorder_mods_flow(
        &["A".to_string()],
        || Some(PathBuf::from("/game")),
        |p| p.join("settings.mxml"),
        |path, names| Ok(format!("{}:{}", path.display(), names.len())),
    )
    .expect("reorder should work");
    assert!(reordered.contains("/game/settings.mxml:1"));

    let renamed = update_mod_name_in_xml_flow(
        "old",
        "new",
        || Some(PathBuf::from("/game")),
        |p| p.join("settings.mxml"),
        |path, old, new| Ok(format!("{}:{}->{}", path.display(), old, new)),
    )
    .expect("rename should work");
    assert!(renamed.contains("/game/settings.mxml:old->new"));
}

#[test]
fn reorder_and_update_name_flows_require_game_path() {
    let err = reorder_mods_flow(
        &["A".to_string()],
        || None,
        |_| PathBuf::new(),
        |_, _| Ok(String::new()),
    )
    .expect_err("missing game path should fail reorder");
    assert_eq!(err, "Could not find game installation path.");

    let err = update_mod_name_in_xml_flow(
        "old",
        "new",
        || None,
        |_| PathBuf::new(),
        |_, _, _| Ok(String::new()),
    )
    .expect_err("missing game path should fail rename");
    assert_eq!(err, "Could not find game installation path.");
}

#[test]
fn update_mod_id_and_ensure_mod_info_flows_require_game_path() {
    let err = update_mod_id_in_json_flow("Mod", "id", || None, |_, _, _| Ok(()))
        .expect_err("missing game path should fail");
    assert_eq!(err, "Could not find game installation path.");

    let input = EnsureModInfoInput {
        mod_id: "1".to_string(),
        file_id: "2".to_string(),
        version: "1.0".to_string(),
        install_source: "archive.zip".to_string(),
    };
    let err = ensure_mod_info_flow("Mod", &input, || None, |_, _, _| Ok(()))
        .expect_err("missing game path should fail");
    assert_eq!(err, "Could not find game installation path.");
}

#[test]
fn update_mod_id_and_ensure_mod_info_flows_forward_success_paths() {
    update_mod_id_in_json_flow(
        "Mod",
        "id-1",
        || Some(PathBuf::from("/game")),
        |game, folder, id| {
            assert_eq!(game, Path::new("/game"));
            assert_eq!(folder, "Mod");
            assert_eq!(id, "id-1");
            Ok(())
        },
    )
    .expect("update_mod_id flow should forward arguments");

    let input = EnsureModInfoInput {
        mod_id: "1".to_string(),
        file_id: "2".to_string(),
        version: "1.0".to_string(),
        install_source: "archive.zip".to_string(),
    };
    ensure_mod_info_flow(
        "Mod",
        &input,
        || Some(PathBuf::from("/game")),
        |game, folder, seen| {
            assert_eq!(game, Path::new("/game"));
            assert_eq!(folder, "Mod");
            assert_eq!(seen.mod_id, "1");
            assert_eq!(seen.file_id, "2");
            assert_eq!(seen.version, "1.0");
            assert_eq!(seen.install_source, "archive.zip");
            Ok(())
        },
    )
    .expect("ensure_mod_info flow should forward arguments");
}

#[test]
fn download_mod_archive_flow_downloads_and_logs_errors() {
    tauri::async_runtime::block_on(async {
        let logs = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
        let logs_out = logs.clone();
        let ok = download_mod_archive_flow(
            "mod.zip",
            "https://example.com/mod.zip",
            Some("id-1"),
            || Ok(PathBuf::from("/downloads")),
            |_, path, _| async move {
                Ok(DownloadResult {
                    path: path.to_string_lossy().to_string(),
                    size: 1,
                    created_at: 2,
                })
            },
            |level, msg| {
                logs_out
                    .borrow_mut()
                    .push((level.to_string(), msg.to_string()))
            },
        )
        .await
        .expect("download should succeed");
        assert!(ok.path.ends_with("/downloads/mod.zip"));
        assert_eq!(ok.size, 1);
        assert_eq!(ok.created_at, 2);
        assert!(logs.borrow().is_empty());

        let logs_err = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
        let logs_err_out = logs_err.clone();
        let err = download_mod_archive_flow(
            "mod.zip",
            "https://example.com/mod.zip",
            None,
            || Ok(PathBuf::from("/downloads")),
            |_, _, _| async { Err("download failed".to_string()) },
            |level, msg| {
                logs_err_out
                    .borrow_mut()
                    .push((level.to_string(), msg.to_string()))
            },
        )
        .await;
        let err = match err {
            Ok(_) => panic!("download should fail"),
            Err(e) => e,
        };
        assert_eq!(err, "download failed");
        assert_eq!(logs_err.borrow().len(), 1);
        assert_eq!(logs_err.borrow()[0].0, "ERROR");
    });
}

#[test]
fn download_mod_archive_flow_propagates_download_dir_error_and_forwards_id() {
    tauri::async_runtime::block_on(async {
        let err = download_mod_archive_flow(
            "mod.zip",
            "https://example.com/mod.zip",
            None,
            || Err("downloads missing".to_string()),
            |_, _, _| async {
                Ok(DownloadResult {
                    path: String::new(),
                    size: 0,
                    created_at: 0,
                })
            },
            |_, _| {},
        )
        .await;
        let err = match err {
            Ok(_) => panic!("downloads-dir error should bubble"),
            Err(err) => err,
        };
        assert_eq!(err, "downloads missing");

        let seen = Rc::new(RefCell::new(None::<(String, PathBuf, Option<String>)>));
        let seen_out = seen.clone();
        let _ = download_mod_archive_flow(
            "mod.zip",
            "https://example.com/mod.zip",
            Some("download-1"),
            || Ok(PathBuf::from("/downloads")),
            move |url, path, id| {
                *seen_out.borrow_mut() = Some((url, path.clone(), id.clone()));
                async move {
                    Ok(DownloadResult {
                        path: path.to_string_lossy().to_string(),
                        size: 1,
                        created_at: 2,
                    })
                }
            },
            |_, _| {},
        )
        .await
        .expect("download should succeed");

        let seen = seen.borrow().clone().expect("download callback should run");
        assert_eq!(seen.0, "https://example.com/mod.zip");
        assert_eq!(seen.1, PathBuf::from("/downloads/mod.zip"));
        assert_eq!(seen.2.as_deref(), Some("download-1"));
    });
}

#[test]
fn update_mod_id_rejects_missing_or_invalid_files() {
    let dir = temp_test_dir("update_err");
    let path = dir.join("mod_info.json");
    assert!(update_mod_id_in_json_file(&path, "1").is_err());

    fs::write(&path, "not json").unwrap();
    assert!(update_mod_id_in_json_file(&path, "1").is_err());

    fs::write(&path, "[]").unwrap();
    assert!(update_mod_id_in_json_file(&path, "1").is_err());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn update_mod_id_propagates_read_errors_for_directory_targets() {
    let dir = temp_test_dir("update_read_err");
    let path = dir.join("mod_info.json");
    fs::create_dir_all(&path).unwrap();

    let err = update_mod_id_in_json_file(&path, "new-id").expect_err("directory read should fail");
    assert!(err.contains("Failed to read mod_info.json"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn update_mod_id_sets_id_field() {
    let dir = temp_test_dir("update_ok");
    let path = dir.join("mod_info.json");
    fs::write(&path, r#"{"modId":"a"}"#).unwrap();

    update_mod_id_in_json_file(&path, "new-id").unwrap();
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value.get("id").and_then(|v| v.as_str()), Some("new-id"));

    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn update_mod_id_propagates_write_errors_for_read_only_files() {
    let dir = temp_test_dir("update_write_err");
    let path = dir.join("mod_info.json");
    fs::write(&path, r#"{"modId":"a"}"#).unwrap();

    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&path, perms).unwrap();

    let err = update_mod_id_in_json_file(&path, "new-id").expect_err("read-only write should fail");
    assert!(err.contains("Failed to write updated mod_info.json"));

    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&path, perms).unwrap();
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_mod_info_creates_and_updates_expected_fields() {
    let dir = temp_test_dir("ensure_ok");
    let path = dir.join("mod_info.json");

    let input = EnsureModInfoInput {
        mod_id: "m1".to_string(),
        file_id: "f1".to_string(),
        version: "1.0".to_string(),
        install_source: "archive.zip".to_string(),
    };
    ensure_mod_info_file(&path, &input).unwrap();

    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value.get("modId").and_then(|v| v.as_str()), Some("m1"));
    assert_eq!(value.get("fileId").and_then(|v| v.as_str()), Some("f1"));
    assert_eq!(value.get("version").and_then(|v| v.as_str()), Some("1.0"));
    assert_eq!(
        value.get("installSource").and_then(|v| v.as_str()),
        Some("archive.zip")
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_mod_info_preserves_existing_values_when_inputs_empty() {
    let dir = temp_test_dir("ensure_preserve");
    let path = dir.join("mod_info.json");
    fs::write(
        &path,
        r#"{"modId":"old","fileId":"oldf","version":"oldv","other":"keep"}"#,
    )
    .unwrap();

    let input = EnsureModInfoInput {
        mod_id: "".to_string(),
        file_id: "".to_string(),
        version: "".to_string(),
        install_source: "src.zip".to_string(),
    };
    ensure_mod_info_file(&path, &input).unwrap();

    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value.get("modId").and_then(|v| v.as_str()), Some("old"));
    assert_eq!(value.get("fileId").and_then(|v| v.as_str()), Some("oldf"));
    assert_eq!(value.get("version").and_then(|v| v.as_str()), Some("oldv"));
    assert_eq!(value.get("other").and_then(|v| v.as_str()), Some("keep"));
    assert_eq!(
        value.get("installSource").and_then(|v| v.as_str()),
        Some("src.zip")
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_mod_info_rejects_non_object_json() {
    let dir = temp_test_dir("ensure_non_object");
    let path = dir.join("mod_info.json");
    fs::write(&path, "[]").unwrap();

    let input = EnsureModInfoInput {
        mod_id: "x".to_string(),
        file_id: "y".to_string(),
        version: "z".to_string(),
        install_source: "src.zip".to_string(),
    };
    assert!(ensure_mod_info_file(&path, &input).is_err());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_mod_info_propagates_parse_and_write_errors() {
    let dir = temp_test_dir("ensure_errs");
    let invalid = dir.join("invalid.json");
    fs::write(&invalid, "not json").unwrap();

    let input = EnsureModInfoInput {
        mod_id: "x".to_string(),
        file_id: "y".to_string(),
        version: "z".to_string(),
        install_source: "src.zip".to_string(),
    };
    assert!(ensure_mod_info_file(&invalid, &input).is_err());

    let missing_parent = dir.join("missing").join("mod_info.json");
    assert!(ensure_mod_info_file(&missing_parent, &input).is_err());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ensure_mod_info_propagates_read_errors_for_directory_targets() {
    let dir = temp_test_dir("ensure_read_err");
    let path = dir.join("mod_info.json");
    fs::create_dir_all(&path).unwrap();

    let input = EnsureModInfoInput {
        mod_id: "x".to_string(),
        file_id: "y".to_string(),
        version: "z".to_string(),
        install_source: "src.zip".to_string(),
    };
    assert!(ensure_mod_info_file(&path, &input).is_err());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_mod_info_file_reads_valid_json_and_ignores_missing() {
    let dir = temp_test_dir("read_mod_info");
    assert!(read_mod_info_file(&dir).is_none());

    fs::write(
        dir.join("mod_info.json"),
        r#"{"modId":"abc","fileId":"123","version":"1.0"}"#,
    )
    .unwrap();
    let info = read_mod_info_file(&dir).expect("expected parsed mod info");
    assert_eq!(info.mod_id.as_deref(), Some("abc"));
    assert_eq!(info.file_id.as_deref(), Some("123"));
    assert_eq!(info.version.as_deref(), Some("1.0"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_mod_info_file_returns_none_for_directory_targets() {
    let dir = temp_test_dir("read_mod_info_dir");
    let info_dir = dir.join("mod_info.json");
    fs::create_dir_all(&info_dir).unwrap();

    assert!(read_mod_info_file(&dir).is_none());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_mod_info_file_returns_none_for_invalid_json() {
    let dir = temp_test_dir("read_mod_info_invalid");
    fs::write(dir.join("mod_info.json"), "not json").unwrap();

    assert!(read_mod_info_file(&dir).is_none());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_mod_info_file_accepts_id_alias_and_install_source() {
    let dir = temp_test_dir("read_mod_info_alias");
    fs::write(
        dir.join("mod_info.json"),
        r#"{"id":"fallback","fileId":"77","installSource":"mod.zip"}"#,
    )
    .unwrap();

    let info = read_mod_info_file(&dir).expect("expected parsed mod info");
    assert_eq!(info.mod_id.as_deref(), Some("fallback"));
    assert_eq!(info.file_id.as_deref(), Some("77"));
    assert_eq!(info.install_source.as_deref(), Some("mod.zip"));

    fs::remove_dir_all(dir).unwrap();
}
