use super::{
    delete_mod_app_flow_with, delete_mod_command_entry_with, delete_mod_runtime_with,
    download_mod_archive_app_flow_with, download_mod_archive_command_entry_with,
    download_mod_archive_runtime_with, ensure_mod_info_command_entry_with,
    ensure_mod_info_runtime_with, rename_mod_folder_app_flow_with,
    rename_mod_folder_command_entry_with, rename_mod_folder_runtime_with,
    reorder_mods_command_entry_with, reorder_mods_command_with, reorder_mods_runtime_with,
    update_mod_id_in_json_command_entry_with, update_mod_id_in_json_runtime_with,
    update_mod_name_in_xml_command_entry_with, update_mod_name_in_xml_runtime_with,
};
use crate::adapters::tauri::mods::{
    ensure_mod_info, reorder_mods, update_mod_id_in_json, update_mod_name_in_xml,
};
use crate::models::{DownloadResult, ModRenderData};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn fake_game_path() -> Option<PathBuf> {
    Some(PathBuf::from("/game"))
}

fn missing_game_path() -> Option<PathBuf> {
    None
}

fn settings_path(root: &Path) -> PathBuf {
    root.join("Binaries/SETTINGS/MODS_ENABLED.mxml")
}

fn library_dir_ok() -> Result<PathBuf, String> {
    Ok(PathBuf::from("/library"))
}

fn save_file_ok(_path: String, _content: String) -> Result<(), String> {
    Ok(())
}

fn empty_render_ok() -> Result<Vec<ModRenderData>, String> {
    Ok(Vec::new())
}

fn noop_log(_level: &str, _message: &str) {}

fn rename_dir_ok(_old: &Path, _new: &Path) -> Result<(), String> {
    Ok(())
}

fn rename_settings_ok(
    _settings: &Path,
    _old_name: &str,
    _new_name: &str,
) -> Result<String, String> {
    Ok("<xml/>".to_string())
}

fn mod_folder(root: &Path, name: &str) -> PathBuf {
    root.join("GAMEDATA").join("MODS").join(name)
}

fn remove_mod_folder_ok(_mod_path: &Path, _mod_name: &str) -> Result<bool, String> {
    Ok(true)
}

fn delete_settings_ok(_settings: &Path, _mod_name: &str) -> Result<(), String> {
    Ok(())
}

#[test]
fn runtime_reorder_wrapper_propagates_errors() {
    let err = reorder_mods_command_with(vec!["A".to_string()], |_names| {
        Err("reorder-failed".to_string())
    })
    .expect_err("wrapper should propagate error");
    assert_eq!(err, "reorder-failed");
}

#[test]
fn runtime_reorder_wrapper_forwards_inputs() {
    let out = reorder_mods_runtime_with(vec!["B".to_string(), "A".to_string()], |names| {
        assert_eq!(names, vec!["B".to_string(), "A".to_string()]);
        Ok("ok".to_string())
    })
    .expect("reorder should forward");
    assert_eq!(out, "ok");
}

#[test]
fn runtime_rename_wrapper_logs_and_forwards() {
    let mut logs = Vec::new();
    let out = rename_mod_folder_runtime_with(
        "Old".to_string(),
        "New".to_string(),
        |level, msg| logs.push(format!("{level}:{msg}")),
        |old_name, new_name| {
            assert_eq!(old_name, "Old");
            assert_eq!(new_name, "New");
            Ok(Vec::new())
        },
    )
    .expect("rename should succeed");
    assert!(out.is_empty());
    assert!(logs.iter().any(|m| m.contains("Requesting rename")));
}

#[test]
fn runtime_delete_wrapper_logs_and_propagates_error() {
    let mut logs = Vec::new();
    let err = delete_mod_runtime_with(
        "MyMod".to_string(),
        |level, msg| logs.push(format!("{level}:{msg}")),
        |_name| Err("delete-failed".to_string()),
    )
    .err()
    .expect("delete should fail");
    assert_eq!(err, "delete-failed");
    assert!(logs.iter().any(|m| m.contains("Requesting deletion")));
}

#[test]
fn runtime_update_wrappers_forward_inputs() {
    let out = update_mod_name_in_xml_runtime_with(
        "Old".to_string(),
        "New".to_string(),
        |old_name, new_name| {
            assert_eq!(old_name, "Old");
            assert_eq!(new_name, "New");
            Ok("ok".to_string())
        },
    )
    .expect("update name should forward");
    assert_eq!(out, "ok");

    update_mod_id_in_json_runtime_with("Folder".to_string(), "123".to_string(), |folder, id| {
        assert_eq!(folder, "Folder");
        assert_eq!(id, "123");
        Ok(())
    })
    .expect("update id should forward");
}

