use super::{
    app_commands as tauri_app_commands, game_launch, install, mods, nexus, profiles, startup,
    storage,
};
use crate::app::entry::{
    apply_linux_backend_config_with, handle_window_event_with,
    restore_focus_if_window_available_with, run_single_instance_event_with, StartupWindowEventKind,
};
use crate::app::linux as app_linux;
use crate::app::single_instance;
use crate::linux;
use crate::models::StartupState;
use crate::startup::logic;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

fn configure_linux_environment_process() {
    let (
        has_webkit_disable_dmabuf_renderer,
        has_libgl_always_software,
        has_webkit_disable_compositing_mode,
        has_egl_platform,
        has_gdk_backend,
    ) = (
        std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_ok(),
        std::env::var("LIBGL_ALWAYS_SOFTWARE").is_ok(),
        std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_ok(),
        std::env::var("EGL_PLATFORM").is_ok(),
        std::env::var("GDK_BACKEND").is_ok(),
    );

    app_linux::configure_linux_environment_with(
        app_linux::is_running_on_steam_deck(),
        linux::env::is_flatpak_runtime(|k| std::env::var(k).ok()),
        has_webkit_disable_dmabuf_renderer,
        has_libgl_always_software,
        has_webkit_disable_compositing_mode,
        has_egl_platform,
        has_gdk_backend,
        &mut |k, v| unsafe { std::env::set_var(k, v) },
        &mut |msg| println!("{}", msg),
    );
}

pub(crate) fn run_app() {
    #[cfg(target_os = "linux")]
    {
        let is_flatpak = linux::env::is_flatpak_runtime(|k| std::env::var(k).ok());
        apply_linux_backend_config_with(
            is_flatpak,
            std::env::var("GDK_BACKEND").is_ok(),
            &mut |k, v| unsafe { std::env::set_var(k, v) },
            &mut |msg| println!("{}", msg),
        );
    }

    configure_linux_environment_process();

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(StartupState {
            pending_nxm: Mutex::new(None),
        })
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            run_single_instance_event_with(
                &argv,
                &logic::find_nxm_argument,
                &mut |nxm_link| {
                    let _ = app.emit("nxm-link-received", nxm_link);
                },
                &mut || {
                    restore_focus_if_window_available_with(
                        app.get_webview_window("main").is_some(),
                        &mut || {
                            let window = app.get_webview_window("main");
                            let Some(window) = window else {
                                return Ok(());
                            };
                            single_instance::restore_focus_for_main_window_with(
                                &mut || window.unminimize().map_err(|e| e.to_string()),
                                &mut || window.set_focus().map_err(|e| e.to_string()),
                            )
                        },
                    )
                },
                &mut |message| println!("{}", message),
                &mut |message| eprintln!("{}", message),
            );
        }))
        .setup(startup::run_startup_setup)
        .on_window_event(|window, event| {
            let kind = match event {
                tauri::WindowEvent::Resized(_) => StartupWindowEventKind::Resized,
                tauri::WindowEvent::Moved(_) => StartupWindowEventKind::Moved,
                tauri::WindowEvent::CloseRequested { .. } => StartupWindowEventKind::CloseRequested,
                _ => StartupWindowEventKind::Other,
            };
            handle_window_event_with(kind, || startup::persist_window_state_on_event(window));
        })
        .invoke_handler(tauri::generate_handler![
            tauri_app_commands::check_startup_intent,
            tauri_app_commands::detect_game_installation,
            tauri_app_commands::open_mods_folder,
            tauri_app_commands::save_file,
            tauri_app_commands::delete_settings_file,
            mods::reorder_mods,
            install::install_mod_from_archive,
            install::resolve_conflict,
            tauri_app_commands::resize_window,
            mods::delete_mod,
            mods::update_mod_name_in_xml,
            mods::update_mod_id_in_json,
            mods::ensure_mod_info,
            nexus::get_nexus_api_key,
            nexus::register_nxm_protocol,
            nexus::unregister_nxm_protocol,
            nexus::is_protocol_handler_registered,
            install::get_all_mods_for_render,
            mods::download_mod_archive,
            storage::show_in_folder,
            storage::delete_archive_file,
            storage::clear_downloads_folder,
            game_launch::launch_game,
            profiles::list_profiles,
            profiles::save_active_profile,
            profiles::apply_profile,
            profiles::delete_profile,
            profiles::rename_profile,
            profiles::create_empty_profile,
            tauri_app_commands::check_for_untracked_mods,
            profiles::get_profile_mod_list,
            profiles::copy_profile,
            nexus::login_to_nexus,
            nexus::logout_nexus,
            storage::get_downloads_path,
            storage::set_downloads_path,
            storage::open_special_folder,
            storage::open_folder_path,
            storage::clean_staging_folder,
            install::finalize_installation,
            storage::get_staging_contents,
            tauri_app_commands::run_legacy_migration,
            tauri_app_commands::write_to_log,
            storage::set_library_path,
            storage::get_library_path,
            storage::delete_library_folder,
            storage::check_library_existence,
            mods::rename_mod_folder,
            tauri_app_commands::is_app_installed,
            tauri_app_commands::http_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
