use super::{
    allow_fs_scope_candidates_with, apply_runtime_window_icon_with,
    apply_steam_deck_window_decorations_with, choose_icon_source_with,
    configure_main_window_stage_with, expand_fs_scope_with, fs_scope_candidates,
    maybe_show_main_window_with, persist_window_state_action_with,
    restore_window_state_from_snapshot_with, restore_window_state_with,
    should_apply_steam_deck_window_config,
};
use crate::models::WindowState;
use crate::startup::logic::WindowPersistAction;
use crate::utils::window_state::load_window_state;
use std::path::PathBuf;
use std::sync::Mutex;

#[test]
fn fs_scope_candidates_filters_missing_entries() {
    let out = fs_scope_candidates(Some(PathBuf::from("/a")), None);
    assert_eq!(out, vec![PathBuf::from("/a")]);

    let out = fs_scope_candidates(Some(PathBuf::from("/a")), Some(PathBuf::from("/b")));
    assert_eq!(out, vec![PathBuf::from("/a"), PathBuf::from("/b")]);
}

#[test]
fn steam_deck_window_config_toggle_is_identity() {
    assert!(should_apply_steam_deck_window_config(true));
    assert!(!should_apply_steam_deck_window_config(false));
}

#[test]
fn startup_runtime_helpers_cover_fs_scope_icon_and_decorations() {
    let candidates = vec![PathBuf::from("/a"), PathBuf::from("/b")];
    let seen = Mutex::new(Vec::<PathBuf>::new());
    allow_fs_scope_candidates_with(&candidates, &mut |path| {
        seen.lock().expect("seen lock").push(path.clone());
        if path == &PathBuf::from("/b") {
            return Err("allow-failed".to_string());
        }
        Ok(())
    });
    assert_eq!(*seen.lock().expect("seen lock"), candidates);

    let chosen = choose_icon_source_with(Some(1), Some(2));
    assert_eq!(chosen, Some(1));
    let chosen = choose_icon_source_with(None::<i32>, Some(2));
    assert_eq!(chosen, Some(2));
    let chosen = choose_icon_source_with(None::<i32>, None::<i32>);
    assert_eq!(chosen, None);

    let logs = Mutex::new(Vec::<String>::new());
    let mut set_called = 0usize;
    let mut log = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"));
    };
    apply_steam_deck_window_decorations_with(true, &mut log, &mut |_decorations| {
        set_called += 1;
        Ok(())
    });
    assert_eq!(set_called, 1);
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Steam Deck detected")));

    let logs = Mutex::new(Vec::<String>::new());
    let mut log = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"));
    };
    apply_steam_deck_window_decorations_with(true, &mut log, &mut |_decorations| {
        Err("decorations-failed".to_string())
    });
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Could not enable decorations on Steam Deck")));

    let logs = Mutex::new(Vec::<String>::new());
    let mut called = 0usize;
    let mut log = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"));
    };
    apply_steam_deck_window_decorations_with(false, &mut log, &mut |_decorations| {
        called += 1;
        Ok(())
    });
    assert_eq!(called, 0);
    assert!(logs.lock().expect("logs lock").is_empty());
}

#[test]
fn maybe_show_main_window_with_logs_only_on_error() {
    let logs = Mutex::new(Vec::<String>::new());
    let mut log_ok = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"));
    };
    maybe_show_main_window_with(&mut || Ok(()), &mut log_ok);
    assert!(logs.lock().expect("logs lock").is_empty());

    let mut log_err = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"));
    };
    maybe_show_main_window_with(&mut || Err("boom".to_string()), &mut log_err);
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Failed to show main window: boom")));
}

#[test]
fn restore_window_state_from_snapshot_with_covers_move_invalid_and_skip() {
    let logs = Mutex::new(Vec::<String>::new());
    let positions = Mutex::new(Vec::<(i32, i32)>::new());
    let maximized = Mutex::new(0usize);
    let mut log = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"));
    };

    restore_window_state_from_snapshot_with(
        Some(WindowState {
            x: 10,
            y: 20,
            maximized: true,
        }),
        &mut log,
        &mut |x, y| {
            positions.lock().expect("positions lock").push((x, y));
            Ok(())
        },
        &mut || {
            *maximized.lock().expect("maximized lock") += 1;
            Ok(())
        },
    );
    assert_eq!(*positions.lock().expect("positions lock"), vec![(10, 20)]);
    assert_eq!(*maximized.lock().expect("maximized lock"), 1);

    restore_window_state_from_snapshot_with(
        Some(WindowState {
            x: -99999,
            y: 0,
            maximized: false,
        }),
        &mut log,
        &mut |_x, _y| Ok(()),
        &mut || Ok(()),
    );
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Saved coordinates were invalid")));

    restore_window_state_from_snapshot_with(
        None,
        &mut |_level, _message| {},
        &mut |_x, _y| Ok(()),
        &mut || Ok(()),
    );

    restore_window_state_from_snapshot_with(
        Some(WindowState {
            x: 1,
            y: 2,
            maximized: true,
        }),
        &mut log,
        &mut |_x, _y| Err("set-pos-failed".to_string()),
        &mut || Err("maximize-failed".to_string()),
    );
    let logs_guard = logs.lock().expect("logs lock");
    assert!(logs_guard
        .iter()
        .any(|m| m.contains("Failed to restore window position: set-pos-failed")));
    assert!(logs_guard
        .iter()
        .any(|m| m.contains("Failed to restore maximized window state: maximize-failed")));

    restore_window_state_from_snapshot_with(
        Some(WindowState {
            x: 5,
            y: 6,
            maximized: false,
        }),
        &mut |_level, _message| {},
        &mut |_x, _y| Ok(()),
        &mut || {
            panic!("maximize should not be called when maximized=false");
        },
    );
}