#[test]
fn runtime_ensure_wrapper_forward_inputs() {
    ensure_mod_info_runtime_with(
        "Folder".to_string(),
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "archive.zip".to_string(),
        |folder, mod_id, file_id, version, source| {
            assert_eq!(folder, "Folder");
            assert_eq!(mod_id, "1");
            assert_eq!(file_id, "2");
            assert_eq!(version, "3");
            assert_eq!(source, "archive.zip");
            Ok(())
        },
    )
    .expect("ensure should forward");
}

#[test]
fn runtime_download_wrapper_forwards_and_propagates() {
    let ok = tauri::async_runtime::block_on(download_mod_archive_runtime_with(
        "https://example.invalid/mod.zip".to_string(),
        "mod.zip".to_string(),
        Some("id-1".to_string()),
        |file_name, url, download_id| async move {
            assert_eq!(url, "https://example.invalid/mod.zip");
            assert_eq!(file_name, "mod.zip");
            assert_eq!(download_id.as_deref(), Some("id-1"));
            Ok(DownloadResult {
                path: "/tmp/mod.zip".to_string(),
                size: 1,
                created_at: 2,
            })
        },
    ))
    .expect("download helper should forward success");
    assert_eq!(ok.path, "/tmp/mod.zip");

    let err = tauri::async_runtime::block_on(download_mod_archive_runtime_with(
        "u".to_string(),
        "f".to_string(),
        None,
        |_url, _file_name, _download_id| async move { Err("download-failed".to_string()) },
    ))
    .err()
    .expect("download helper should propagate error");
    assert_eq!(err, "download-failed");
}

