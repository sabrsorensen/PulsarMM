use super::{
    build_ensure_mod_info_input, delete_mod_command_with, delete_mod_with_deps,
    delete_request_log_message, download_mod_archive_command_with, download_mod_archive_with,
    download_request_log_message, ensure_mod_info_command_with, ensure_mod_info_with,
    rename_mod_folder_command_with, rename_mod_folder_with_deps, rename_request_log_message,
    reorder_mods_command_with, reorder_mods_with, update_mod_id_in_json_command_with,
    update_mod_id_in_json_with, update_mod_name_in_xml_command_with, update_mod_name_in_xml_with,
};
use crate::models::{DownloadResult, ModInfo, ModRenderData};
use crate::mods::command_download_ops::ensure_success_status;
use crate::mods::command_download_ops::progress_payload;
use crate::mods::command_logic::{
    build_download_result, map_update_mod_id_error, mod_info_path_for,
};
use crate::mods::command_ops::{
    library_rename_paths, mod_folder_path, mods_root_from_game_path, validate_rename_paths,
};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

#[test]
fn mod_commands_wrapper_uses_expected_logic_contracts() {
    let info_path = mod_info_path_for(Path::new("/game"), "MyMod");
    assert!(info_path.ends_with("GAMEDATA/MODS/MyMod/mod_info.json"));

    let (old_path, new_path) = library_rename_paths(Path::new("/lib"), "source.zip", "Old", "New");
    assert!(old_path.ends_with("source.zip_unpacked/Old"));
    assert!(new_path.ends_with("source.zip_unpacked/New"));

    ensure_success_status(reqwest::StatusCode::OK).expect("expected success status");
    let payload = progress_payload("id", 50);
    assert_eq!(payload.step, "Downloading: 50%");
    assert_eq!(payload.progress, Some(50));
    assert_eq!(
        mods_root_from_game_path(Path::new("/tmp/NMS")),
        Path::new("/tmp/NMS/GAMEDATA/MODS")
    );
    assert_eq!(
        mod_folder_path(Path::new("/tmp/NMS"), "MyMod"),
        Path::new("/tmp/NMS/GAMEDATA/MODS/MyMod")
    );
    validate_rename_paths(true, false).expect("expected valid rename paths");
    assert_eq!(
        map_update_mod_id_error("MyMod", "file not found for path".to_string()),
        "mod_info.json not found for mod 'MyMod'."
    );

    let out = build_download_result(Path::new("/tmp/a.zip"), 1, 2);
    assert_eq!(out.path, "/tmp/a.zip");
    assert_eq!(out.size, 1);
    assert_eq!(out.created_at, 2);
}

#[test]
fn mod_commands_wrapper_builds_expected_log_messages() {
    assert_eq!(
        rename_request_log_message("Old", "New"),
        "Requesting rename: 'Old' -> 'New'"
    );
    assert_eq!(
        delete_request_log_message("MyMod"),
        "Requesting deletion of mod: MyMod"
    );
    assert_eq!(
        download_request_log_message("a.zip"),
        "Starting download request for: a.zip"
    );
}

#[test]
fn build_ensure_mod_info_input_sets_fields() {
    let input = build_ensure_mod_info_input(
        "1".to_string(),
        "2".to_string(),
        "3.0".to_string(),
        "archive.zip".to_string(),
    );
    assert_eq!(input.mod_id, "1");
    assert_eq!(input.file_id, "2");
    assert_eq!(input.version, "3.0");
    assert_eq!(input.install_source, "archive.zip");
}

#[test]
fn reorder_mods_with_forwards_arguments() {
    let ordered = vec!["B".to_string(), "A".to_string()];
    let out = reorder_mods_with(
        &ordered,
        || Some("/game".into()),
        |path| path.join("settings.xml"),
        |settings_file, names| {
            assert_eq!(settings_file, Path::new("/game/settings.xml"));
            assert_eq!(names, &["B".to_string(), "A".to_string()]);
            Ok("ok".to_string())
        },
    )
    .expect("expected forwarded reorder");
    assert_eq!(out, "ok");

    let err = reorder_mods_with(
        &ordered,
        || None,
        |path| path.join("settings.xml"),
        |_settings_file, _names| Ok("ok".to_string()),
    )
    .err()
    .expect("missing game path should error");
    assert_eq!(err, "Could not find game installation path.");
}

