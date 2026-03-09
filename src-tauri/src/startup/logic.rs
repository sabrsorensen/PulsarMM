#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowPersistAction {
    None,
    SavePosition { x: i32, y: i32 },
    MarkMaximized,
}

pub fn find_nxm_argument(args: &[String]) -> Option<String> {
    args.iter().find(|arg| arg.starts_with("nxm://")).cloned()
}

pub fn decide_window_persist_action(
    is_minimized: bool,
    is_maximized: bool,
    position: Option<(i32, i32)>,
) -> WindowPersistAction {
    if is_minimized {
        return WindowPersistAction::None;
    }
    if is_maximized {
        return WindowPersistAction::MarkMaximized;
    }
    match position {
        Some((x, y)) => WindowPersistAction::SavePosition { x, y },
        None => WindowPersistAction::None,
    }
}

#[cfg(test)]
#[path = "logic_tests.rs"]
mod tests;
