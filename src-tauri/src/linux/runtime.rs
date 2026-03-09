use crate::linux::game_paths::find_linux_game_path_with;
use crate::linux::launch_strategy::is_flatpak_runtime_with;
use std::path::PathBuf;

pub fn find_linux_game_path() -> Option<PathBuf> {
    find_linux_game_path_with(
        |key| std::env::var(key).ok(),
        |p| std::fs::read_to_string(p).ok(),
        |p| p.is_dir(),
    )
}

pub fn is_flatpak_runtime() -> bool {
    is_flatpak_runtime_with(|key| std::env::var(key).ok())
}