#[test]
fn mod_command_wrappers_forward_and_log() {
    let mut logs = Vec::<String>::new();
    let renamed = rename_mod_folder_command_with(
        "Old".to_string(),
        "New".to_string(),
        |level, msg| logs.push(format!("{level}:{msg}")),
        |old_name, new_name| {
            assert_eq!(old_name, "Old");
            assert_eq!(new_name, "New");
            Ok(Vec::new())
        },
    )
    .expect("rename wrapper");
    assert!(renamed.is_empty());
    assert!(logs
        .iter()
        .any(|line| line.contains("Requesting rename: 'Old' -> 'New'")));

    let deleted = delete_mod_command_with(
        "MyMod".to_string(),
        |level, msg| logs.push(format!("{level}:{msg}")),
        |mod_name| {
            assert_eq!(mod_name, "MyMod");
            Ok(Vec::new())
        },
    )
    .expect("delete wrapper");
    assert!(deleted.is_empty());
    assert!(logs
        .iter()
        .any(|line| line.contains("Requesting deletion of mod: MyMod")));

    let rename_err = rename_mod_folder_command_with(
        "Old".to_string(),
        "New".to_string(),
        |_level, _msg| {},
        |_old_name, _new_name| Err("rename-failed".to_string()),
    )
    .err()
    .expect("rename wrapper error path");
    assert_eq!(rename_err, "rename-failed");

    let delete_err = delete_mod_command_with(
        "MyMod".to_string(),
        |_level, _msg| {},
        |_mod_name| Err("delete-failed".to_string()),
    )
    .err()
    .expect("delete wrapper error path");
    assert_eq!(delete_err, "delete-failed");
}

#[test]
fn mod_command_update_and_ensure_wrappers_forward_inputs() {
    let out = update_mod_name_in_xml_with("Old", "New", |old_name, new_name| {
        assert_eq!(old_name, "Old");
        assert_eq!(new_name, "New");
        Ok("ok".to_string())
    })
    .expect("update name wrapper");
    assert_eq!(out, "ok");

    update_mod_id_in_json_with("Folder", "123", |folder, id| {
        assert_eq!(folder, "Folder");
        assert_eq!(id, "123");
        Ok(())
    })
    .expect("update id wrapper");

    let input = build_ensure_mod_info_input(
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "archive.zip".to_string(),
    );
    ensure_mod_info_with("Folder", &input, |folder, forwarded| {
        assert_eq!(folder, "Folder");
        assert_eq!(forwarded.install_source, "archive.zip");
        Ok(())
    })
    .expect("ensure wrapper");

    let err = update_mod_name_in_xml_with("Old", "New", |_old_name, _new_name| {
        Err("update-name-failed".to_string())
    })
    .err()
    .expect("update name error should bubble");
    assert_eq!(err, "update-name-failed");

    let err = update_mod_id_in_json_with("Folder", "123", |_folder, _id| {
        Err("update-id-failed".to_string())
    })
    .err()
    .expect("update id error should bubble");
    assert_eq!(err, "update-id-failed");

    let err = ensure_mod_info_with("Folder", &input, |_folder, _forwarded| {
        Err("ensure-failed".to_string())
    })
    .err()
    .expect("ensure error should bubble");
    assert_eq!(err, "ensure-failed");
}

