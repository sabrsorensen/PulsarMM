use super::linux;
use super::single_instance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartupWindowEventKind {
    Resized,
    Moved,
    CloseRequested,
    Other,
}

pub(crate) fn apply_linux_backend_config_with(
    is_flatpak: bool,
    gdk_backend_is_set: bool,
    set_env: &mut dyn FnMut(&str, &str),
    log: &mut dyn FnMut(&str),
) {
    linux::apply_linux_backend_config_with(is_flatpak, gdk_backend_is_set, set_env, log);
}

pub(crate) fn restore_focus_if_window_available_with(
    has_window: bool,
    restore_focus: &mut dyn FnMut() -> Result<(), String>,
) -> Result<(), String> {
    if has_window {
        restore_focus()?;
    }
    Ok(())
}

pub(crate) fn run_single_instance_event_with(
    argv: &[String],
    find_nxm: &dyn Fn(&[String]) -> Option<String>,
    emit_nxm: &mut dyn FnMut(String),
    restore_focus_if_available: &mut dyn FnMut() -> Result<(), String>,
    log_info: &mut dyn FnMut(&str),
    log_warn: &mut dyn FnMut(&str),
) {
    single_instance::run_single_instance_flow_with(
        argv,
        log_info,
        find_nxm,
        emit_nxm,
        restore_focus_if_available,
        log_warn,
    );
}

fn should_persist_window_state_event(kind: StartupWindowEventKind) -> bool {
    matches!(
        kind,
        StartupWindowEventKind::Resized
            | StartupWindowEventKind::Moved
            | StartupWindowEventKind::CloseRequested
    )
}

pub(crate) fn handle_window_event_with(
    kind: StartupWindowEventKind,
    mut persist_window_state: impl FnMut(),
) {
    if should_persist_window_state_event(kind) {
        persist_window_state();
    }
}

#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
