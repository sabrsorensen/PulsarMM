use crate::app::linux;
use crate::installation_detection::set_manual_game_path;
use crate::models::StartupState;
use crate::startup::logic::find_nxm_argument;
use crate::startup::runtime::{
    apply_runtime_window_icon_with, apply_steam_deck_window_decorations_with,
    configure_main_window_stage_with, expand_fs_scope_with, maybe_show_main_window_with,
    persist_window_state_action_with, restore_window_state_from_snapshot_with,
    restore_window_state_with,
};
use crate::startup::{
    cache_pending_nxm_with, configure_main_window_if_present_with,
    persist_window_state_on_event_with, run_startup_setup_with, window_event_snapshot_with,
};
use crate::utils::config::load_config_or_default;
use crate::{
    get_config_file_path, get_state_file_path, load_runtime_window_icon, log_internal,
};
use tauri::{App, AppHandle, Manager, PhysicalPosition};
use tauri_plugin_fs::FsExt;

pub(crate) fn expand_fs_scope(app_handle: &AppHandle) {
    let mut allow_directory = |path: &std::path::PathBuf| {
        app_handle
            .fs_scope()
            .allow_directory(path, true)
            .map_err(|e| e.to_string())
    };
    let app_data = app_handle.path().app_data_dir().ok();
    let data_dir = app_handle
        .path()
        .resolve("Pulsar", tauri::path::BaseDirectory::Data)
        .ok();
    expand_fs_scope_with(app_data, data_dir, &mut allow_directory);

    if let Ok(config_path) = get_config_file_path(app_handle) {
        let config = load_config_or_default(&config_path, true);
        if let Some(game_path) = config.custom_game_path {
            let game_path = std::path::PathBuf::from(game_path);
            set_manual_game_path(Some(game_path.clone()));
            let _ = app_handle.fs_scope().allow_directory(&game_path, true);
        }
    }
}

pub(crate) fn apply_runtime_window_icon(app_handle: &AppHandle, window: &tauri::WebviewWindow) {
    let mut set_icon = |icon| window.set_icon(icon).map_err(|e| e.to_string());
    let mut log = |level: &str, message: &str| log_internal(app_handle, level, message);
    apply_runtime_window_icon_with(
        load_runtime_window_icon(),
        app_handle.default_window_icon().cloned(),
        &mut set_icon,
        &mut log,
    );
}

pub(crate) fn apply_steam_deck_window_config(
    app_handle: &AppHandle,
    window: &tauri::WebviewWindow,
) {
    let mut set_decorations = |decorations| {
        window
            .set_decorations(decorations)
            .map_err(|e| e.to_string())
    };
    let mut log = |level: &str, message: &str| log_internal(app_handle, level, message);
    apply_steam_deck_window_decorations_with(
        linux::is_running_on_steam_deck(),
        &mut log,
        &mut set_decorations,
    );
}

pub(crate) fn restore_window_state(app_handle: &AppHandle, window: &tauri::WebviewWindow) {
    let mut restore = |state| {
        let mut set_position = |x, y| {
            window
                .set_position(PhysicalPosition::new(x, y))
                .map_err(|e| e.to_string())
        };
        let mut maximize = || window.maximize().map_err(|e| e.to_string());
        let mut log = |level: &str, message: &str| log_internal(app_handle, level, message);
        restore_window_state_from_snapshot_with(state, &mut log, &mut set_position, &mut maximize);
    };
    restore_window_state_with(
        get_state_file_path(app_handle),
        crate::utils::window_state::load_window_state,
        &mut restore,
    );
}

pub fn run_startup_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let args: Vec<String> = std::env::args().collect();
    let mut main_window = app.get_webview_window("main");
    let main_window_exists = main_window.is_some();

    run_startup_setup_with(
        &args,
        main_window_exists,
        || crate::rotate_logs(app_handle),
        |level, message| log_internal(app_handle, level, message),
        || expand_fs_scope(app_handle),
        find_nxm_argument,
        |nxm_link| {
            let state = app.try_state::<StartupState>();
            cache_pending_nxm_with(state.as_deref(), nxm_link);
        },
        || {
            let configure_window = |window: tauri::WebviewWindow| {
                let mut apply_icon = || apply_runtime_window_icon(app_handle, &window);
                let mut apply_steam_deck = || apply_steam_deck_window_config(app_handle, &window);
                let mut restore_state = || restore_window_state(app_handle, &window);
                let mut show_window = || {
                    let mut show = || window.show().map_err(|e| e.to_string());
                    let mut log =
                        |level: &str, message: &str| log_internal(app_handle, level, message);
                    maybe_show_main_window_with(&mut show, &mut log)
                };
                configure_main_window_stage_with(
                    &mut apply_icon,
                    &mut apply_steam_deck,
                    &mut restore_state,
                    &mut show_window,
                );
            };
            configure_main_window_if_present_with(main_window.take(), |window| {
                configure_window(window);
            });
        },
    );
    Ok(())
}

pub fn persist_window_state_on_event(window: &tauri::Window) {
    let (is_minimized, is_maximized, outer_position, state_path) = window_event_snapshot_with(
        || window.is_minimized().map_err(|e| e.to_string()),
        || window.is_maximized().map_err(|e| e.to_string()),
        || {
            window
                .outer_position()
                .map(|p| (p.x, p.y))
                .map_err(|e| e.to_string())
        },
        || get_state_file_path(&window.app_handle()),
    );
    persist_window_state_on_event_with(
        is_minimized,
        is_maximized,
        outer_position,
        state_path,
        |action, state_path| persist_window_state_action_with(action, state_path),
    );
}
