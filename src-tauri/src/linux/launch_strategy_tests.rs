use super::*;

#[test]
fn flatpak_runtime_detects_either_env_flag() {
    assert!(is_flatpak_runtime_with(|key| {
        (key == "FLATPAK_ID").then(|| "com.example.App".to_string())
    }));
    assert!(is_flatpak_runtime_with(|key| {
        (key == "PULSAR_FLATPAK").then(|| "1".to_string())
    }));
    assert!(!is_flatpak_runtime_with(|_| None));
}

#[test]
fn flatpak_plan_has_host_bridge_steps_only() {
    let plan = linux_steam_launch_plan(true);
    assert_eq!(plan.len(), 2);
    assert!(matches!(plan[0], LinuxSteamLaunchStep::Command { .. }));
    assert!(matches!(plan[1], LinuxSteamLaunchStep::Command { .. }));
}

#[test]
fn native_plan_includes_open_url_fallback() {
    let plan = linux_steam_launch_plan(false);
    assert_eq!(plan.len(), 3);
    assert!(matches!(plan[2], LinuxSteamLaunchStep::OpenUrl(_)));
}

#[test]
fn step_label_formats_command() {
    let step = LinuxSteamLaunchStep::Command {
        program: "steam".to_string(),
        args: vec!["-applaunch".to_string(), "275850".to_string()],
    };
    assert_eq!(step_label(&step), "steam -applaunch 275850");
}

#[test]
fn step_label_formats_command_without_args() {
    let step = LinuxSteamLaunchStep::Command {
        program: "steam".to_string(),
        args: vec![],
    };
    assert_eq!(step_label(&step), "steam");
}

#[test]
fn step_label_formats_open_url() {
    let step = LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string());
    assert_eq!(step_label(&step), "open::that steam://run/275850");
}

#[test]
fn failure_message_flatpak_prefix() {
    let msg = steam_launch_failure_message(
        true,
        &["flatpak-spawn --host steam -applaunch 275850".to_string()],
        &["spawn failed".to_string()],
    );
    assert!(msg.contains("Flatpak host bridge"));
    assert!(msg.contains("spawn failed"));
}

#[test]
fn failure_message_native_prefix() {
    let msg = steam_launch_failure_message(
        false,
        &["steam -applaunch 275850".to_string()],
        &["not found".to_string()],
    );
    assert!(msg.contains("via Steam"));
    assert!(msg.contains("not found"));
}
