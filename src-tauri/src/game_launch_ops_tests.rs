use super::{
    execute_linux_launch_step, execute_linux_steam_launch_plan, is_steam_version,
    launch_direct_exe, nms_exe_path, resolve_direct_launch_exe,
};
use crate::linux::launch_strategy::LinuxSteamLaunchStep;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_game_launch_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn nms_exe_path_appends_binaries_nms_exe() {
    let base = "/tmp/NMS";
    let path = nms_exe_path(base);
    let suffix = Path::new("Binaries").join("NMS.exe");
    assert!(path.ends_with(suffix));
}

#[test]
fn is_steam_version_matches_only_steam() {
    assert!(is_steam_version("Steam"));
    assert!(!is_steam_version("GOG"));
    assert!(!is_steam_version("GamePass"));
}

#[test]
fn resolve_direct_launch_exe_returns_path_when_exe_exists() {
    let dir = temp_test_dir("direct_ok");
    let bin = dir.join("Binaries");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("NMS.exe"), "stub").unwrap();

    let exe = resolve_direct_launch_exe(dir.to_string_lossy().as_ref()).unwrap();
    assert!(exe.ends_with(Path::new("Binaries").join("NMS.exe")));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn resolve_direct_launch_exe_errors_when_missing() {
    let dir = temp_test_dir("direct_missing");
    let err = resolve_direct_launch_exe(dir.to_string_lossy().as_ref()).unwrap_err();
    assert_eq!(err, "Could not find NMS.exe in Binaries folder.");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn execute_linux_steam_launch_plan_returns_first_successful_step() {
    let plan = vec![
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
    ];
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut spawn_ok = |_program: &str, _args: &[String]| Ok(());
    let mut open_ok = |_url: &str| Ok(());
    let mut log = |lvl: &str, msg: &str| logs.push((lvl.to_string(), msg.to_string()));
    let out = execute_linux_steam_launch_plan(false, plan, &mut spawn_ok, &mut open_ok, &mut log)
        .expect("expected launch success");
    assert!(out.contains("steam"));
    assert!(logs
        .iter()
        .any(|(lvl, msg)| lvl == "INFO" && msg.contains("succeeded")));
}

#[test]
fn execute_linux_steam_launch_plan_returns_failure_message_when_all_fail() {
    let plan = vec![
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
    ];
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut spawn_err = |_program: &str, _args: &[String]| Err("spawn failed".to_string());
    let mut open_err = |_url: &str| Err("open failed".to_string());
    let mut log = |lvl: &str, msg: &str| logs.push((lvl.to_string(), msg.to_string()));
    let out = execute_linux_steam_launch_plan(false, plan, &mut spawn_err, &mut open_err, &mut log)
        .expect_err("expected launch failure");
    assert!(out.contains("Failed to launch No Man's Sky via Steam."));
    assert!(logs.iter().any(|(lvl, _)| lvl == "ERROR"));
}

#[test]
fn execute_linux_steam_launch_plan_uses_open_url_fallback_after_command_failure() {
    let plan = vec![
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
    ];
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut spawn_err = |_program: &str, _args: &[String]| Err("spawn failed".to_string());
    let mut open_ok = |_url: &str| Ok(());
    let mut log = |lvl: &str, msg: &str| logs.push((lvl.to_string(), msg.to_string()));
    let out = execute_linux_steam_launch_plan(false, plan, &mut spawn_err, &mut open_ok, &mut log)
        .expect("open_url fallback should succeed");

    assert!(out.contains("steam://"));
    assert!(logs
        .iter()
        .any(|(lvl, msg)| lvl == "INFO" && msg.contains("succeeded")));
}

#[test]
fn execute_linux_launch_step_covers_command_and_open_url_paths() {
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut spawn_ok = |_program: &str, _args: &[String]| Ok(());
    let mut open_ok = |_url: &str| Ok(());

    let out = execute_linux_launch_step(
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        &mut spawn_ok,
        &mut open_ok,
        &mut |lvl, msg| logs.push((lvl.to_string(), msg.to_string())),
    )
    .expect("command step should succeed");
    assert!(out.contains("steam"));

    let mut spawn_err = |_program: &str, _args: &[String]| Err("spawn failed".to_string());
    let err = execute_linux_launch_step(
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        &mut spawn_err,
        &mut open_ok,
        &mut |_lvl, _msg| {},
    )
    .expect_err("command step should fail");
    assert!(err.0.contains("steam"));
    assert_eq!(err.1, "spawn failed");

    let out = execute_linux_launch_step(
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
        &mut spawn_ok,
        &mut open_ok,
        &mut |lvl, msg| logs.push((lvl.to_string(), msg.to_string())),
    )
    .expect("open-url step should succeed");
    assert!(out.contains("steam://"));

    let mut open_err = |_url: &str| Err("open failed".to_string());
    let err = execute_linux_launch_step(
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
        &mut spawn_ok,
        &mut open_err,
        &mut |_lvl, _msg| {},
    )
    .expect_err("open-url step should fail");
    assert!(err.0.contains("steam://"));
    assert_eq!(err.1, "open failed");

    assert!(logs
        .iter()
        .any(|(lvl, msg)| lvl == "INFO" && msg.contains("succeeded")));
}

#[test]
fn execute_linux_steam_launch_plan_uses_flatpak_failure_prefix_and_handles_empty_plan() {
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut spawn_ok = |_program: &str, _args: &[String]| Ok(());
    let mut open_ok = |_url: &str| Ok(());
    let mut log = |lvl: &str, msg: &str| logs.push((lvl.to_string(), msg.to_string()));
    let out =
        execute_linux_steam_launch_plan(true, Vec::new(), &mut spawn_ok, &mut open_ok, &mut log)
            .expect_err("empty plan should fail");

    assert!(out.contains("Failed to launch No Man's Sky via Flatpak host bridge."));
    assert!(logs.iter().any(|(lvl, _)| lvl == "ERROR"));
}

#[test]
fn launch_direct_exe_uses_open_path_and_logs_success() {
    let dir = temp_test_dir("launch_direct");
    let bin = dir.join("Binaries");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("NMS.exe"), "stub").unwrap();
    let mut opened: Option<PathBuf> = None;
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut open_path = |p: &Path| {
        opened = Some(p.to_path_buf());
        Ok(())
    };
    let mut log = |lvl: &str, msg: &str| logs.push((lvl.to_string(), msg.to_string()));

    let out = launch_direct_exe(dir.to_string_lossy().as_ref(), &mut open_path, &mut log)
        .expect("expected direct launch success");

    assert_eq!(out, "direct NMS.exe");
    assert!(opened
        .expect("expected opened path")
        .ends_with(Path::new("Binaries").join("NMS.exe")));
    assert!(logs
        .iter()
        .any(|(lvl, msg)| lvl == "INFO" && msg.contains("direct NMS.exe")));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn launch_direct_exe_propagates_open_path_error() {
    let dir = temp_test_dir("launch_direct_error");
    let bin = dir.join("Binaries");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("NMS.exe"), "stub").unwrap();
    let mut open_path = |_p: &Path| Err("open-path-failed".to_string());
    let mut log = |_lvl: &str, _msg: &str| panic!("log should not run when direct launch fails");

    let err = launch_direct_exe(dir.to_string_lossy().as_ref(), &mut open_path, &mut log)
        .expect_err("open path error should bubble");

    assert_eq!(err, "open-path-failed");
    fs::remove_dir_all(dir).unwrap();
}
