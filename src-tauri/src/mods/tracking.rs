use serde_json::Value;
use std::fs;
use std::path::Path;

fn is_untracked_mod_dir(mod_dir: &Path) -> bool {
    let info_path = mod_dir.join("mod_info.json");
    if !info_path.exists() {
        return true;
    }

    let Ok(content) = fs::read_to_string(&info_path) else {
        return true;
    };
    let Ok(json) = serde_json::from_str::<Value>(&content) else {
        return true;
    };

    !json
        .get("installSource")
        .and_then(|value| value.as_str())
        .map(|source| !source.is_empty())
        .unwrap_or(false)
}

pub fn has_untracked_mods_in_dir(mods_path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(mods_path) else {
        return false;
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .any(|path| is_untracked_mod_dir(&path))
}

#[cfg(test)]
#[path = "tracking_tests.rs"]
mod tests;
