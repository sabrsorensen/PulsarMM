pub(crate) mod logic;
pub(crate) mod ops;
pub(crate) mod runtime;

use self::logic::decide_window_persist_action;
use self::ops::{set_pending_nxm, startup_nxm_detected_message};
use crate::models::StartupState;
use std::path::PathBuf;

pub(crate) fn handle_startup_nxm_args_with(
    args: &[String],
    find_nxm: impl FnOnce(&[String]) -> Option<String>,
    mut log: impl FnMut(&str, &str),
    mut cache: impl FnMut(String),
) {
    if let Some(nxm_link) = find_nxm(args) {
        log("INFO", &startup_nxm_detected_message(&nxm_link));
        cache(nxm_link);
    }
}

pub(crate) fn cache_pending_nxm_with(state: Option<&StartupState>, nxm_link: String) {
    if let Some(state_ref) = state {
        set_pending_nxm(&state_ref.pending_nxm, nxm_link);
    }
}

pub(crate) fn startup_main_window_missing_message() -> &'static str {
    "Main window not found during setup"
}

pub(crate) fn configure_main_window_if_present_with<T>(
    window: Option<T>,
    mut configure_window: impl FnMut(T),
) {
    if let Some(window) = window {
        configure_window(window);
    }
}

pub(crate) fn run_startup_setup_with(
    args: &[String],
    main_window_exists: bool,
    mut rotate_logs: impl FnMut(),
    mut log: impl FnMut(&str, &str),
    mut expand_scope: impl FnMut(),
    mut find_nxm: impl FnMut(&[String]) -> Option<String>,
    mut cache_nxm: impl FnMut(String),
    mut configure_main_window: impl FnMut(),
) {
    rotate_logs();
    log("INFO", "=== PULSAR MOD MANAGER STARTUP ===");

    expand_scope();

    handle_startup_nxm_args_with(args, |a| find_nxm(a), &mut log, &mut cache_nxm);

    if main_window_exists {
        configure_main_window();
    } else {
        log("ERROR", startup_main_window_missing_message());
    }
}

pub(crate) fn persist_window_state_on_event_with(
    is_minimized: bool,
    is_maximized: bool,
    outer_position: Option<(i32, i32)>,
    state_path: Option<PathBuf>,
    mut persist_action: impl FnMut(self::logic::WindowPersistAction, Option<PathBuf>),
) {
    let action = decide_window_persist_action(is_minimized, is_maximized, outer_position);
    persist_action(action, state_path);
}

pub(crate) fn window_event_snapshot_with(
    is_minimized: impl FnOnce() -> Result<bool, String>,
    is_maximized: impl FnOnce() -> Result<bool, String>,
    outer_position: impl FnOnce() -> Result<(i32, i32), String>,
    state_path: impl FnOnce() -> Result<PathBuf, String>,
) -> (bool, bool, Option<(i32, i32)>, Option<PathBuf>) {
    (
        is_minimized().unwrap_or(false),
        is_maximized().unwrap_or(false),
        outer_position().ok(),
        state_path().ok(),
    )
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
