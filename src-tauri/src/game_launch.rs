use crate::game_launch_ops::is_steam_version;
#[cfg(target_os = "linux")]
use crate::linux::launch_strategy::LinuxSteamLaunchStep;

#[cfg(target_os = "linux")]
fn launch_game_with(
    version_type: &str,
    game_path: &str,
    is_flatpak: bool,
    run_steam_launch: impl FnOnce(bool) -> Result<String, String>,
    run_direct_launch: impl FnOnce(&str) -> Result<String, String>,
) -> Result<String, String> {
    if is_steam_version(version_type) {
        return run_steam_launch(is_flatpak);
    }

    run_direct_launch(game_path)
}

#[cfg(not(target_os = "linux"))]
fn launch_game_with(
    version_type: &str,
    game_path: &str,
    run_direct_launch: impl FnOnce(&str) -> Result<String, String>,
    run_steam_launch: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    if is_steam_version(version_type) {
        return run_steam_launch();
    }

    run_direct_launch(game_path)
}

#[cfg(target_os = "linux")]
pub(crate) fn launch_game_command_with(
    version_type: &str,
    game_path: &str,
    is_flatpak: bool,
    build_linux_steam_plan: impl FnOnce(bool) -> Vec<LinuxSteamLaunchStep>,
    execute_linux_steam_plan: impl FnOnce(bool, Vec<LinuxSteamLaunchStep>) -> Result<String, String>,
    run_direct_launch: impl FnOnce(&str) -> Result<String, String>,
) -> Result<String, String> {
    launch_game_with(
        version_type,
        game_path,
        is_flatpak,
        |flatpak| {
            let plan = build_linux_steam_plan(flatpak);
            execute_linux_steam_plan(flatpak, plan)
        },
        run_direct_launch,
    )
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn launch_game_command_with(
    version_type: &str,
    game_path: &str,
    run_direct_launch: impl FnOnce(&str) -> Result<String, String>,
    run_steam_launch: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    launch_game_with(version_type, game_path, run_direct_launch, run_steam_launch)
}

#[cfg(test)]
#[path = "game_launch_tests.rs"]
mod tests;