#[test]
fn rename_mod_folder_app_flow_with_forwards_and_returns_rendered_mods() {
    let render = vec![ModRenderData {
        priority: 1000,
        folder_name: "B".to_string(),
        enabled: true,
        local_info: None,
    }];
    let calls = Rc::new(RefCell::new(Vec::<String>::new()));
    let out = rename_mod_folder_app_flow_with(
        "Old".to_string(),
        "New".to_string(),
        || Some(PathBuf::from("/game")),
        |_root, name| {
            PathBuf::from("/game")
                .join("GAMEDATA")
                .join("MODS")
                .join(name)
        },
        |_old_exists, _new_exists| Ok(()),
        |_p| None,
        || Ok(PathBuf::from("/lib")),
        |_lib, _source_zip, _old_name, _new_name| {
            Ok(crate::mods::command_ops::LibraryRenameSync::SourceMissing)
        },
        |_old, _new| Ok(()),
        |_root| PathBuf::from("/game/Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |_old, _new| Ok("<xml/>".to_string()),
        |_path, _content| Ok(()),
        || Ok(render.clone()),
        |_level, _message| calls.borrow_mut().push("log".to_string()),
    )
    .expect("rename app flow should succeed");
    assert_eq!(out.len(), 1);
}

#[test]
fn delete_mod_app_flow_with_forwards_and_logs() {
    let logs = Rc::new(RefCell::new(Vec::<String>::new()));
    let out = delete_mod_app_flow_with(
        "DeleteMe".to_string(),
        || Some(PathBuf::from("/game")),
        |_root| PathBuf::from("/game/Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |_root, name| PathBuf::from("/game/GAMEDATA/MODS").join(name),
        |_path, _name| Ok(true),
        |_settings, _name| Ok(()),
        || Ok(Vec::new()),
        |level, message| logs.borrow_mut().push(format!("{level}:{message}")),
    )
    .expect("delete app flow should succeed");
    assert!(out.is_empty());
    assert!(logs.borrow().iter().any(|m| m.contains("Deleted folder")));
}

#[test]
fn delete_mod_app_flow_with_propagates_missing_game_path() {
    let err = delete_mod_app_flow_with(
        "DeleteMe".to_string(),
        || None,
        |_root| PathBuf::from("/game/Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |_root, name| PathBuf::from("/game/GAMEDATA/MODS").join(name),
        |_path, _name| Ok(false),
        |_settings, _name| Ok(()),
        || Ok(Vec::new()),
        |_level, _message| {},
    )
    .err()
    .expect("missing game path should error");
    assert_eq!(err, "Could not find game installation path.");
}

#[test]
fn download_mod_archive_app_flow_with_forwards_and_logs_errors() {
    let logs = Rc::new(RefCell::new(Vec::<String>::new()));
    let ok = tauri::async_runtime::block_on(download_mod_archive_app_flow_with(
        "file.zip".to_string(),
        "https://example.invalid/file.zip".to_string(),
        Some("id-1".to_string()),
        || Ok(PathBuf::from("/downloads")),
        |url, path, id| async move {
            assert_eq!(url, "https://example.invalid/file.zip");
            assert_eq!(path, Path::new("/downloads/file.zip"));
            assert_eq!(id.as_deref(), Some("id-1"));
            Ok(DownloadResult {
                path: "/downloads/file.zip".to_string(),
                size: 5,
                created_at: 6,
            })
        },
        |_level, _message| {},
    ))
    .expect("download app flow should succeed");
    assert_eq!(ok.path, "/downloads/file.zip");

    let err = tauri::async_runtime::block_on(download_mod_archive_app_flow_with(
        "file.zip".to_string(),
        "https://example.invalid/file.zip".to_string(),
        None,
        || Ok(PathBuf::from("/downloads")),
        |_url, _path, _id| async move { Err("network-failed".to_string()) },
        |level, message| logs.borrow_mut().push(format!("{level}:{message}")),
    ))
    .err()
    .expect("download flow should propagate error");
    assert_eq!(err, "network-failed");
    assert!(logs
        .borrow()
        .iter()
        .any(|m| m.contains("ERROR:network-failed")));
}

#[test]
fn download_mod_archive_command_entry_with_logs_and_forwards() {
    let logs = Rc::new(RefCell::new(Vec::<String>::new()));
    let ok = tauri::async_runtime::block_on(download_mod_archive_command_entry_with(
        "file.zip".to_string(),
        "https://example.invalid/file.zip".to_string(),
        Some("id-9".to_string()),
        |level, message| logs.borrow_mut().push(format!("{level}:{message}")),
        |file_name, url, download_id| async move {
            assert_eq!(file_name, "file.zip");
            assert_eq!(url, "https://example.invalid/file.zip");
            assert_eq!(download_id.as_deref(), Some("id-9"));
            Ok(DownloadResult {
                path: "/downloads/file.zip".to_string(),
                size: 77,
                created_at: 88,
            })
        },
    ))
    .expect("command entry should succeed");
    assert_eq!(ok.path, "/downloads/file.zip");
    assert!(logs
        .borrow()
        .iter()
        .any(|m| m.contains("INFO:Starting download")));

    let err = tauri::async_runtime::block_on(download_mod_archive_command_entry_with(
        "file.zip".to_string(),
        "https://example.invalid/file.zip".to_string(),
        None,
        |level, message| logs.borrow_mut().push(format!("{level}:{message}")),
        |_file_name, _url, _download_id| async move { Err("download-step-failed".to_string()) },
    ))
    .err()
    .expect("command entry should propagate error");
    assert_eq!(err, "download-step-failed");
}

#[test]
fn rename_mod_folder_command_entry_with_forwards_to_app_flow() {
    let err = rename_mod_folder_command_entry_with(
        "OLD_MOD".to_string(),
        "NEW_MOD".to_string(),
        missing_game_path,
        settings_path,
        rename_settings_ok,
        library_dir_ok,
        save_file_ok,
        empty_render_ok,
        noop_log,
        rename_dir_ok,
    )
    .err()
    .expect("rename command entry should propagate missing game path");
    assert_eq!(err, "Could not find game path.");
}

#[test]
fn rename_mod_folder_command_entry_with_forwards_success_path() {
    let root = std::env::temp_dir().join("pulsarmm_mod_commands_runtime_rename_entry");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("GAMEDATA/MODS/OLD_MOD")).unwrap();
    std::fs::create_dir_all(root.join("Binaries/SETTINGS")).unwrap();
    std::fs::write(root.join("Binaries/SETTINGS/MODS_ENABLED.mxml"), "<xml/>").unwrap();

    let out = rename_mod_folder_command_entry_with(
        "OLD_MOD".to_string(),
        "NEW_MOD".to_string(),
        || Some(root.clone()),
        settings_path,
        rename_settings_ok,
        library_dir_ok,
        save_file_ok,
        empty_render_ok,
        noop_log,
        rename_dir_ok,
    )
    .expect("rename command entry should succeed");

    assert!(out.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn delete_mod_command_entry_with_forwards_to_app_flow() {
    let err = delete_mod_command_entry_with(
        "DELETE_ME".to_string(),
        missing_game_path,
        settings_path,
        mod_folder,
        remove_mod_folder_ok,
        delete_settings_ok,
        empty_render_ok,
        noop_log,
    )
    .err()
    .expect("delete command entry should propagate missing game path");
    assert_eq!(err, "Could not find game installation path.");
}

#[test]
fn delete_mod_command_entry_with_forwards_success_path() {
    let out = delete_mod_command_entry_with(
        "DELETE_ME".to_string(),
        fake_game_path,
        settings_path,
        mod_folder,
        remove_mod_folder_ok,
        delete_settings_ok,
        empty_render_ok,
        noop_log,
    )
    .expect("delete command entry should succeed");
    assert!(out.is_empty());
}

#[test]
fn settings_command_entries_cover_reorder_update_and_ensure_paths() {
    let reorder_err = reorder_mods_command_entry_with(
        vec!["A".to_string()],
        || None,
        |_root| PathBuf::from("/game/Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |_settings, _ordered| Ok("ok".to_string()),
    )
    .err()
    .expect("reorder entry should require game path");
    assert_eq!(reorder_err, "Could not find game installation path.");

    let rename_err = update_mod_name_in_xml_command_entry_with(
        "OLD".to_string(),
        "NEW".to_string(),
        || None,
        |_root| PathBuf::from("/game/Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |_settings, _old_name, _new_name| Ok("<xml/>".to_string()),
    )
    .err()
    .expect("rename entry should require game path");
    assert_eq!(rename_err, "Could not find game installation path.");

    let id_err = update_mod_id_in_json_command_entry_with(
        "FOLDER".to_string(),
        "123".to_string(),
        || None,
        |_root, _folder, _id| Ok(()),
    )
    .err()
    .expect("update id entry should require game path");
    assert_eq!(id_err, "Could not find game installation path.");

    let ensure_err = ensure_mod_info_command_entry_with(
        "FOLDER".to_string(),
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "archive.zip".to_string(),
        || None,
        |_root, _folder, _input| Ok(()),
    )
    .err()
    .expect("ensure entry should require game path");
    assert_eq!(ensure_err, "Could not find game installation path.");
}

#[test]
fn settings_command_entries_forward_success_paths() {
    let reorder = reorder_mods_command_entry_with(
        vec!["B".to_string(), "A".to_string()],
        || Some(PathBuf::from("/game")),
        |root| root.join("Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |settings_path, ordered| {
            assert_eq!(
                settings_path,
                Path::new("/game/Binaries/SETTINGS/MODS_ENABLED.mxml")
            );
            assert_eq!(ordered, ["B".to_string(), "A".to_string()]);
            Ok("reordered".to_string())
        },
    )
    .expect("reorder entry should succeed");
    assert_eq!(reorder, "reordered");

    let renamed = update_mod_name_in_xml_command_entry_with(
        "OLD".to_string(),
        "NEW".to_string(),
        || Some(PathBuf::from("/game")),
        |root| root.join("Binaries/SETTINGS/MODS_ENABLED.mxml"),
        |settings_path, old_name, new_name| {
            assert_eq!(
                settings_path,
                Path::new("/game/Binaries/SETTINGS/MODS_ENABLED.mxml")
            );
            assert_eq!(old_name, "OLD");
            assert_eq!(new_name, "NEW");
            Ok("<xml/>".to_string())
        },
    )
    .expect("rename entry should succeed");
    assert_eq!(renamed, "<xml/>");

    update_mod_id_in_json_command_entry_with(
        "FOLDER".to_string(),
        "123".to_string(),
        || Some(PathBuf::from("/game")),
        |root, folder, id| {
            assert_eq!(root, Path::new("/game"));
            assert_eq!(folder, "FOLDER");
            assert_eq!(id, "123");
            Ok(())
        },
    )
    .expect("update id entry should succeed");

    ensure_mod_info_command_entry_with(
        "FOLDER".to_string(),
        "mod-1".to_string(),
        "file-1".to_string(),
        "1.0.0".to_string(),
        "archive.zip".to_string(),
        || Some(PathBuf::from("/game")),
        |root, folder, input| {
            assert_eq!(root, Path::new("/game"));
            assert_eq!(folder, "FOLDER");
            assert_eq!(input.mod_id, "mod-1");
            assert_eq!(input.file_id, "file-1");
            assert_eq!(input.version, "1.0.0");
            assert_eq!(input.install_source, "archive.zip");
            Ok(())
        },
    )
    .expect("ensure entry should succeed");
}

#[test]
fn public_settings_commands_fail_cleanly_when_game_path_missing() {
    let reorder_err =
        reorder_mods(vec!["A".to_string()]).expect_err("reorder should fail without game path");
    assert!(
        !reorder_err.is_empty(),
        "reorder error should provide user-facing context"
    );

    let rename_err = update_mod_name_in_xml("Old".to_string(), "New".to_string())
        .expect_err("update name should fail without game path");
    assert!(
        !rename_err.is_empty(),
        "update name error should provide user-facing context"
    );

    let update_id_err = update_mod_id_in_json("Folder".to_string(), "42".to_string())
        .expect_err("update id should fail without game path");
    assert!(
        !update_id_err.is_empty(),
        "update id error should provide user-facing context"
    );

    let ensure_err = ensure_mod_info(
        "Folder".to_string(),
        "mod-id".to_string(),
        "file-id".to_string(),
        "1.0.0".to_string(),
        "manual".to_string(),
    )
    .expect_err("ensure mod info should fail without game path");
    assert!(
        !ensure_err.is_empty(),
        "ensure mod info error should provide user-facing context"
    );
}
