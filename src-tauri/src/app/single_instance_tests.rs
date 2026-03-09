use super::{
    handle_single_instance_with, restore_focus_for_main_window_with, run_single_instance_flow_with,
};

#[test]
fn handle_single_instance_with_emits_and_reports_focus_error() {
    let args = vec!["pulsar".to_string(), "nxm://foo".to_string()];
    let mut emitted = Vec::new();
    let mut warnings = Vec::new();

    handle_single_instance_with(
        &args,
        &|_argv| Some("nxm://foo".to_string()),
        &mut |link| emitted.push(link),
        &mut || Err("focus failed".to_string()),
        &mut |message| warnings.push(message.to_string()),
    );

    assert_eq!(emitted, vec!["nxm://foo".to_string()]);
    assert_eq!(warnings, vec!["focus failed".to_string()]);
}

#[test]
fn handle_single_instance_with_no_nxm_and_focus_success() {
    let args = vec!["pulsar".to_string(), "--silent".to_string()];
    handle_single_instance_with(
        &args,
        &|_argv| None,
        &mut std::mem::drop::<String>,
        &mut || Ok(()),
        &mut |_message| {},
    );
}

#[test]
fn run_single_instance_flow_with_logs_args_and_formats_warnings() {
    let args = vec!["pulsar".to_string(), "nxm://bar".to_string()];
    let mut infos = Vec::new();
    let mut emitted = Vec::new();
    let mut warns = Vec::new();

    run_single_instance_flow_with(
        &args,
        &mut |msg| infos.push(msg.to_string()),
        &|_argv| Some("nxm://bar".to_string()),
        &mut |link| emitted.push(link),
        &mut || Err("focus boom".to_string()),
        &mut |msg| warns.push(msg.to_string()),
    );

    assert!(infos
        .iter()
        .any(|m| m.contains("New instance detected, args:")));
    assert_eq!(emitted, vec!["nxm://bar".to_string()]);
    assert_eq!(
        warns,
        vec!["single-instance window activation failed: focus boom".to_string()]
    );
}

#[test]
fn restore_focus_for_main_window_with_covers_success_and_failures() {
    restore_focus_for_main_window_with(&mut || Ok(()), &mut || Ok(()))
        .expect("focus should succeed");

    let unminimize_err = restore_focus_for_main_window_with(
        &mut || Err("unminimize failed".to_string()),
        &mut || Ok(()),
    )
    .expect_err("unminimize error should bubble");
    assert_eq!(unminimize_err, "unminimize failed");

    let focus_err =
        restore_focus_for_main_window_with(&mut || Ok(()), &mut || Err("focus failed".to_string()))
            .expect_err("focus error should bubble");
    assert_eq!(focus_err, "focus failed");
}
