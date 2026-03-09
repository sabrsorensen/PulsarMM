use crate::log_internal;
use crate::nexus::auth::handshake_payload;
#[cfg(target_os = "windows")]
use crate::nexus::command_ops::windows_nxm_command;
use crate::nexus::command_ops::{
    linux_protocol_handler_registered, linux_unregister_nxm_protocol_with,
    remove_auth_file_if_exists, save_api_key_to_auth_path,
};
use crate::nexus::commands::{
    await_api_key_from_messages_with, ensure_auth_path_for_app_data, get_auth_file_path_with,
    get_nexus_api_key_command_with, handle_login_text_with, handle_login_ws_message_with,
    is_protocol_handler_registered_command_linux_with, linux_home_from_env,
    logout_nexus_command_with, nexus_ws_url, open_auth_url_with, parse_login_message_for_api_key,
    parse_nexus_ws_url_with, persist_api_key_for_login_with,
    register_nxm_protocol_command_linux_with, send_handshake_with,
    unregister_nxm_protocol_command_linux_with,
};
use crate::utils::auth::load_api_key_from_file;
use futures_util::{SinkExt, StreamExt};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio_tungstenite::connect_async;
use url::Url;
use uuid::Uuid;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

fn parse_nexus_ws_url(url: &str) -> Result<Url, String> {
    Url::parse(url).map_err(|e| format!("Failed to parse WebSocket URL: {}", e))
}

#[cfg(target_os = "linux")]
fn remove_file_at_path(path: &std::path::Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn unregister_nxm_protocol_home(home: &str) -> Result<(), String> {
    linux_unregister_nxm_protocol_with(home, &remove_file_at_path)
}

#[cfg(target_os = "linux")]
fn create_dir_all_string(path: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn write_string_file(path: &std::path::Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn run_xdg_mime_command() -> Result<(), String> {
    use std::process::Command;
    Command::new("xdg-mime")
        .args(["default", "nxm-handler.desktop", "x-scheme-handler/nxm"])
        .output()
        .map_err(|e| format!("Failed to run xdg-mime: {}", e))?;
    Ok(())
}

fn get_auth_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = || app.path().app_data_dir().map_err(|e| e.to_string());
    let ensure_auth_path = |app_data_dir: &std::path::Path| {
        ensure_auth_path_for_app_data(app_data_dir, &|path| {
            fs::create_dir_all(path).map_err(|e| e.to_string())
        })
    };
    get_auth_file_path_with(&app_data_dir, &ensure_auth_path)
}

#[tauri::command]
pub fn get_nexus_api_key(app: AppHandle) -> Result<String, String> {
    get_nexus_api_key_command_with(&|| get_auth_file_path(&app), &load_api_key_from_file)
}

#[tauri::command]
pub fn unregister_nxm_protocol() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.delete_subkey_all("Software\\Classes\\nxm")
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let get_home = || linux_home_from_env(&|k| std::env::var(k).ok());
        unregister_nxm_protocol_command_linux_with(&get_home, &unregister_nxm_protocol_home)?;
    }

    Ok(())
}

#[tauri::command]
pub fn is_protocol_handler_registered() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_path_str) = exe_path.to_str() {
                let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
                if let Ok(command_key) = hkcr.open_subkey("nxm\\shell\\open\\command") {
                    if let Ok(command_val) = command_key.get_value::<String, _>("") {
                        return command_val.contains(exe_path_str);
                    }
                }
            }
        }
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        is_protocol_handler_registered_command_linux_with(
            &|| std::env::var("HOME").ok(),
            &linux_protocol_handler_registered,
        )
    }
}

#[tauri::command]
pub fn register_nxm_protocol() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let command = windows_nxm_command(&exe_path);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (nxm_key, _) = hkcu
            .create_subkey("Software\\Classes\\nxm")
            .map_err(|e| e.to_string())?;

        nxm_key
            .set_value("", &"URL:NXM Protocol")
            .map_err(|e| e.to_string())?;
        nxm_key
            .set_value("URL Protocol", &"")
            .map_err(|e| e.to_string())?;

        let (command_key, _) = nxm_key
            .create_subkey_with_flags("shell\\open\\command", KEY_WRITE)
            .map_err(|e| e.to_string())?;
        command_key
            .set_value("", &command)
            .map_err(|e| e.to_string())?;

        println!("Successfully registered nxm:// protocol handler to current user.");
    }

    #[cfg(target_os = "linux")]
    {
        let get_exe_path = || std::env::current_exe().map_err(|e| e.to_string());
        let get_home = || linux_home_from_env(&|k| std::env::var(k).ok());
        register_nxm_protocol_command_linux_with(
            &get_exe_path,
            &get_home,
            &create_dir_all_string,
            &write_string_file,
            &run_xdg_mime_command,
        )?;
    }

    Ok(())
}

#[tauri::command]
pub async fn login_to_nexus(app: AppHandle) -> Result<String, String> {
    log_internal(&app, "INFO", "Starting Nexus login process...");
    let uuid = Uuid::new_v4().to_string();

    let sso_url = parse_nexus_ws_url_with(nexus_ws_url(), &parse_nexus_ws_url).map_err(|err| {
        log_internal(&app, "ERROR", &err);
        err
    })?;

    log_internal(&app, "INFO", &format!("Connecting to: {}", sso_url));

    let (ws_stream, _) = connect_async(sso_url.to_string()).await.map_err(|e| {
        let err = format!("Failed to connect to Nexus WebSocket: {}", e);
        log_internal(&app, "ERROR", &err);
        err
    })?;

    log_internal(&app, "INFO", "WebSocket connection established");

    let (mut write, read) = ws_stream.split();
    let msg = handshake_payload(&uuid);
    send_handshake_with(
        msg.to_string(),
        |level, message| log_internal(&app, level, message),
        |message| async move { write.send(message).await.map_err(|e| e.to_string()) },
    )
    .await?;

    open_auth_url_with(&uuid, &mut |url| open::that(url).map_err(|e| e.to_string()))?;

    let messages = read.map(|m| m.map_err(|e| e.to_string()));
    await_api_key_from_messages_with(messages, |message| {
        handle_login_ws_message_with(message, &|text| {
            handle_login_text_with(text, &parse_login_message_for_api_key, &mut |api_key| {
                persist_api_key_for_login_with(
                    api_key,
                    &|| get_auth_file_path(&app),
                    &save_api_key_to_auth_path,
                )
            })
        })
    })
    .await
}

#[tauri::command]
pub fn logout_nexus(app: AppHandle) -> Result<(), String> {
    if logout_nexus_command_with(&|| get_auth_file_path(&app), &remove_auth_file_if_exists)? {
        println!("Logged out. Auth file deleted.");
    }
    Ok(())
}
