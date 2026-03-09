use super::{decide_window_persist_action, find_nxm_argument, WindowPersistAction};

#[test]
fn find_nxm_argument_returns_first_nxm_link() {
    let args = vec![
        "pulsar".to_string(),
        "--flag".to_string(),
        "nxm://nms/mods/123".to_string(),
        "nxm://nms/mods/999".to_string(),
    ];
    let found = find_nxm_argument(&args);
    assert_eq!(found.as_deref(), Some("nxm://nms/mods/123"));
}

#[test]
fn find_nxm_argument_returns_none_when_absent() {
    let args = vec!["pulsar".to_string(), "--flag".to_string()];
    assert!(find_nxm_argument(&args).is_none());
}

#[test]
fn decide_window_persist_action_prefers_none_when_minimized() {
    assert_eq!(
        decide_window_persist_action(true, false, Some((1, 2))),
        WindowPersistAction::None
    );
}

#[test]
fn decide_window_persist_action_marks_maximized() {
    assert_eq!(
        decide_window_persist_action(false, true, Some((1, 2))),
        WindowPersistAction::MarkMaximized
    );
}

#[test]
fn decide_window_persist_action_saves_position_for_normal_window() {
    assert_eq!(
        decide_window_persist_action(false, false, Some((10, 20))),
        WindowPersistAction::SavePosition { x: 10, y: 20 }
    );
}

#[test]
fn decide_window_persist_action_returns_none_when_position_missing() {
    assert_eq!(
        decide_window_persist_action(false, false, None),
        WindowPersistAction::None
    );
}
