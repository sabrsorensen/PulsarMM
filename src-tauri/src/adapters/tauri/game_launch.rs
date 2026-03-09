use crate::game_launch::launch_game_command_with;
use crate::game_launch_ops::{execute_linux_steam_launch_plan, launch_direct_exe};
#[cfg(target_os = "linux")]
use crate::linux::launch_strategy::LinuxSteamLaunchStep;
use crate::{linux, log_internal};
use tauri::AppHandle;

#[cfg(target_os = "linux")]
pub(crate) fn launch_game_runtime_with(
    version_type: &str,
    game_path: &str,
    is_flatpak: bool,
    build_linux_steam_plan: impl FnOnce(bool) -> Vec<LinuxSteamLaunchStep>,
    execute_linux_steam_plan: impl FnOnce(bool, Vec<LinuxSteamLaunchStep>) -> Result<String, String>,
    mut open_path: impl FnMut(&str) -> Result<(), String>,
    mut log: impl FnMut(&str, &str),
) -> Result<String, String> {
    launch_game_command_with(
        version_type,
        game_path,
        is_flatpak,
        build_linux_steam_plan,
        execute_linux_steam_plan,
        |path| {
            let mut open_direct_path = |open_path_arg: &std::path::Path| {
                open_path(open_path_arg.to_string_lossy().as_ref())
            };
            let mut log_direct = |level: &str, message: &str| log(level, message);
            launch_direct_exe(path, &mut open_direct_path, &mut log_direct)
        },
    )
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn launch_game_runtime_with(
    version_type: &str,
    game_path: &str,
    open_steam: impl FnOnce() -> Result<(), String>,
    open_path: impl FnOnce(&str) -> Result<(), String>,
    mut log: impl FnMut(&str, &str),
) -> Result<String, String> {
    launch_game_command_with(
        version_type,
        game_path,
        |path| {
            let mut open_direct_path = |open_path_arg: &std::path::Path| {
                open_path(open_path_arg.to_string_lossy().as_ref())
            };
            let mut log_direct = |level: &str, message: &str| log(level, message);
            launch_direct_exe(path, &mut open_direct_path, &mut log_direct)
        },
        || {
            open_steam()?;
            log("INFO", "Game launch succeeded using steam://run/275850");
            Ok("steam://run/275850".to_string())
        },
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn launch_game_command_entry_with(
    version_type: String,
    game_path: String,
    is_flatpak: bool,
    build_linux_steam_plan: impl FnOnce(bool) -> Vec<LinuxSteamLaunchStep>,
    execute_linux_steam_plan: impl FnOnce(bool, Vec<LinuxSteamLaunchStep>) -> Result<String, String>,
    open_path: impl FnMut(&str) -> Result<(), String>,
    log: impl FnMut(&str, &str),
) -> Result<String, String> {
    launch_game_runtime_with(
        version_type.as_str(),
        &game_path,
        is_flatpak,
        build_linux_steam_plan,
        execute_linux_steam_plan,
        open_path,
        log,
    )
}

#[tauri::command]
pub fn launch_game(
    app: AppHandle,
    version_type: String,
    game_path: String,
) -> Result<String, String> {
    let mut spawn_command = |program: &str, args: &[String]| {
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    };
    let mut open_url = |url: &str| open::that(url).map_err(|e| e.to_string());
    let mut log_launch = |level: &str, message: &str| log_internal(&app, level, message);

    launch_game_command_entry_with(
        version_type,
        game_path,
        linux::runtime::is_flatpak_runtime(),
        linux::launch_strategy::linux_steam_launch_plan,
        |is_flatpak, plan| {
            execute_linux_steam_launch_plan(
                is_flatpak,
                plan,
                &mut spawn_command,
                &mut open_url,
                &mut log_launch,
            )
        },
        |path| open::that(path).map_err(|e| e.to_string()),
        |level, message| log_internal(&app, level, message),
    )
}
