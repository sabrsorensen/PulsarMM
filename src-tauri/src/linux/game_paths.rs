use crate::linux::paths as linux_paths;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn has_binaries_dir(game_path: &Path, is_dir: &dyn Fn(&Path) -> bool) -> bool {
    is_dir(&game_path.join("Binaries"))
}

fn linux_manual_game_path_with(
    manual_path: Option<String>,
    is_dir: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let manual = PathBuf::from(manual_path?);
    if has_binaries_dir(&manual, is_dir) {
        Some(manual)
    } else {
        None
    }
}

#[cfg(test)]
pub fn linux_manual_game_path<FIsDir>(
    manual_path: Option<String>,
    is_dir: FIsDir,
) -> Option<PathBuf>
where
    FIsDir: Fn(&Path) -> bool,
{
    linux_manual_game_path_with(manual_path, &is_dir)
}

pub fn linux_steam_roots(home_path: &Path) -> Vec<PathBuf> {
    vec![
        home_path.join(".steam/steam"),
        home_path.join(".local/share/Steam"),
        home_path.join(".var/app/com.valvesoftware.Steam/data/Steam"),
    ]
}

pub fn linux_fallback_game_paths(home_path: &Path) -> Vec<PathBuf> {
    vec![
        home_path.join(".steam/steam/steamapps/common/No Man's Sky"),
        home_path.join(".local/share/Steam/steamapps/common/No Man's Sky"),
        home_path.join(".var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/No Man's Sky"),
    ]
}

fn find_linux_steam_game_path_with(
    home_path: &Path,
    read_to_string: &dyn Fn(&Path) -> Option<String>,
    is_dir: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let steam_roots = linux_steam_roots(home_path);
    let mut library_folders: Vec<PathBuf> = Vec::new();

    for root in &steam_roots {
        if !is_dir(root) {
            continue;
        }

        library_folders.push(root.clone());
        let vdf_path = root.join("steamapps").join("libraryfolders.vdf");
        if let Some(content) = read_to_string(&vdf_path) {
            library_folders.extend(linux_paths::parse_steam_library_folders(&content));
        }
    }

    let mut seen = HashSet::new();
    library_folders.retain(|p| seen.insert(p.clone()));

    for folder in library_folders {
        let manifest_path = folder.join("steamapps").join("appmanifest_275850.acf");
        let Some(content) = read_to_string(&manifest_path) else {
            continue;
        };
        let Some(installdir) = linux_paths::extract_installdir_from_manifest(&content) else {
            continue;
        };

        let game_path = folder.join("steamapps").join("common").join(installdir);
        if has_binaries_dir(&game_path, is_dir) {
            return Some(game_path);
        }
    }

    linux_fallback_game_paths(home_path)
        .into_iter()
        .find(|p| has_binaries_dir(p, is_dir))
}

#[cfg(test)]
pub fn find_linux_steam_game_path<FReadToString, FIsDir>(
    home_path: &Path,
    read_to_string: FReadToString,
    is_dir: FIsDir,
) -> Option<PathBuf>
where
    FReadToString: Fn(&Path) -> Option<String>,
    FIsDir: Fn(&Path) -> bool,
{
    find_linux_steam_game_path_with(home_path, &read_to_string, &is_dir)
}

fn find_linux_game_path_with_impl(
    get_var: &dyn Fn(&str) -> Option<String>,
    read_to_string: &dyn Fn(&Path) -> Option<String>,
    is_dir: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let manual_path = get_var("PULSAR_NMS_PATH");
    if let Some(path) = linux_manual_game_path_with(manual_path, is_dir) {
        return Some(path);
    }

    let home = get_var("HOME")?;
    find_linux_steam_game_path_with(Path::new(&home), read_to_string, is_dir)
}

pub fn find_linux_game_path_with<FGetVar, FReadToString, FIsDir>(
    get_var: FGetVar,
    read_to_string: FReadToString,
    is_dir: FIsDir,
) -> Option<PathBuf>
where
    FGetVar: Fn(&str) -> Option<String>,
    FReadToString: Fn(&Path) -> Option<String>,
    FIsDir: Fn(&Path) -> bool,
{
    find_linux_game_path_with_impl(&get_var, &read_to_string, &is_dir)
}

#[cfg(test)]
#[path = "game_paths_tests.rs"]
mod tests;
