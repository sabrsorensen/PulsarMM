use std::path::{Path, PathBuf};

pub const MOD_SETTINGS_FILE: &str = "GCMODSETTINGS.MXML";

pub fn binaries_dir(game_root: &Path) -> PathBuf {
    game_root.join("Binaries")
}

pub fn settings_dir(game_root: &Path) -> PathBuf {
    binaries_dir(game_root).join("SETTINGS")
}

pub fn mod_settings_file(game_root: &Path) -> PathBuf {
    settings_dir(game_root).join(MOD_SETTINGS_FILE)
}

#[cfg(test)]
#[path = "settings_paths_tests.rs"]
mod tests;