#[test]
fn mod_command_entry_wrappers_forward_and_propagate() {
    let reordered = reorder_mods_command_with(vec!["B".to_string(), "A".to_string()], |names| {
        assert_eq!(names, vec!["B".to_string(), "A".to_string()]);
        Ok("ok".to_string())
    })
    .expect("reorder command wrapper");
    assert_eq!(reordered, "ok");

    let renamed = update_mod_name_in_xml_command_with(
        "Old".to_string(),
        "New".to_string(),
        |old, new_name| {
            assert_eq!(old, "Old");
            assert_eq!(new_name, "New");
            Ok("done".to_string())
        },
    )
    .expect("update name command wrapper");
    assert_eq!(renamed, "done");

    update_mod_id_in_json_command_with("Folder".to_string(), "123".to_string(), |folder, id| {
        assert_eq!(folder, "Folder");
        assert_eq!(id, "123");
        Ok(())
    })
    .expect("update id command wrapper");

    ensure_mod_info_command_with(
        "Folder".to_string(),
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "archive.zip".to_string(),
        |folder, input| {
            assert_eq!(folder, "Folder");
            assert_eq!(input.mod_id, "1");
            assert_eq!(input.file_id, "2");
            assert_eq!(input.version, "3");
            assert_eq!(input.install_source, "archive.zip");
            Ok(())
        },
    )
    .expect("ensure command wrapper");

    tauri::async_runtime::block_on(async {
        let out = download_mod_archive_command_with(
            "a.zip".to_string(),
            "https://example.invalid/a.zip".to_string(),
            Some("id-1".to_string()),
            |name, url, id| async move {
                assert_eq!(name, "a.zip");
                assert_eq!(url, "https://example.invalid/a.zip");
                assert_eq!(id.as_deref(), Some("id-1"));
                Ok(DownloadResult {
                    path: "/tmp/a.zip".to_string(),
                    size: 1,
                    created_at: 2,
                })
            },
        )
        .await
        .expect("download command wrapper");
        assert_eq!(out.path, "/tmp/a.zip");
    });

    let err = update_mod_name_in_xml_command_with(
        "Old".to_string(),
        "New".to_string(),
        |_old, _new_name| Err("update-name-failed".to_string()),
    )
    .err()
    .expect("update name error should bubble");
    assert_eq!(err, "update-name-failed");
}

#[test]
fn download_mod_archive_with_logs_and_forwards_arguments() {
    tauri::async_runtime::block_on(async {
        let mut logs = Vec::<String>::new();
        let mut seen = None::<(String, String, Option<String>)>;
        let result = download_mod_archive_with(
            "a.zip".to_string(),
            "https://example.invalid/a.zip".to_string(),
            Some("id-1".to_string()),
            |level, message| logs.push(format!("{level}:{message}")),
            |file_name, download_url, download_id| {
                seen = Some((file_name, download_url, download_id));
                async move {
                    Ok(DownloadResult {
                        path: "/tmp/a.zip".to_string(),
                        size: 10,
                        created_at: 20,
                    })
                }
            },
        )
        .await
        .expect("download wrapper should return result");

        assert!(logs
            .iter()
            .any(|m| m.contains("Starting download request for: a.zip")));
        assert_eq!(
            seen,
            Some((
                "a.zip".to_string(),
                "https://example.invalid/a.zip".to_string(),
                Some("id-1".to_string())
            ))
        );
        assert_eq!(result.path, "/tmp/a.zip");
    });
}

#[test]
fn download_mod_archive_with_propagates_flow_errors() {
    tauri::async_runtime::block_on(async {
        let err = download_mod_archive_with(
            "a.zip".to_string(),
            "https://example.invalid/a.zip".to_string(),
            None,
            |_level, _message| {},
            |_file_name, _download_url, _download_id| async { Err("download-failed".to_string()) },
        )
        .await
        .err()
        .expect("flow error should bubble");
        assert_eq!(err, "download-failed");
    });
}

