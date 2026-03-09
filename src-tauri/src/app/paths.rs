use std::fs;
use std::path::{Path, PathBuf};

type PathProviderFn<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type LoadCustomFn<'a> = dyn Fn(&PathBuf) -> Option<String> + 'a;

fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn storage_dir_from_custom_or_default(
    custom: Option<String>,
    default_root: &Path,
    default_leaf: &str,
) -> PathBuf {
    custom
        .map(PathBuf::from)
        .unwrap_or_else(|| default_root.join(default_leaf))
}

pub(crate) fn app_data_file_path_with(
    resolve_app_data_dir: &PathProviderFn<'_>,
    file_name: &str,
) -> Result<PathBuf, String> {
    let app_data = resolve_app_data_dir()?;
    ensure_dir(&app_data)?;
    Ok(app_data.join(file_name))
}

pub(crate) fn storage_dir_with(
    custom: Option<String>,
    resolve_default_root: &PathProviderFn<'_>,
    default_leaf: &str,
) -> Result<PathBuf, String> {
    let root = resolve_default_root()?;
    let dir = storage_dir_from_custom_or_default(custom, &root, default_leaf);
    ensure_dir(&dir)?;
    Ok(dir)
}

pub(crate) fn get_pulsar_root_with(
    resolve_pulsar_root: &PathProviderFn<'_>,
) -> Result<PathBuf, String> {
    let path = resolve_pulsar_root()?;
    ensure_dir(&path)?;
    Ok(path)
}

pub(crate) fn resolve_custom_path_with(
    get_config_file_path: &PathProviderFn<'_>,
    load_custom: &LoadCustomFn<'_>,
) -> Option<String> {
    get_config_file_path()
        .ok()
        .and_then(|config_path| load_custom(&config_path))
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
