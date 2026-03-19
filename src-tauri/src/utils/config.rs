use crate::models::GlobalAppConfig;
use std::fs;
use std::path::Path;

pub(crate) fn default_config(legacy_migration_done: bool) -> GlobalAppConfig {
    GlobalAppConfig {
        custom_download_path: None,
        custom_library_path: None,
        custom_game_path: None,
        legacy_migration_done,
    }
}

pub(crate) fn parse_config_or_default(
    content: &str,
    legacy_migration_done: bool,
) -> GlobalAppConfig {
    match serde_json::from_str(content) {
        Ok(config) => config,
        Err(_) => default_config(legacy_migration_done),
    }
}

pub(crate) fn load_config_or_default(
    config_path: &Path,
    legacy_migration_done: bool,
) -> GlobalAppConfig {
    if !config_path.exists() {
        return default_config(legacy_migration_done);
    }

    let Ok(content) = fs::read_to_string(config_path) else {
        return default_config(legacy_migration_done);
    };

    parse_config_or_default(&content, legacy_migration_done)
}

pub(crate) fn save_config(config_path: &Path, config: &GlobalAppConfig) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(config).expect("serializing GlobalAppConfig should not fail");

    match fs::write(config_path, json) {
        Ok(()) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
