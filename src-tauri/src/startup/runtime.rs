use super::ops::{plan_restore_window_state, restore_attempt_message, RestorePlan};
use crate::models::WindowState;
use crate::utils::window_state::{mark_window_state_maximized, save_position_state};
use std::path::{Path, PathBuf};

type StartupLog<'a> = dyn FnMut(&str, &str) + 'a;
type AllowDirectory<'a> = dyn FnMut(&PathBuf) -> Result<(), String> + 'a;

pub(crate) fn fs_scope_candidates(
    app_data: Option<PathBuf>,
    data_dir: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(path) = app_data {
        out.push(path);
    }
    if let Some(path) = data_dir {
        out.push(path);
    }
    out
}

pub(crate) fn allow_fs_scope_candidates_with(
    candidates: &[PathBuf],
    allow_directory: &mut AllowDirectory<'_>,
) {
    for path in candidates {
        let _ = allow_directory(path);
    }
}

pub(crate) fn expand_fs_scope_with(
    app_data: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    allow_directory: &mut AllowDirectory<'_>,
) {
    let candidates = fs_scope_candidates(app_data, data_dir);
    allow_fs_scope_candidates_with(&candidates, allow_directory);
}

pub(crate) fn choose_icon_source_with<T>(
    runtime_icon: Option<T>,
    default_icon: Option<T>,
) -> Option<T> {
    runtime_icon.or(default_icon)
}

pub(crate) fn apply_runtime_window_icon_with<T>(
    runtime_icon: Option<T>,
    default_icon: Option<T>,
    set_icon: &mut dyn FnMut(T) -> Result<(), String>,
    log: &mut StartupLog<'_>,
) {
    if let Some(icon) = choose_icon_source_with(runtime_icon, default_icon) {
        if let Err(e) = set_icon(icon) {
            log("WARN", &format!("Failed to set runtime window icon: {}", e));
        }
    } else {
        log("WARN", "No runtime window icon source found");
    }
}

pub(crate) fn should_apply_steam_deck_window_config(is_steam_deck: bool) -> bool {
    is_steam_deck
}

pub(crate) fn apply_steam_deck_window_decorations_with(
    is_steam_deck: bool,
    log: &mut StartupLog<'_>,
    set_decorations: &mut dyn FnMut(bool) -> Result<(), String>,
) {
    if should_apply_steam_deck_window_config(is_steam_deck) {
        log(
            "INFO",
            "Steam Deck detected - applying window configuration",
        );
        if let Err(e) = set_decorations(true) {
            log(
                "WARN",
                &format!("Could not enable decorations on Steam Deck: {}", e),
            );
        }
    }
}

pub(crate) fn restore_window_state_from_snapshot_with(
    state: Option<WindowState>,
    log: &mut StartupLog<'_>,
    set_position: &mut dyn FnMut(i32, i32) -> Result<(), String>,
    maximize: &mut dyn FnMut() -> Result<(), String>,
) {
    if let Some(state_ref) = state.as_ref() {
        log("INFO", &restore_attempt_message(state_ref));
    }

    match plan_restore_window_state(state.as_ref()) {
        RestorePlan::Skip => {}
        RestorePlan::InvalidCoordinates => {
            log(
                "WARN",
                "Saved coordinates were invalid (off-screen). Resetting to center.",
            );
        }
        RestorePlan::Move { x, y, maximize: m } => {
            if let Err(e) = set_position(x, y) {
                log("WARN", &format!("Failed to restore window position: {}", e));
            }
            if m {
                if let Err(e) = maximize() {
                    log(
                        "WARN",
                        &format!("Failed to restore maximized window state: {}", e),
                    );
                }
            }
        }
    }
}

pub(crate) fn restore_window_state_with(
    state_path: Result<PathBuf, String>,
    load_state: impl FnOnce(&Path) -> Option<WindowState>,
    restore: &mut dyn FnMut(Option<WindowState>),
) {
    if let Ok(state_path) = state_path {
        let state = load_state(&state_path);
        restore(state);
    }
}

pub(crate) fn maybe_show_main_window_with(
    show: &mut dyn FnMut() -> Result<(), String>,
    log: &mut StartupLog<'_>,
) {
    if let Err(e) = show() {
        log("WARN", &format!("Failed to show main window: {}", e));
    }
}

pub(crate) fn configure_main_window_stage_with(
    apply_runtime_window_icon: &mut dyn FnMut(),
    apply_steam_deck_window_config: &mut dyn FnMut(),
    restore_window_state: &mut dyn FnMut(),
    show_main_window: &mut dyn FnMut(),
) {
    apply_runtime_window_icon();
    apply_steam_deck_window_config();
    restore_window_state();
    show_main_window();
}

pub(crate) fn persist_window_state_action_with(
    action: super::logic::WindowPersistAction,
    state_path: Option<PathBuf>,
) {
    super::ops::apply_window_persist_action(
        action,
        state_path.as_deref(),
        save_position_state,
        mark_window_state_maximized,
    );
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
