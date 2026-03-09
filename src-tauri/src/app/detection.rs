use crate::models::GamePaths;
use std::path::{Path, PathBuf};

type DetectionLog<'a> = dyn FnMut(&str, &str) + 'a;

pub fn found_game_path_message(path: &Path) -> String {
    format!("Found game path: {:?}", path)
}

pub fn detection_failure_message() -> &'static str {
    "Game detection failed: No valid installation found."
}

pub fn missing_settings_warning(settings_initialized: bool) -> Option<&'static str> {
    if settings_initialized {
        None
    } else {
        Some(
            "Game install found, but GCMODSETTINGS.MXML is missing. Ask user to run the game once.",
        )
    }
}

pub fn detect_game_installation_with(
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    detect_game_paths: &dyn Fn(&Path) -> Option<GamePaths>,
) -> Option<(PathBuf, GamePaths)> {
    let path = find_game_path()?;
    let detected = detect_game_paths(&path)?;
    Some((path, detected))
}

pub fn run_game_detection_workflow(
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    detect_game_paths: &dyn Fn(&Path) -> Option<GamePaths>,
    allow_directory: &dyn Fn(&Path) -> Result<(), String>,
    log: &mut DetectionLog<'_>,
    missing_settings_warning: &dyn Fn(bool) -> Option<&'static str>,
) -> Option<GamePaths> {
    log("INFO", "Starting Game Detection...");

    if let Some((path, detected)) = detect_game_installation_with(find_game_path, detect_game_paths)
    {
        log("INFO", &found_game_path_message(&path));

        match allow_directory(&path) {
            Ok(()) => log(
                "INFO",
                &format!("Expanded fs scope for game path: {:?}", path),
            ),
            Err(err) => log(
                "WARN",
                &format!("Failed to expand fs scope for game path: {}", err),
            ),
        }

        if let Some(warn) = missing_settings_warning(detected.settings_initialized) {
            log("WARN", warn);
        }
        return Some(detected);
    }

    log("WARN", detection_failure_message());
    None
}

#[cfg(test)]
#[path = "detection_tests.rs"]
mod tests;
