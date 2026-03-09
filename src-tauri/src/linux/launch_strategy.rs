#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxSteamLaunchStep {
    Command { program: String, args: Vec<String> },
    OpenUrl(String),
}

pub fn is_flatpak_runtime_with(get_var: impl Fn(&str) -> Option<String>) -> bool {
    get_var("FLATPAK_ID").is_some() || get_var("PULSAR_FLATPAK").is_some()
}

pub fn linux_steam_launch_plan(is_flatpak: bool) -> Vec<LinuxSteamLaunchStep> {
    if is_flatpak {
        return vec![
            LinuxSteamLaunchStep::Command {
                program: "flatpak-spawn".to_string(),
                args: vec![
                    "--host".to_string(),
                    "steam".to_string(),
                    "-applaunch".to_string(),
                    "275850".to_string(),
                ],
            },
            LinuxSteamLaunchStep::Command {
                program: "flatpak-spawn".to_string(),
                args: vec![
                    "--host".to_string(),
                    "xdg-open".to_string(),
                    "steam://run/275850".to_string(),
                ],
            },
        ];
    }

    vec![
        LinuxSteamLaunchStep::Command {
            program: "steam".to_string(),
            args: vec!["-applaunch".to_string(), "275850".to_string()],
        },
        LinuxSteamLaunchStep::Command {
            program: "xdg-open".to_string(),
            args: vec!["steam://run/275850".to_string()],
        },
        LinuxSteamLaunchStep::OpenUrl("steam://run/275850".to_string()),
    ]
}

pub fn step_label(step: &LinuxSteamLaunchStep) -> String {
    match step {
        LinuxSteamLaunchStep::Command { program, args } => {
            if args.is_empty() {
                program.clone()
            } else {
                format!("{} {}", program, args.join(" "))
            }
        }
        LinuxSteamLaunchStep::OpenUrl(url) => format!("open::that {}", url),
    }
}

pub fn steam_launch_failure_message(
    is_flatpak: bool,
    attempts: &[String],
    errors: &[String],
) -> String {
    let prefix = if is_flatpak {
        "Failed to launch No Man's Sky via Flatpak host bridge."
    } else {
        "Failed to launch No Man's Sky via Steam."
    };

    format!(
        "{} Attempts: {}. Errors: {}",
        prefix,
        attempts.join(", "),
        errors.join(" | ")
    )
}

#[cfg(test)]
#[path = "launch_strategy_tests.rs"]
mod tests;