#[test]
fn configure_main_window_stage_with_runs_in_order() {
    let calls = Mutex::new(Vec::<&'static str>::new());
    configure_main_window_stage_with(
        &mut || calls.lock().expect("calls lock").push("icon"),
        &mut || calls.lock().expect("calls lock").push("deck"),
        &mut || calls.lock().expect("calls lock").push("restore"),
        &mut || calls.lock().expect("calls lock").push("show"),
    );
    assert_eq!(
        *calls.lock().expect("calls lock"),
        vec!["icon", "deck", "restore", "show"]
    );
}

#[test]
fn expand_fs_scope_with_ignores_directory_errors() {
    let seen = Mutex::new(Vec::<PathBuf>::new());
    expand_fs_scope_with(
        Some(PathBuf::from("/a")),
        Some(PathBuf::from("/b")),
        &mut |path| {
            seen.lock().expect("seen lock").push(path.clone());
            if path == &PathBuf::from("/b") {
                return Err("boom".to_string());
            }
            Ok(())
        },
    );
    assert_eq!(
        *seen.lock().expect("seen lock"),
        vec![PathBuf::from("/a"), PathBuf::from("/b")]
    );
}

#[test]
fn restore_window_state_with_runs_only_when_state_path_is_available() {
    let restored = Mutex::new(Vec::<Option<WindowState>>::new());
    restore_window_state_with(
        Ok(PathBuf::from("/tmp/pulsar-window-state.json")),
        |_path| {
            Some(WindowState {
                x: 1,
                y: 2,
                maximized: false,
            })
        },
        &mut |state| restored.lock().expect("restored lock").push(state),
    );
    assert_eq!(restored.lock().expect("restored lock").len(), 1);

    restore_window_state_with(Err("no path".to_string()), |_path| None, &mut |state| {
        restored.lock().expect("restored lock").push(state)
    });
    assert_eq!(restored.lock().expect("restored lock").len(), 1);

    restore_window_state_with(
        Ok(PathBuf::from("/tmp/pulsar-window-state-missing.json")),
        |_path| None,
        &mut |state| restored.lock().expect("restored lock").push(state),
    );
    let restored_values = restored.lock().expect("restored lock");
    assert_eq!(restored_values.len(), 2);
    assert!(restored_values[1].is_none());
}

#[test]
fn persist_window_state_action_with_writes_expected_state() {
    let base = std::env::temp_dir().join(format!("pulsarmm_state_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base).expect("create state dir");
    let state_path = base.join("window-state.json");

    persist_window_state_action_with(
        WindowPersistAction::SavePosition { x: 21, y: 34 },
        Some(state_path.clone()),
    );
    let state = load_window_state(&state_path).expect("saved position state should load");
    assert_eq!(state.x, 21);
    assert_eq!(state.y, 34);
    assert!(!state.maximized);

    persist_window_state_action_with(WindowPersistAction::MarkMaximized, Some(state_path.clone()));
    let state = load_window_state(&state_path).expect("maximized state should load");
    assert_eq!(state.x, 21);
    assert_eq!(state.y, 34);
    assert!(state.maximized);

    persist_window_state_action_with(WindowPersistAction::None, Some(state_path.clone()));
    let state = load_window_state(&state_path).expect("skip should preserve state");
    assert!(state.maximized);

    std::fs::remove_dir_all(base).expect("cleanup state dir");
}

#[test]
fn apply_runtime_window_icon_with_covers_success_set_error_and_no_icon() {
    let logs = Mutex::new(Vec::<String>::new());
    let mut set_calls = 0usize;
    let mut log = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"))
    };
    apply_runtime_window_icon_with(
        Some(1u8),
        Some(2u8),
        &mut |_icon| {
            set_calls += 1;
            Ok(())
        },
        &mut log,
    );
    assert_eq!(set_calls, 1);
    assert!(logs.lock().expect("logs lock").is_empty());

    apply_runtime_window_icon_with(
        Some(3u8),
        None::<u8>,
        &mut |_icon| Err("icon-failed".to_string()),
        &mut log,
    );
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Failed to set runtime window icon: icon-failed")));

    let logs = Mutex::new(Vec::<String>::new());
    let mut log = |level: &str, message: &str| {
        logs.lock()
            .expect("logs lock")
            .push(format!("{level}:{message}"))
    };
    apply_runtime_window_icon_with(None::<u8>, None::<u8>, &mut |_icon| Ok(()), &mut log);
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("No runtime window icon source found")));
}
