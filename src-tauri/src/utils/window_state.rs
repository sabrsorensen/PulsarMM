use crate::models::WindowState;
use std::fs;
use std::path::Path;

const MIN_RESTORABLE_COORD: i32 = -10000;

fn maximized_state(mut state: WindowState) -> WindowState {
    state.maximized = true;
    state
}

pub fn should_restore_position(state: &WindowState) -> bool {
    state.x > MIN_RESTORABLE_COORD && state.y > MIN_RESTORABLE_COORD
}

pub fn load_window_state(path: &Path) -> Option<WindowState> {
    let state_json = fs::read_to_string(path).ok()?;
    serde_json::from_str::<WindowState>(&state_json).ok()
}

pub fn save_window_state(path: &Path, state: &WindowState) -> Result<(), String> {
    let state_json = serde_json::to_string(state).expect("serializing WindowState should not fail");

    match fs::write(path, state_json) {
        Ok(()) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn save_position_state(path: &Path, x: i32, y: i32) -> Result<(), String> {
    let state = WindowState {
        x,
        y,
        maximized: false,
    };
    save_window_state(path, &state)
}

pub fn mark_window_state_maximized(path: &Path) -> Result<(), String> {
    let Some(state) = load_window_state(path) else {
        return Ok(());
    };
    save_window_state(path, &maximized_state(state))
}

#[cfg(test)]
#[path = "window_state_tests.rs"]
mod tests;
