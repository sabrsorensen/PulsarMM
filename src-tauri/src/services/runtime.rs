use std::path::PathBuf;

pub trait AppRuntime {
    fn find_game_path(&self) -> Option<PathBuf>;
    fn get_library_dir(&self) -> Result<PathBuf, String>;
    fn get_pulsar_root(&self) -> Result<PathBuf, String>;
    fn log(&self, level: &str, message: &str);
}

pub(crate) fn runtime_find_game_path_with(
    find_game_path: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    find_game_path()
}

pub(crate) fn runtime_get_library_dir_with(
    get_library_dir: impl FnOnce() -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    get_library_dir()
}

pub(crate) fn runtime_get_pulsar_root_with(
    get_pulsar_root: impl FnOnce() -> Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    get_pulsar_root()
}

pub(crate) fn runtime_log_with(log: impl FnOnce(&str, &str), level: &str, message: &str) {
    log(level, message);
}

#[cfg(test)]
pub fn snapshot_runtime_paths(
    runtime: &impl AppRuntime,
) -> Result<(Option<PathBuf>, PathBuf, PathBuf), String> {
    Ok((
        runtime.find_game_path(),
        runtime.get_library_dir()?,
        runtime.get_pulsar_root()?,
    ))
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
