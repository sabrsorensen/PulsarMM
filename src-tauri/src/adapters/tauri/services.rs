use crate::installation_detection::find_game_path;
use crate::services::runtime::{
    runtime_find_game_path_with, runtime_get_library_dir_with, runtime_get_pulsar_root_with,
    runtime_log_with, AppRuntime,
};
use crate::{get_library_dir, get_pulsar_root, log_internal};
use std::path::PathBuf;
use tauri::AppHandle;

pub struct TauriRuntime<'a> {
    app: &'a AppHandle,
}

impl<'a> TauriRuntime<'a> {
    pub fn new(app: &'a AppHandle) -> Self {
        Self { app }
    }
}

impl AppRuntime for TauriRuntime<'_> {
    fn find_game_path(&self) -> Option<PathBuf> {
        runtime_find_game_path_with(find_game_path)
    }

    fn get_library_dir(&self) -> Result<PathBuf, String> {
        runtime_get_library_dir_with(|| get_library_dir(self.app))
    }

    fn get_pulsar_root(&self) -> Result<PathBuf, String> {
        runtime_get_pulsar_root_with(|| get_pulsar_root(self.app))
    }

    fn log(&self, level: &str, message: &str) {
        runtime_log_with(
            |level, message| log_internal(self.app, level, message),
            level,
            message,
        );
    }
}
