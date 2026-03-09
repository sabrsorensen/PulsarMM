use super::{
    apply_linux_backend_config_with, handle_window_event_with,
    restore_focus_if_window_available_with, run_single_instance_event_with,
    should_persist_window_state_event, StartupWindowEventKind,
};
use std::collections::HashMap;

#[test]
fn apply_linux_backend_config_with_handles_flatpak_and_non_flatpak() {
    let mut env = HashMap::new();
    let mut logs = Vec::new();
    apply_linux_backend_config_with(
        false,
        false,
        &mut |k, v| {
            env.insert(k.to_string(), v.to_string());
        },
        &mut |m| logs.push(m.to_string()),
    );
    assert_eq!(env.get("GDK_BACKEND").map(String::as_str), Some("x11"));
    assert!(logs.iter().any(|m| m.contains("Forced GDK_BACKEND=x11")));

    env.clear();
    logs.clear();
    apply_linux_backend_config_with(true, false, &mut |_k, _v| {}, &mut |m| {
        logs.push(m.to_string())
    });
    assert!(env.is_empty());
    assert!(logs
        .iter()
        .any(|m| m.contains("Running in Flatpak - using native display backend")));
}

#[test]
fn restore_focus_if_window_available_with_covers_branches() {
    let mut calls = 0usize;
    restore_focus_if_window_available_with(false, &mut || {
        calls += 1;
        Ok(())
    })
    .expect("no-window branch should succeed");
    assert_eq!(calls, 0);

    restore_focus_if_window_available_with(true, &mut || {
        calls += 1;
        Ok(())
    })
    .expect("window branch should call restore");
    assert_eq!(calls, 1);

    let err = restore_focus_if_window_available_with(true, &mut || Err("focus-failed".to_string()))
        .err()
        .expect("error should propagate");
    assert_eq!(err, "focus-failed");
}

#[test]
fn run_single_instance_event_with_forwards_emit_logs_and_warnings() {
    let argv = vec!["pulsar".to_string(), "nxm://nms/mods/77".to_string()];
    let mut emitted = Vec::<String>::new();
    let mut infos = Vec::<String>::new();
    let mut warns = Vec::<String>::new();

    run_single_instance_event_with(
        &argv,
        &|_args| Some("nxm://nms/mods/77".to_string()),
        &mut |nxm| emitted.push(nxm),
        &mut || Err("focus-failed".to_string()),
        &mut |m| infos.push(m.to_string()),
        &mut |m| warns.push(m.to_string()),
    );

    assert_eq!(emitted, vec!["nxm://nms/mods/77".to_string()]);
    assert!(infos
        .iter()
        .any(|m| m.contains("New instance detected, args:")));
    assert!(warns
        .iter()
        .any(|m| m.contains("single-instance window activation failed: focus-failed")));
}

#[test]
fn run_single_instance_event_with_handles_no_nxm_and_focus_success() {
    let argv = vec!["pulsar".to_string(), "--silent".to_string()];
    let mut infos = Vec::<String>::new();
    let mut warns = Vec::<String>::new();

    run_single_instance_event_with(
        &argv,
        &|_args| None,
        &mut |_nxm| {},
        &mut || Ok(()),
        &mut |m| infos.push(m.to_string()),
        &mut |m| warns.push(m.to_string()),
    );

    assert!(infos
        .iter()
        .any(|m| m.contains("New instance detected, args:")));
    assert!(warns.is_empty());
}

#[test]
fn run_single_instance_event_with_emits_nxm_and_restores_focus_without_warning() {
    let argv = vec!["pulsar".to_string(), "nxm://nms/mods/88".to_string()];
    let mut emitted = Vec::<String>::new();
    let mut infos = Vec::<String>::new();
    let mut warns = Vec::<String>::new();
    let mut focus_calls = 0usize;

    run_single_instance_event_with(
        &argv,
        &|_args| Some("nxm://nms/mods/88".to_string()),
        &mut |nxm| emitted.push(nxm),
        &mut || {
            focus_calls += 1;
            Ok(())
        },
        &mut |m| infos.push(m.to_string()),
        &mut |m| warns.push(m.to_string()),
    );

    assert_eq!(emitted, vec!["nxm://nms/mods/88".to_string()]);
    assert_eq!(focus_calls, 1);
    assert!(infos
        .iter()
        .any(|m| m.contains("New instance detected, args:")));
    assert!(warns.is_empty());
}

#[test]
fn should_persist_window_state_event_matches_expected_kinds() {
    assert!(should_persist_window_state_event(
        StartupWindowEventKind::Resized
    ));
    assert!(should_persist_window_state_event(
        StartupWindowEventKind::Moved
    ));
    assert!(should_persist_window_state_event(
        StartupWindowEventKind::CloseRequested
    ));
    assert!(!should_persist_window_state_event(
        StartupWindowEventKind::Other
    ));
}

#[test]
fn handle_window_event_with_invokes_persist_only_for_persistable_events() {
    let mut calls = 0usize;
    handle_window_event_with(StartupWindowEventKind::Other, || {});
    assert_eq!(calls, 0);

    handle_window_event_with(StartupWindowEventKind::Resized, || {
        calls += 1;
    });
    assert_eq!(calls, 1);

    handle_window_event_with(StartupWindowEventKind::Moved, || {
        calls += 1;
    });
    assert_eq!(calls, 2);

    handle_window_event_with(StartupWindowEventKind::CloseRequested, || {
        calls += 1;
    });
    assert_eq!(calls, 3);
}
