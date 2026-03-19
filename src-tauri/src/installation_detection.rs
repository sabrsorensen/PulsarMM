use crate::models::GamePaths;
use crate::settings_paths;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

static MANUAL_GAME_PATH: LazyLock<Mutex<Option<PathBuf>>> = LazyLock::new(|| Mutex::new(None));

#[cfg(target_os = "windows")]
fn determine_version_type(path: &Path) -> &'static str {
    let path_text = path.to_string_lossy();
    if path_text.contains("Xbox") {
        "GamePass"
    } else if path_text.contains("GOG") {
        "GOG"
    } else {
        "Steam"
    }
}

#[cfg(target_os = "linux")]
fn determine_version_type(_path: &Path) -> &'static str {
    "Steam"
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn determine_version_type(_path: &Path) -> &'static str {
    "Steam"
}

pub fn detect_game_paths(path: &Path) -> Option<GamePaths> {
    let binaries_dir = settings_paths::binaries_dir(path);
    if !binaries_dir.exists() {
        return None;
    }

    let settings_initialized = settings_paths::mod_settings_file(path).exists();
    let root = path.to_string_lossy().into_owned();
    Some(GamePaths {
        game_root_path: root.clone(),
        settings_root_path: root,
        version_type: determine_version_type(path).to_string(),
        settings_initialized,
    })
}

fn has_binaries_dir(path: &Path) -> bool {
    settings_paths::binaries_dir(path).is_dir()
}

pub fn resolve_game_root_from_selection(path: &Path) -> Option<PathBuf> {
    if has_binaries_dir(path) {
        return Some(path.to_path_buf());
    }

    let settings_file = path.file_name()?.to_str()?;
    if settings_file.eq_ignore_ascii_case(settings_paths::MOD_SETTINGS_FILE) {
        let settings_dir = path.parent()?;
        let binaries_dir = settings_dir.parent()?;
        if settings_dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("SETTINGS"))
            && binaries_dir
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("Binaries"))
        {
            let root = binaries_dir.parent()?;
            if has_binaries_dir(root) {
                return Some(root.to_path_buf());
            }
        }
    }

    let tail = path.file_name()?.to_str()?;
    let parent = path.parent()?;
    let parent_name = parent.file_name().and_then(|name| name.to_str())?;

    if tail.eq_ignore_ascii_case("SETTINGS") && parent_name.eq_ignore_ascii_case("Binaries") {
        let root = parent.parent()?;
        if has_binaries_dir(root) {
            return Some(root.to_path_buf());
        }
    }

    if tail.eq_ignore_ascii_case("MODS") && parent_name.eq_ignore_ascii_case("GAMEDATA") {
        let root = parent.parent()?;
        if has_binaries_dir(root) {
            return Some(root.to_path_buf());
        }
    }

    None
}

pub fn set_manual_game_path(path: Option<PathBuf>) {
    let mut guard = match MANUAL_GAME_PATH.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = path.filter(|candidate| has_binaries_dir(candidate));
}

pub fn get_manual_game_path() -> Option<PathBuf> {
    let guard = match MANUAL_GAME_PATH.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.clone().filter(|path| has_binaries_dir(path))
}

pub fn find_game_path() -> Option<PathBuf> {
    if let Some(path) = get_manual_game_path() {
        return Some(path);
    }

    #[cfg(target_os = "windows")]
    {
        return find_windows_game_path();
    }

    #[cfg(target_os = "linux")]
    {
        return crate::linux::runtime::find_linux_game_path();
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    return None;
}

#[cfg(target_os = "windows")]
fn find_windows_game_path() -> Option<PathBuf> {
    find_steam_path()
        .or_else(find_gog_path)
        .or_else(find_gamepass_path)
}

#[cfg(any(test, target_os = "windows"))]
fn parse_steam_library_folders(content: &str) -> Vec<PathBuf> {
    content
        .lines()
        .filter_map(|line| {
            let quoted: Vec<&str> = line.split('"').skip(1).step_by(2).collect();
            if quoted.len() >= 2 && quoted[0].eq_ignore_ascii_case("path") {
                return Some(quoted[1]);
            }
            if quoted.len() >= 3 && quoted[1].eq_ignore_ascii_case("path") {
                return Some(quoted[2]);
            }
            None
        })
        .map(|path| PathBuf::from(path.replace("\\\\", "\\")))
        .collect()
}

#[cfg(any(test, target_os = "windows"))]
fn parse_steam_installdir(content: &str) -> Option<String> {
    content
        .lines()
        .find(|line| line.contains("\"installdir\""))
        .and_then(|line| line.split('"').nth(3))
        .map(|name| name.to_string())
}

#[cfg(target_os = "windows")]
fn find_gog_path() -> Option<PathBuf> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let known_ids = ["1446213994", "1446223351"];

    for id in known_ids {
        let key_path = format!(r"SOFTWARE\WOW6432Node\GOG.com\Games\{}", id);

        let Ok(gog_key) = hklm.open_subkey(&key_path) else {
            continue;
        };
        let Ok(game_path_str) = gog_key.get_value::<String, _>("PATH") else {
            continue;
        };
        let game_path = PathBuf::from(game_path_str);
        if has_binaries_dir(&game_path) {
            return Some(game_path);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_steam_path() -> Option<PathBuf> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let steam_key = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam").ok()?;
    let steam_path_str = steam_key.get_value::<String, _>("InstallPath").ok()?;
    let steam_path = PathBuf::from(steam_path_str);
    let mut library_folders = vec![steam_path.clone()];

    let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        for path in parse_steam_library_folders(&content) {
            if path.exists() {
                library_folders.push(path);
            }
        }
    }

    for folder in library_folders {
        let manifest_path = folder.join("steamapps").join("appmanifest_275850.acf");
        let Ok(content) = std::fs::read_to_string(manifest_path) else {
            continue;
        };
        let Some(installdir) = parse_steam_installdir(&content) else {
            continue;
        };

        let game_path = folder.join("steamapps").join("common").join(installdir);
        if game_path.is_dir() {
            return Some(game_path);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_gamepass_path() -> Option<PathBuf> {
    use std::process::Command;

    let default_path = PathBuf::from("C:\\XboxGames\\No Man's Sky\\Content");
    if has_binaries_dir(&default_path) {
        return Some(default_path);
    }

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-AppxPackage -Name 'HelloGames.NoMansSky' | Select-Object -ExpandProperty InstallLocation",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path_str = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if path_str.is_empty() {
        return None;
    }

    let game_path = PathBuf::from(path_str).join("Content");
    if has_binaries_dir(&game_path) {
        Some(game_path)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "installation_detection_tests.rs"]
mod tests;
