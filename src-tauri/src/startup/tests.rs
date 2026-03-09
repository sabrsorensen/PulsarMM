use super::{
    cache_pending_nxm_with, configure_main_window_if_present_with, decide_window_persist_action,
    handle_startup_nxm_args_with, persist_window_state_on_event_with, run_startup_setup_with,
    startup_main_window_missing_message, window_event_snapshot_with,
};
use crate::startup::logic::find_nxm_argument;
use crate::startup::logic::WindowPersistAction;
use std::path::PathBuf;
use std::sync::Mutex;

#[test]
fn startup_wrapper_uses_expected_logic_contracts() {
    let args = vec!["pulsar".to_string(), "nxm://nms/mods/123".to_string()];
    assert_eq!(
        find_nxm_argument(&args).as_deref(),
        Some("nxm://nms/mods/123")
    );
    assert_eq!(
        decide_window_persist_action(false, false, Some((10, 20))),
        WindowPersistAction::SavePosition { x: 10, y: 20 }
    );
}

#[test]
fn startup_main_window_missing_message_is_stable() {
    assert_eq!(
        startup_main_window_missing_message(),
        "Main window not found during setup"
    );
}

#[test]
fn handle_startup_nxm_args_with_logs_and_caches_when_present() {
    let logs = Mutex::new(Vec::<String>::new());
    let cached = Mutex::new(None::<String>);
    let args = vec!["pulsar".to_string(), "nxm://nms/mods/9".to_string()];
    handle_startup_nxm_args_with(
        &args,
        find_nxm_argument,
        |level, message| {
            logs.lock()
                .expect("logs lock")
                .push(format!("{level}:{message}"));
        },
        |link| *cached.lock().expect("cached lock") = Some(link),
    );
    assert!(logs
        .lock()
        .expect("logs lock")
        .iter()
        .any(|m| m.contains("Startup Argument detected")));
    assert_eq!(
        cached.lock().expect("cached lock").as_deref(),
        Some("nxm://nms/mods/9")
    );
}

#[test]
fn handle_startup_nxm_args_with_no_link_noops() {
    let logs = Mutex::new(Vec::<String>::new());
    let cache_calls = Mutex::new(0usize);
    let args = vec!["pulsar".to_string(), "--flag".to_string()];
    handle_startup_nxm_args_with(
        &args,
        |_argv| None,
        |_level, _message| {
            let _ = &logs;
        },
        |_link| *cache_calls.lock().expect("cache lock") += 1,
    );
    assert!(logs.lock().expect("logs lock").is_empty());
    assert_eq!(*cache_calls.lock().expect("cache lock"), 0);
}

#[test]
fn cache_pending_nxm_with_sets_pending_value_and_handles_none() {
    let state = crate::models::StartupState {
        pending_nxm: Mutex::new(None),
    };
    cache_pending_nxm_with(Some(&state), "nxm://nms/mods/99".to_string());
    assert_eq!(
        state.pending_nxm.lock().expect("pending lock").as_deref(),
        Some("nxm://nms/mods/99")
    );

    cache_pending_nxm_with(None, "nxm://nms/mods/100".to_string());
}

