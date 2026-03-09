use super::{
    check_for_untracked_mods_command_with, check_for_untracked_mods_with,
    delete_settings_file_command_with, delete_settings_file_with, detect_game_installation_with,
    has_uninstaller_in_parent, http_request_with, open_mods_folder_command_with,
    open_mods_folder_with, resize_window_with, run_legacy_migration_command_with,
    run_legacy_migration_with, save_file_with, take_pending_intent, write_to_log_with,
};
use crate::models::{GamePaths, HttpResponse};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn has_uninstaller_in_parent_detects_expected_file() {
    let root = temp_test_dir("uninstaller");
    let app_dir = root.join("app");
    fs::create_dir_all(&app_dir).expect("failed to create app dir");
    let exe_path = app_dir.join("Pulsar.exe");
    fs::write(&exe_path, "bin").expect("failed to create exe");

    assert!(!has_uninstaller_in_parent(&exe_path));
    fs::write(app_dir.join("Uninstall.exe"), "bin").expect("failed to create uninstaller");
    assert!(has_uninstaller_in_parent(&exe_path));

    fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn take_pending_intent_takes_once() {
    let pending = Mutex::new(Some("nxm://example".to_string()));
    assert_eq!(
        take_pending_intent(&pending).as_deref(),
        Some("nxm://example")
    );
    assert!(take_pending_intent(&pending).is_none());
}

#[test]
fn take_pending_intent_recovers_from_poisoned_mutex() {
    let pending = Mutex::new(Some("nxm://poisoned".to_string()));
    let _ = std::panic::catch_unwind(|| {
        let _guard = pending.lock().expect("lock before poison");
        panic!("intentional poison");
    });

    assert_eq!(
        take_pending_intent(&pending).as_deref(),
        Some("nxm://poisoned")
    );
    assert!(take_pending_intent(&pending).is_none());
}

#[test]
fn delete_settings_file_with_routes_to_correct_handler() {
    let game_path = Some(PathBuf::from("/game"));
    let out = delete_settings_file_with(
        game_path,
        |p| p.join("settings.xml"),
        |_settings| Ok("alertDeleteSuccess".to_string()),
        crate::app::ops::delete_settings_without_game_path,
    )
    .expect("expected success");
    assert_eq!(out, "alertDeleteSuccess");

    let out = delete_settings_file_with(
        None,
        crate::settings_paths::mod_settings_file,
        crate::app::ops::delete_settings_at_path,
        crate::app::ops::delete_settings_without_game_path,
    )
    .expect_err("expected missing path route");
    assert_eq!(out, "alertDeleteError");

    let out = delete_settings_file_with(
        Some(PathBuf::from("/game")),
        |p| p.join("settings.xml"),
        |_settings| Err("alertDeleteError".to_string()),
        crate::app::ops::delete_settings_without_game_path,
    )
    .err()
    .expect("expected game-path delete error");
    assert_eq!(out, "alertDeleteError");

    let out = delete_settings_file_with(
        None,
        |_p| PathBuf::from("/unused"),
        |_settings| Err("alertDeleteError".to_string()),
        || Ok("alertDeleteSuccess".to_string()),
    )
    .expect("expected no-game-path success route");
    assert_eq!(out, "alertDeleteSuccess");
}

#[test]
fn open_mods_folder_with_passes_optional_path() {
    let mut called = false;
    open_mods_folder_with(Some(PathBuf::from("/game")), |p| {
        called = true;
        assert_eq!(p, Some(std::path::Path::new("/game")));
        Ok(())
    })
    .expect("open path should be forwarded");
    assert!(called);

    let err = open_mods_folder_with(Some(PathBuf::from("/game")), |_p| {
        Err("open-failed".to_string())
    })
    .err()
    .expect("open error should bubble");
    assert_eq!(err, "open-failed");
}

#[test]
fn command_wrapper_entrypoints_forward_find_game_path_results() {
    open_mods_folder_command_with(
        || Some(PathBuf::from("/game")),
        |path| {
            assert_eq!(path, Some(std::path::Path::new("/game")));
            Ok(())
        },
    )
    .expect("open command wrapper should forward game path");

    let delete_result = delete_settings_file_command_with(
        || Some(PathBuf::from("/game")),
        |path| path.join("settings.xml"),
        |settings| {
            assert_eq!(settings, std::path::Path::new("/game/settings.xml"));
            Ok("alertDeleteSuccess".to_string())
        },
        || Err("should-not-run".to_string()),
    )
    .expect("delete command wrapper should use game path");
    assert_eq!(delete_result, "alertDeleteSuccess");

    assert!(check_for_untracked_mods_command_with(
        || Some(PathBuf::from("/mods")),
        |path| path == Some(std::path::Path::new("/mods"))
    ));
}

#[test]
fn check_for_untracked_mods_with_forwards_path() {
    assert!(check_for_untracked_mods_with(
        Some(PathBuf::from("/game")),
        |p| p == Some(std::path::Path::new("/game"))
    ));
    assert!(!check_for_untracked_mods_with(None, |p| p.is_some()));
}

#[test]
fn run_legacy_migration_with_maps_result_to_unit() {
    run_legacy_migration_with(
        std::path::Path::new("/cfg"),
        std::path::Path::new("/profiles"),
        Some(std::path::Path::new("/game")),
        |config, profiles, game| {
            assert_eq!(config, std::path::Path::new("/cfg"));
            assert_eq!(profiles, std::path::Path::new("/profiles"));
            assert_eq!(game, Some(std::path::Path::new("/game")));
            Ok(true)
        },
    )
    .expect("expected wrapper success");
}

#[test]
fn run_legacy_migration_with_propagates_errors() {
    let err = run_legacy_migration_with(
        std::path::Path::new("/cfg"),
        std::path::Path::new("/profiles"),
        None,
        |_config, _profiles, _game| Err("migration-failed".to_string()),
    )
    .err()
    .expect("migration error should bubble");
    assert_eq!(err, "migration-failed");
}

#[test]
fn run_legacy_migration_command_with_propagates_profiles_dir_error() {
    let err = run_legacy_migration_command_with(
        || Ok(PathBuf::from("/cfg.json")),
        || Err("profiles-failed".to_string()),
        crate::installation_detection::find_game_path,
        |_cfg, _profiles, _game| Ok(true),
    )
    .expect_err("profiles dir error should bubble");
    assert_eq!(err, "profiles-failed");
}

#[test]
fn detect_game_installation_with_runs_workflow_with_delegates() {
    let allowed = std::sync::Mutex::new(Vec::<PathBuf>::new());
    let logs = std::sync::Mutex::new(Vec::<String>::new());
    let mut log = |level: &str, msg: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{msg}"));
    };
    let detected = detect_game_installation_with(
        &|| Some(PathBuf::from("/tmp/game")),
        &|_path| {
            Some(GamePaths {
                game_root_path: "/tmp/game".to_string(),
                settings_root_path: "/tmp/game/Binaries/SETTINGS".to_string(),
                version_type: "Steam".to_string(),
                settings_initialized: false,
            })
        },
        &|path| {
            allowed
                .lock()
                .expect("allowed lock")
                .push(path.to_path_buf());
            Ok(())
        },
        &mut log,
        &|_p| None,
    )
    .expect("expected detected paths");
    assert_eq!(detected.game_root_path, "/tmp/game");
    assert!(allowed
        .lock()
        .expect("allowed lock")
        .iter()
        .any(|p| p == &PathBuf::from("/tmp/game")));
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Found game path")));
}

