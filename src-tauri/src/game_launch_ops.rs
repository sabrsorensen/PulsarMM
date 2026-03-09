use crate::linux::launch_strategy::{
    steam_launch_failure_message, step_label, LinuxSteamLaunchStep,
};
use std::path::{Path, PathBuf};

type SpawnCommand<'a> = dyn FnMut(&str, &[String]) -> Result<(), String> + 'a;
type OpenUrl<'a> = dyn FnMut(&str) -> Result<(), String> + 'a;
type Log<'a> = dyn FnMut(&str, &str) + 'a;
type OpenPath<'a> = dyn FnMut(&Path) -> Result<(), String> + 'a;

pub fn nms_exe_path(game_path: &str) -> PathBuf {
    Path::new(game_path).join("Binaries").join("NMS.exe")
}

pub fn is_steam_version(version_type: &str) -> bool {
    version_type == "Steam"
}

pub fn resolve_direct_launch_exe(game_path: &str) -> Result<PathBuf, String> {
    let exe_path = nms_exe_path(game_path);
    if exe_path.exists() {
        Ok(exe_path)
    } else {
        Err("Could not find NMS.exe in Binaries folder.".to_string())
    }
}

fn log_launch_success(attempt_label: &str, log: &mut Log<'_>) {
    log(
        "INFO",
        &format!("Game launch succeeded using {}", attempt_label),
    );
}

pub(crate) fn execute_linux_launch_step(
    step: LinuxSteamLaunchStep,
    spawn_command: &mut SpawnCommand<'_>,
    open_url: &mut OpenUrl<'_>,
    log: &mut Log<'_>,
) -> Result<String, (String, String)> {
    let attempt_label = step_label(&step);

    match step {
        LinuxSteamLaunchStep::Command { program, args } => {
            if let Err(e) = spawn_command(&program, &args) {
                return Err((attempt_label, e));
            }
            log_launch_success(&attempt_label, log);
            Ok(attempt_label)
        }
        LinuxSteamLaunchStep::OpenUrl(url) => {
            if let Err(e) = open_url(&url) {
                return Err((attempt_label, e));
            }
            log_launch_success(&attempt_label, log);
            Ok(attempt_label)
        }
    }
}

pub fn execute_linux_steam_launch_plan(
    is_flatpak: bool,
    plan: Vec<LinuxSteamLaunchStep>,
    spawn_command: &mut SpawnCommand<'_>,
    open_url: &mut OpenUrl<'_>,
    log: &mut Log<'_>,
) -> Result<String, String> {
    let mut launch_errors: Vec<String> = Vec::new();
    let mut attempts: Vec<String> = Vec::new();

    for step in plan {
        match execute_linux_launch_step(step, spawn_command, open_url, log) {
            Ok(attempt_label) => {
                attempts.push(attempt_label.clone());
                return Ok(attempt_label);
            }
            Err((attempt_label, e)) => {
                attempts.push(attempt_label.clone());
                launch_errors.push(format!("{} failed: {}", attempt_label, e));
            }
        }
    }

    let fail_msg = steam_launch_failure_message(is_flatpak, &attempts, &launch_errors);
    log("ERROR", &fail_msg);
    Err(fail_msg)
}

pub fn launch_direct_exe(
    game_path: &str,
    open_path: &mut OpenPath<'_>,
    log: &mut Log<'_>,
) -> Result<String, String> {
    let exe_path = resolve_direct_launch_exe(game_path)?;
    open_path(exe_path.as_path())?;
    log("INFO", "Game launch succeeded using direct NMS.exe");
    Ok("direct NMS.exe".to_string())
}

#[cfg(test)]
#[path = "game_launch_ops_tests.rs"]
mod tests;
