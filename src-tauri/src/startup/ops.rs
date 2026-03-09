use super::logic::WindowPersistAction;
use crate::models::WindowState;
use crate::utils::window_state::should_restore_position;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestorePlan {
    Skip,
    InvalidCoordinates,
    Move { x: i32, y: i32, maximize: bool },
}

pub fn set_pending_nxm(pending: &Mutex<Option<String>>, nxm_link: String) {
    match pending.lock() {
        Ok(mut guard) => *guard = Some(nxm_link),
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            *guard = Some(nxm_link);
        }
    }
}

pub fn startup_nxm_detected_message(nxm_link: &str) -> String {
    format!("Startup Argument detected (NXM Link): {}", nxm_link)
}

pub fn restore_attempt_message(state: &WindowState) -> String {
    format!(
        "Attempting to restore Window: X={}, Y={}, Max={}",
        state.x, state.y, state.maximized
    )
}

pub fn plan_restore_window_state(state: Option<&WindowState>) -> RestorePlan {
    let Some(state) = state else {
        return RestorePlan::Skip;
    };

    if !should_restore_position(state) {
        return RestorePlan::InvalidCoordinates;
    }

    RestorePlan::Move {
        x: state.x,
        y: state.y,
        maximize: state.maximized,
    }
}

pub fn apply_window_persist_action(
    action: WindowPersistAction,
    state_path: Option<&Path>,
    save_position: impl Fn(&Path, i32, i32) -> Result<(), String>,
    mark_maximized: impl Fn(&Path) -> Result<(), String>,
) {
    let Some(path) = state_path else {
        return;
    };

    match action {
        WindowPersistAction::None => {}
        WindowPersistAction::SavePosition { x, y } => {
            let _ = save_position(path, x, y);
        }
        WindowPersistAction::MarkMaximized => {
            let _ = mark_maximized(path);
        }
    }
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
