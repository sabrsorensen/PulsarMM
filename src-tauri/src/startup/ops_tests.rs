use super::{
    apply_window_persist_action, plan_restore_window_state, restore_attempt_message,
    set_pending_nxm, startup_nxm_detected_message, RestorePlan,
};
use crate::models::WindowState;
use crate::startup::logic::WindowPersistAction;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

thread_local! {
    static CALLBACK_CALLS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn reset_callback_calls() {
    CALLBACK_CALLS.with(|calls| calls.borrow_mut().clear());
}

fn callback_calls() -> Vec<String> {
    CALLBACK_CALLS.with(|calls| calls.borrow().clone())
}

fn record_save(_path: &Path, x: i32, y: i32) -> Result<(), String> {
    CALLBACK_CALLS.with(|calls| calls.borrow_mut().push(format!("save:{x}:{y}")));
    Ok(())
}

fn record_mark(_path: &Path) -> Result<(), String> {
    CALLBACK_CALLS.with(|calls| calls.borrow_mut().push("mark".to_string()));
    Ok(())
}

fn fail_save(_path: &Path, _x: i32, _y: i32) -> Result<(), String> {
    Err("save-failed".to_string())
}

fn fail_mark(_path: &Path) -> Result<(), String> {
    Err("mark-failed".to_string())
}

#[test]
fn set_pending_nxm_stores_latest_link() {
    let pending = Mutex::new(None);
    set_pending_nxm(&pending, "nxm://a".to_string());
    let value = pending.lock().expect("lock should succeed").clone();
    assert_eq!(value.as_deref(), Some("nxm://a"));
}

#[test]
fn set_pending_nxm_recovers_from_poisoned_mutex() {
    let pending = Mutex::new(None);

    let _ = std::panic::catch_unwind(|| {
        let _guard = pending.lock().expect("lock before poison");
        panic!("intentional poison");
    });

    set_pending_nxm(&pending, "nxm://poisoned".to_string());

    let value = pending
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    assert_eq!(value.as_deref(), Some("nxm://poisoned"));
}

#[test]
fn startup_nxm_detected_message_formats_expected_text() {
    assert_eq!(
        startup_nxm_detected_message("nxm://nms/mods/1"),
        "Startup Argument detected (NXM Link): nxm://nms/mods/1"
    );
}

#[test]
fn restore_attempt_message_formats_coordinates_and_max_flag() {
    let msg = restore_attempt_message(&WindowState {
        x: 10,
        y: 20,
        maximized: true,
    });
    assert_eq!(msg, "Attempting to restore Window: X=10, Y=20, Max=true");
}

#[test]
fn plan_restore_window_state_covers_all_paths() {
    assert_eq!(plan_restore_window_state(None), RestorePlan::Skip);

    let invalid = WindowState {
        x: -10000,
        y: 10,
        maximized: false,
    };
    assert_eq!(
        plan_restore_window_state(Some(&invalid)),
        RestorePlan::InvalidCoordinates
    );

    let normal = WindowState {
        x: 10,
        y: 20,
        maximized: false,
    };
    assert_eq!(
        plan_restore_window_state(Some(&normal)),
        RestorePlan::Move {
            x: 10,
            y: 20,
            maximize: false
        }
    );

    let maximized = WindowState {
        x: 30,
        y: 40,
        maximized: true,
    };
    assert_eq!(
        plan_restore_window_state(Some(&maximized)),
        RestorePlan::Move {
            x: 30,
            y: 40,
            maximize: true
        }
    );
}

#[test]
fn apply_window_persist_action_handles_none_and_missing_path() {
    reset_callback_calls();
    let state_path = PathBuf::from("/tmp/state.json");
    apply_window_persist_action(
        WindowPersistAction::None,
        Some(state_path.as_path()),
        record_save,
        record_mark,
    );
    assert!(callback_calls().is_empty());

    reset_callback_calls();
    apply_window_persist_action(
        WindowPersistAction::SavePosition { x: 1, y: 2 },
        None,
        record_save,
        record_mark,
    );
    assert!(callback_calls().is_empty());
}

#[test]
fn apply_window_persist_action_invokes_expected_operation() {
    reset_callback_calls();
    let state_path = PathBuf::from("/tmp/state.json");
    apply_window_persist_action(
        WindowPersistAction::SavePosition { x: 10, y: 20 },
        Some(state_path.as_path()),
        record_save,
        record_mark,
    );

    assert_eq!(callback_calls(), vec!["save:10:20".to_string()]);

    reset_callback_calls();
    apply_window_persist_action(
        WindowPersistAction::MarkMaximized,
        Some(state_path.as_path()),
        record_save,
        record_mark,
    );

    assert_eq!(callback_calls(), vec!["mark".to_string()]);
}

#[test]
fn apply_window_persist_action_ignores_save_and_mark_errors() {
    let state_path = PathBuf::from("/tmp/state.json");
    apply_window_persist_action(
        WindowPersistAction::SavePosition { x: 1, y: 2 },
        Some(state_path.as_path()),
        fail_save,
        record_mark,
    );

    apply_window_persist_action(
        WindowPersistAction::MarkMaximized,
        Some(state_path.as_path()),
        record_save,
        fail_mark,
    );
}

#[test]
fn startup_ops_test_callbacks_are_callable() {
    reset_callback_calls();
    record_save(Path::new("/tmp/state.json"), 3, 4).expect("save callback");
    record_mark(Path::new("/tmp/state.json")).expect("mark callback");
    assert_eq!(
        callback_calls(),
        vec!["save:3:4".to_string(), "mark".to_string()]
    );
    assert_eq!(
        fail_save(Path::new("/tmp/state.json"), 0, 0).expect_err("save error callback"),
        "save-failed"
    );
    assert_eq!(
        fail_mark(Path::new("/tmp/state.json")).expect_err("mark error callback"),
        "mark-failed"
    );
}