#[test]
fn save_file_with_logs_and_maps_errors() {
    let mut logs = Vec::<String>::new();
    let out = save_file_with(
        std::path::Path::new("/tmp/test.xml"),
        "<Data/>",
        |level, msg| logs.push(format!("{level}:{msg}")),
        |_path, _content| Ok(()),
    );
    assert!(out.is_ok());
    assert!(logs
        .iter()
        .any(|m| m.contains("Saving MXML to: /tmp/test.xml")));

    let mut logs_err = Vec::<String>::new();
    let err = save_file_with(
        std::path::Path::new("/tmp/test.xml"),
        "<Data/>",
        |level, msg| logs_err.push(format!("{level}:{msg}")),
        |_path, _content| Err("denied".to_string()),
    )
    .expect_err("expected save failure");
    assert!(err.contains("Failed to write to file '/tmp/test.xml'"));
    assert!(logs_err.iter().any(|m| m.contains("ERROR:Failed to write")));
}

#[test]
fn resize_window_with_builds_size_from_width_and_height() {
    let mut set_to: Option<(f64, f64)> = None;
    resize_window_with(1024.0, 720, |size| {
        set_to = Some((size.width, size.height));
        Ok(())
    })
    .expect("resize should succeed");
    assert_eq!(set_to, Some((1024.0, 720.0)));

    let err = resize_window_with(800.0, 600, |_size| Err("resize-failed".to_string()))
        .err()
        .expect("resize error should bubble");
    assert_eq!(err, "resize-failed");
}

