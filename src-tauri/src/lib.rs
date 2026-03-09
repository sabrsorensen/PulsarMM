mod adapters;
mod app;
mod fs_ops;
mod game_launch;
mod game_launch_ops;
pub mod installation_detection;
pub mod linux;
mod logging;
pub mod models;
pub mod mods;
pub mod nexus;
mod path_ops;
pub mod profiles;
mod services;
pub mod settings_paths;
mod staging_contents;
mod startup;
mod storage;
mod utils;
mod xml_format;

pub(crate) use crate::mods::info_ops::read_mod_info_file as read_mod_info;
pub(crate) use adapters::tauri::logging::{log_internal, rotate_logs};
pub(crate) use adapters::tauri::paths::{
    get_config_file_path, get_downloads_dir, get_library_dir, get_pulsar_root, get_state_file_path,
};
pub(crate) use app::icon::load_runtime_window_icon;

pub fn run_app() {
    adapters::tauri::app::run_app();
}
