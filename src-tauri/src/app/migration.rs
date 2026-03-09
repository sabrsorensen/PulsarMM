use crate::mods::command_ops::mods_root_from_game_path;
use crate::utils::config::{load_config_or_default, save_config};
use crate::utils::migration::{build_legacy_lookup, heal_mod_infos_in_dir, load_profiles_from_dir};
use std::path::Path;

pub fn run_legacy_migration_in_paths(
    config_path: &Path,
    profiles_dir: &Path,
    game_path: Option<&Path>,
) -> Result<bool, String> {
    let mut config = load_config_or_default(config_path, false);
    if config.legacy_migration_done {
        return Ok(false);
    }

    let all_profiles = load_profiles_from_dir(profiles_dir);
    let legacy_lookup = build_legacy_lookup(all_profiles);

    if let Some(path) = game_path {
        let mods_path = mods_root_from_game_path(path);
        let _ = heal_mod_infos_in_dir(&mods_path, &legacy_lookup);
    }

    config.legacy_migration_done = true;
    save_config(config_path, &config)?;
    Ok(true)
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod tests;