#[test]
fn rename_mod_folder_with_deps_handles_success_warning_and_error_paths() {
    let root = std::env::temp_dir().join("pulsarmm_mod_commands_rename_deps");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("GAMEDATA/MODS/OLD")).unwrap();
    fs::create_dir_all(root.join("Binaries/SETTINGS")).unwrap();
    fs::write(root.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"), "<Data/>").unwrap();
    fs::write(root.join("GAMEDATA/MODS/OLD/mod_info.json"), "{}").unwrap();

    let logs = Rc::new(RefCell::new(Vec::<String>::new()));
    let logs_ref = logs.clone();
    let saved = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
    let saved_ref = saved.clone();

    let out = rename_mod_folder_with_deps(
        "OLD".to_string(),
        "NEW".to_string(),
        || Some(root.clone()),
        |game, name| game.join("GAMEDATA/MODS").join(name),
        validate_rename_paths,
        |_path| {
            Some(ModInfo {
                mod_id: Some("m".to_string()),
                file_id: Some("f".to_string()),
                version: Some("1".to_string()),
                install_source: Some("archive.zip".to_string()),
            })
        },
        || Ok(std::env::temp_dir()),
        |_lib, _src, _old, _new| Err("sync-warning".to_string()),
        |old, new| fs::rename(old, new).map_err(|e| e.to_string()),
        |game| game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"),
        |_old, _new| Ok("<Data/>".to_string()),
        move |file_path, content| {
            saved_ref.borrow_mut().push((file_path, content));
            Ok(())
        },
        || Ok(Vec::<ModRenderData>::new()),
        move |level, msg| logs_ref.borrow_mut().push(format!("{level}:{msg}")),
    )
    .expect("rename should succeed");
    assert!(out.is_empty());
    assert!(root.join("GAMEDATA/MODS/NEW").exists());
    assert!(saved
        .borrow()
        .iter()
        .any(|(p, _)| p.ends_with("GCMODSETTINGS.MXML")));
    assert!(logs.borrow().iter().any(|m| m.contains("sync-warning")));

    let err = rename_mod_folder_with_deps(
        "MISSING".to_string(),
        "NEW2".to_string(),
        || Some(root.clone()),
        |game, name| game.join("GAMEDATA/MODS").join(name),
        validate_rename_paths,
        |_path| None,
        || Ok(std::env::temp_dir()),
        |_lib, _src, _old, _new| Ok(crate::mods::command_ops::LibraryRenameSync::SourceMissing),
        |_old, _new| Err("rename-failed".to_string()),
        |game| game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"),
        |_old, _new| Ok("<Data/>".to_string()),
        |_file_path, _content| Ok(()),
        || Ok(Vec::<ModRenderData>::new()),
        |_level, _msg| {},
    );
    let err = match err {
        Ok(_) => panic!("expected rename error"),
        Err(err) => err,
    };
    assert_eq!(err, "Original mod folder not found.");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn delete_mod_with_deps_covers_success_and_missing_game_path() {
    let root = std::env::temp_dir().join("pulsarmm_mod_commands_delete_deps");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let logs = Rc::new(RefCell::new(Vec::<String>::new()));
    let logs_ref = logs.clone();

    let out = delete_mod_with_deps(
        "MYMOD".to_string(),
        || Some(root.clone()),
        |game| game.join("settings.xml"),
        |game, name| game.join("mods").join(name),
        |_mod_path, _mod_name| Ok(true),
        |_settings, _mod_name| Ok(()),
        || Ok(Vec::<ModRenderData>::new()),
        move |level, msg| logs_ref.borrow_mut().push(format!("{level}:{msg}")),
    )
    .expect("delete should succeed");
    assert!(out.is_empty());
    assert!(logs.borrow().iter().any(|m| m.contains("Deleted folder")));

    let err = delete_mod_with_deps(
        "MYMOD".to_string(),
        || None,
        |game| game.join("settings.xml"),
        |game, name| game.join("mods").join(name),
        |_mod_path, _mod_name| Ok(true),
        |_settings, _mod_name| Ok(()),
        || Ok(Vec::<ModRenderData>::new()),
        |_level, _msg| {},
    );
    let err = match err {
        Ok(_) => panic!("expected missing game path error"),
        Err(err) => err,
    };
    assert_eq!(err, "Could not find game installation path.");

    let _ = fs::remove_dir_all(&root);
}