#[test]
fn run_startup_setup_with_handles_window_presence_and_missing_case() {
    let calls = Mutex::new(Vec::<String>::new());
    let args = vec!["pulsar".to_string(), "nxm://nms/mods/42".to_string()];

    run_startup_setup_with(
        &args,
        true,
        || calls.lock().expect("calls lock").push("rotate".to_string()),
        |level, message| {
            calls
                .lock()
                .expect("calls lock")
                .push(format!("log:{level}:{message}"))
        },
        || calls.lock().expect("calls lock").push("scope".to_string()),
        |_| Some("nxm://nms/mods/42".to_string()),
        |nxm| {
            calls
                .lock()
                .expect("calls lock")
                .push(format!("cache:{nxm}"))
        },
        || {
            calls
                .lock()
                .expect("calls lock")
                .push("configure".to_string())
        },
    );

    let first_run = calls.lock().expect("calls lock").clone();
    assert!(first_run.iter().any(|c| c == "rotate"));
    assert!(first_run.iter().any(|c| c == "scope"));
    assert!(first_run.iter().any(|c| c == "configure"));
    assert!(first_run
        .iter()
        .any(|c| c.starts_with("cache:nxm://nms/mods/42")));

    calls.lock().expect("calls lock").clear();

    run_startup_setup_with(
        &[],
        false,
        || calls.lock().expect("calls lock").push("rotate".to_string()),
        |level, message| {
            calls
                .lock()
                .expect("calls lock")
                .push(format!("log:{level}:{message}"))
        },
        || calls.lock().expect("calls lock").push("scope".to_string()),
        |_| None,
        |_| {},
        || {},
    );

    let second_run = calls.lock().expect("calls lock").clone();
    assert!(second_run.iter().any(|c| c == "rotate"));
    assert!(second_run.iter().any(|c| c == "scope"));
    assert!(!second_run.iter().any(|c| c == "configure"));
    assert!(second_run.iter().any(|c| {
        c.contains("Main window not found during setup") && c.starts_with("log:ERROR")
    }));
}

#[test]
fn persist_window_state_on_event_with_builds_and_forwards_action() {
    let mut captured = Vec::<(WindowPersistAction, Option<PathBuf>)>::new();
    let state_path = Some(PathBuf::from("/tmp/window-state.json"));
    persist_window_state_on_event_with(
        false,
        false,
        Some((10, 20)),
        state_path.clone(),
        |action, path| captured.push((action, path)),
    );

    assert_eq!(captured.len(), 1);
    assert_eq!(
        captured[0].0,
        WindowPersistAction::SavePosition { x: 10, y: 20 }
    );
    assert_eq!(captured[0].1, state_path);

    captured.clear();
    persist_window_state_on_event_with(true, true, Some((1, 2)), None, |action, path| {
        captured.push((action, path))
    });
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].0, WindowPersistAction::None);
    assert!(captured[0].1.is_none());

    captured.clear();
    persist_window_state_on_event_with(
        false,
        true,
        Some((11, 22)),
        Some(PathBuf::from("/tmp/state.json")),
        |action, path| captured.push((action, path)),
    );
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].0, WindowPersistAction::MarkMaximized);
    assert_eq!(
        captured[0].1.as_deref(),
        Some(PathBuf::from("/tmp/state.json").as_path())
    );

    captured.clear();
    persist_window_state_on_event_with(
        false,
        false,
        None,
        Some(PathBuf::from("/tmp/no-pos.json")),
        |action, path| captured.push((action, path)),
    );
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].0, WindowPersistAction::None);
    assert_eq!(
        captured[0].1.as_deref(),
        Some(PathBuf::from("/tmp/no-pos.json").as_path())
    );
}

#[test]
fn configure_main_window_if_present_with_runs_only_for_some() {
    let mut seen = Vec::new();
    configure_main_window_if_present_with(Some(7), |v| seen.push(v));
    configure_main_window_if_present_with::<i32>(None, |v| seen.push(v));
    assert_eq!(seen, vec![7]);
}

#[test]
fn window_event_snapshot_with_uses_defaults_on_errors() {
    let out = window_event_snapshot_with(
        || Err("min-failed".to_string()),
        || Err("max-failed".to_string()),
        || Err("pos-failed".to_string()),
        || Err("state-path-failed".to_string()),
    );
    assert_eq!(out, (false, false, None, None));

    let out = window_event_snapshot_with(
        || Ok(true),
        || Ok(false),
        || Ok((42, 24)),
        || Ok(PathBuf::from("/tmp/state.json")),
    );
    assert_eq!(
        out,
        (
            true,
            false,
            Some((42, 24)),
            Some(PathBuf::from("/tmp/state.json"))
        )
    );
}

#[test]
fn window_event_snapshot_with_handles_mixed_success_and_failures() {
    let out = window_event_snapshot_with(
        || Ok(true),
        || Err("max-failed".to_string()),
        || Ok((5, 6)),
        || Err("state-path-failed".to_string()),
    );
    assert_eq!(out, (true, false, Some((5, 6)), None));
}