#[test]
fn write_to_log_with_forwards_level_and_message() {
    let mut seen = String::new();
    write_to_log_with("INFO", "hello", |level, message| {
        seen = format!("{level}:{message}");
    });
    assert_eq!(seen, "INFO:hello");
}

#[test]
fn open_and_detect_wrappers_cover_none_and_error_paths() {
    open_mods_folder_with(None, |p| {
        assert!(p.is_none());
        Ok(())
    })
    .expect("none path should be forwarded");

    let detected_none = detect_game_installation_with(
        &|| None,
        &crate::installation_detection::detect_game_paths,
        &|_| Ok(()),
        &mut |_level, _msg| {},
        &|_initialized| None,
    );
    assert!(detected_none.is_none());
}

#[test]
fn command_wrappers_route_dependencies_and_errors() {
    let deleted = delete_settings_file_command_with(
        || Some(PathBuf::from("/game")),
        |path| path.join("settings.xml"),
        |settings| {
            assert_eq!(settings, std::path::Path::new("/game/settings.xml"));
            Ok("deleted".to_string())
        },
        crate::app::ops::delete_settings_without_game_path,
    )
    .expect("delete should succeed");
    assert_eq!(deleted, "deleted");

    let open_err = open_mods_folder_command_with(
        || Some(PathBuf::from("/game")),
        |_game_path| Err("open-failed".to_string()),
    )
    .expect_err("open error should bubble");
    assert_eq!(open_err, "open-failed");

    let untracked = check_for_untracked_mods_command_with(
        || Some(PathBuf::from("/game")),
        |game_path| game_path == Some(std::path::Path::new("/game")),
    );
    assert!(untracked);

    run_legacy_migration_command_with(
        || Ok(PathBuf::from("/cfg.json")),
        || Ok(PathBuf::from("/profiles")),
        || Some(PathBuf::from("/game")),
        |cfg, profiles, game| {
            assert_eq!(cfg, std::path::Path::new("/cfg.json"));
            assert_eq!(profiles, std::path::Path::new("/profiles"));
            assert_eq!(game, Some(std::path::Path::new("/game")));
            Ok(true)
        },
    )
    .expect("migration should succeed");

    let migration_err = run_legacy_migration_command_with(
        || Err("cfg-failed".to_string()),
        || Ok(PathBuf::from("/profiles")),
        || None,
        |_cfg, _profiles, _game| Ok(true),
    )
    .expect_err("config error should bubble");
    assert_eq!(migration_err, "cfg-failed");
}

#[test]
fn http_request_with_forwards_arguments_and_errors() {
    tauri::async_runtime::block_on(async {
        let seen = Arc::new(Mutex::new(String::new()));
        let seen_out = seen.clone();
        let out = http_request_with(
            "https://example.com".to_string(),
            Some("GET".to_string()),
            None,
            |url, method, headers| {
                Box::pin(async move {
                    *seen_out.lock().expect("seen lock") =
                        format!("{url}:{:?}:{:?}", method, headers);
                    Ok(HttpResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        body: "body".to_string(),
                        headers: HashMap::new(),
                    })
                })
            },
        )
        .await
        .expect("request should succeed");
        assert_eq!(out.status, 200);
        assert!(seen
            .lock()
            .expect("seen lock")
            .contains("https://example.com"));

        let err = http_request_with(
            "https://example.com".to_string(),
            None,
            None,
            |_url, _method, _headers| Box::pin(async { Err("http-failed".to_string()) }),
        )
        .await
        .err()
        .expect("request error should bubble");
        assert_eq!(err, "http-failed");
    });
}
