use super::{launch_game_command_with, launch_game_with};
#[cfg(target_os = "linux")]
use crate::adapters::tauri::game_launch::{
    launch_game_command_entry_with, launch_game_runtime_with,
};
#[cfg(target_os = "linux")]
use crate::linux::launch_strategy::LinuxSteamLaunchStep;
#[cfg(target_os = "linux")]
use std::cell::{Cell, RefCell};
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use uuid::Uuid;

#[cfg(target_os = "linux")]
fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_game_launch_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[cfg(target_os = "linux")]
fn create_exe_layout(root: &PathBuf) {
    let binaries = root.join("Binaries");
    fs::create_dir_all(&binaries).expect("failed to create binaries dir");
    fs::write(binaries.join("NMS.exe"), b"test").expect("failed to create NMS.exe");
}

#[cfg(target_os = "linux")]
fn standard_plan() -> Vec<LinuxSteamLaunchStep> {
    vec![
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
    ]
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_with_routes_steam_and_direct_paths() {
    let steam_flags = RefCell::new(Vec::<bool>::new());
    let direct_paths = RefCell::new(Vec::<String>::new());

    let steam_result = launch_game_with(
        "Steam",
        "/games/nms",
        true,
        |flatpak| {
            steam_flags.borrow_mut().push(flatpak);
            Ok("steam".to_string())
        },
        |path| {
            direct_paths.borrow_mut().push(path.to_string());
            Ok("direct".to_string())
        },
    )
    .expect("steam branch should succeed");
    assert_eq!(steam_result, "steam");
    assert_eq!(&*steam_flags.borrow(), &[true]);
    assert!(direct_paths.borrow().is_empty());

    let direct_result = launch_game_with(
        "GOG",
        "/games/nms",
        false,
        |_| -> Result<String, String> { panic!("steam branch should not be called") },
        |path| {
            direct_paths.borrow_mut().push(path.to_string());
            Ok("direct".to_string())
        },
    )
    .expect("direct branch should succeed");
    assert_eq!(direct_result, "direct");
    assert_eq!(&*direct_paths.borrow(), &["/games/nms".to_string()]);
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_with_propagates_steam_error_and_skips_direct_branch() {
    let direct_called = Cell::new(false);

    let err = launch_game_with(
        "Steam",
        "/games/nms",
        false,
        |_| Err("steam failed".to_string()),
        |_| {
            direct_called.set(true);
            Ok("direct".to_string())
        },
    )
    .expect_err("steam error should propagate");

    assert_eq!(err, "steam failed");
    assert!(!direct_called.get());
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_command_with_builds_and_executes_linux_plan_for_steam() {
    let built_flags = RefCell::new(Vec::<bool>::new());
    let executed_flags = RefCell::new(Vec::<bool>::new());
    let observed_plans = RefCell::new(Vec::<Vec<LinuxSteamLaunchStep>>::new());
    let expected_plan = standard_plan();

    let out = launch_game_command_with(
        "Steam",
        "/games/nms",
        true,
        |flatpak| {
            built_flags.borrow_mut().push(flatpak);
            expected_plan.clone()
        },
        |flatpak, plan| {
            executed_flags.borrow_mut().push(flatpak);
            observed_plans.borrow_mut().push(plan);
            Ok("steam -applaunch 275850".to_string())
        },
        |_| -> Result<String, String> { panic!("direct branch should not be called") },
    )
    .expect("steam command branch should succeed");

    assert_eq!(out, "steam -applaunch 275850");
    assert_eq!(&*built_flags.borrow(), &[true]);
    assert_eq!(&*executed_flags.borrow(), &[true]);
    assert_eq!(&*observed_plans.borrow(), &[expected_plan]);
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_command_with_uses_direct_for_non_steam_versions() {
    let direct_paths = RefCell::new(Vec::<String>::new());

    let out = launch_game_command_with(
        "GOG",
        "/games/nms",
        false,
        |_| standard_plan(),
        |_, _| -> Result<String, String> { panic!("steam branch should not be called") },
        |path| {
            direct_paths.borrow_mut().push(path.to_string());
            Ok("direct".to_string())
        },
    )
    .expect("direct branch should succeed");

    assert_eq!(out, "direct");
    assert_eq!(&*direct_paths.borrow(), &["/games/nms".to_string()]);
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_runtime_with_covers_direct_and_steam_paths() {
    let direct_root = temp_test_dir("runtime_direct");
    create_exe_layout(&direct_root);

    let opened_paths = RefCell::new(Vec::<String>::new());
    let log_entries = RefCell::new(Vec::<(String, String)>::new());
    let direct_result = launch_game_runtime_with(
        "GOG",
        direct_root.to_string_lossy().as_ref(),
        false,
        |_| standard_plan(),
        |_, _| -> Result<String, String> { panic!("steam branch should not be called") },
        |path| {
            opened_paths.borrow_mut().push(path.to_string());
            Ok(())
        },
        |level, message| {
            log_entries
                .borrow_mut()
                .push((level.to_string(), message.to_string()));
        },
    )
    .expect("direct runtime branch should succeed");

    assert_eq!(direct_result, "direct NMS.exe");
    assert_eq!(opened_paths.borrow().len(), 1);
    assert!(
        opened_paths.borrow()[0].ends_with("Binaries/NMS.exe"),
        "expected direct open path to point at NMS.exe"
    );
    assert_eq!(
        log_entries.borrow()[0],
        (
            "INFO".to_string(),
            "Game launch succeeded using direct NMS.exe".to_string()
        )
    );

    let steam_flags = RefCell::new(Vec::<bool>::new());
    let steam_plans = RefCell::new(Vec::<Vec<LinuxSteamLaunchStep>>::new());
    let steam_result = launch_game_runtime_with(
        "Steam",
        "/games/nms",
        true,
        |flatpak| {
            steam_flags.borrow_mut().push(flatpak);
            standard_plan()
        },
        |flatpak, plan| {
            steam_flags.borrow_mut().push(flatpak);
            steam_plans.borrow_mut().push(plan);
            Ok("steam://run/275850".to_string())
        },
        |_| -> Result<(), String> { panic!("direct open should not be called") },
        |_, _| {},
    )
    .expect("steam runtime branch should succeed");

    assert_eq!(steam_result, "steam://run/275850");
    assert_eq!(&*steam_flags.borrow(), &[true, true]);
    assert_eq!(&*steam_plans.borrow(), &[standard_plan()]);

    fs::remove_dir_all(direct_root).expect("cleanup should succeed");
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_runtime_with_propagates_errors() {
    let direct_root = temp_test_dir("runtime_error");
    create_exe_layout(&direct_root);

    let direct_err = launch_game_runtime_with(
        "GOG",
        direct_root.to_string_lossy().as_ref(),
        false,
        |_| standard_plan(),
        |_, _| Ok("unused".to_string()),
        |_| Err("open failed".to_string()),
        |_, _| {},
    )
    .expect_err("direct open error should propagate");
    assert_eq!(direct_err, "open failed");

    let steam_err = launch_game_runtime_with(
        "Steam",
        "/games/nms",
        false,
        |_| standard_plan(),
        |_, _| Err("steam failed".to_string()),
        |_| Ok(()),
        |_, _| {},
    )
    .expect_err("steam executor error should propagate");
    assert_eq!(steam_err, "steam failed");

    fs::remove_dir_all(direct_root).expect("cleanup should succeed");
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_command_entry_with_routes_direct_and_steam_paths() {
    let direct_root = temp_test_dir("entry");
    create_exe_layout(&direct_root);

    let opened_paths = RefCell::new(Vec::<String>::new());
    let direct_out = launch_game_command_entry_with(
        "GOG".to_string(),
        direct_root.to_string_lossy().to_string(),
        false,
        |_| standard_plan(),
        |_, _| -> Result<String, String> { panic!("steam branch should not be called") },
        |path| {
            opened_paths.borrow_mut().push(path.to_string());
            Ok(())
        },
        |_, _| {},
    )
    .expect("direct entry branch should succeed");
    assert_eq!(direct_out, "direct NMS.exe");
    assert_eq!(opened_paths.borrow().len(), 1);

    let steam_out = launch_game_command_entry_with(
        "Steam".to_string(),
        "/games/nms".to_string(),
        false,
        |_| standard_plan(),
        |_, plan| {
            assert_eq!(plan, standard_plan());
            Ok("steam -applaunch 275850".to_string())
        },
        |_| -> Result<(), String> { panic!("direct open should not be called") },
        |_, _| {},
    )
    .expect("steam entry branch should succeed");
    assert_eq!(steam_out, "steam -applaunch 275850");

    fs::remove_dir_all(direct_root).expect("cleanup should succeed");
}

#[cfg(target_os = "linux")]
#[test]
fn launch_game_command_entry_with_propagates_steam_and_direct_errors() {
    let steam_err = launch_game_command_entry_with(
        "Steam".to_string(),
        "/games/nms".to_string(),
        true,
        |_| standard_plan(),
        |_, _| Err("steam failed".to_string()),
        |_| Ok(()),
        |_, _| {},
    )
    .expect_err("steam error should propagate");
    assert_eq!(steam_err, "steam failed");

    let direct_root = temp_test_dir("entry_error");
    let direct_err = launch_game_command_entry_with(
        "GOG".to_string(),
        direct_root.to_string_lossy().to_string(),
        false,
        |_| standard_plan(),
        |_, _| Ok("unused".to_string()),
        |_| Ok(()),
        |_, _| {},
    )
    .expect_err("missing exe should propagate");
    assert_eq!(direct_err, "Could not find NMS.exe in Binaries folder.");

    fs::remove_dir_all(direct_root).expect("cleanup should succeed");
}
