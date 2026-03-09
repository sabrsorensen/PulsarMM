use crate::nexus::auth::{auth_url, parse_api_key_message};
use crate::nexus::command_ops::linux_desktop_entry;
use crate::nexus::command_ops::{ensure_auth_file_path, linux_register_nxm_protocol_with};
use futures_util::{Stream, StreamExt};
use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

type PathResultFn<'a> = dyn Fn() -> Result<PathBuf, String> + 'a;
type EnsureAuthPathFn<'a> = dyn for<'b> Fn(&'b Path) -> Result<PathBuf, String> + 'a;
type CreateDirFn<'a> = dyn for<'b> Fn(&'b Path) -> Result<(), String> + 'a;
type LoadApiKeyFn<'a> = dyn for<'b> Fn(&'b Path) -> Result<String, String> + 'a;
type RemoveAuthFn<'a> = dyn for<'b> Fn(&'b Path) -> Result<bool, String> + 'a;
type ParseUrlFn<'a> = dyn for<'b> Fn(&'b str) -> Result<Url, String> + 'a;
type GetEnvFn<'a> = dyn for<'b> Fn(&'b str) -> Option<String> + 'a;
type RemoveHomeFileFn<'a> = dyn for<'b> Fn(&'b str) -> Result<(), String> + 'a;
type HomeCheckFn<'a> = dyn for<'b> Fn(&'b str) -> bool + 'a;
type WriteFileFn<'a> = dyn for<'b, 'c> Fn(&'b Path, &'c str) -> Result<(), String> + 'a;
type ParseLoginFn<'a> = dyn for<'b> Fn(&'b str) -> Result<Option<String>, String> + 'a;
type SaveApiKeyFn<'a> = dyn for<'b, 'c> Fn(&'b Path, &'c str) -> Result<(), String> + 'a;

pub(crate) fn get_auth_file_path_with(
    app_data_dir_provider: &PathResultFn<'_>,
    ensure_auth_path: &EnsureAuthPathFn<'_>,
) -> Result<PathBuf, String> {
    let app_data_dir = app_data_dir_provider()?;
    ensure_auth_path(&app_data_dir)
}

pub(crate) fn ensure_auth_path_for_app_data(
    app_data_dir: &Path,
    create_dir_all: &CreateDirFn<'_>,
) -> Result<PathBuf, String> {
    ensure_auth_file_path(app_data_dir, create_dir_all)
}

fn get_nexus_api_key_with(
    auth_path: &Path,
    load_api_key: &LoadApiKeyFn<'_>,
) -> Result<String, String> {
    load_api_key(auth_path)
}

fn logout_nexus_with(
    auth_path: &Path,
    remove_auth_file: &RemoveAuthFn<'_>,
) -> Result<bool, String> {
    remove_auth_file(auth_path)
}

pub(crate) fn get_nexus_api_key_command_with(
    get_auth_file_path: &PathResultFn<'_>,
    load_api_key: &LoadApiKeyFn<'_>,
) -> Result<String, String> {
    let auth_path = get_auth_file_path()?;
    let key = get_nexus_api_key_with(&auth_path, load_api_key)?;
    println!("Loaded API Key from AppData");
    Ok(key)
}

pub(crate) fn logout_nexus_command_with(
    get_auth_file_path: &PathResultFn<'_>,
    remove_auth_file: &RemoveAuthFn<'_>,
) -> Result<bool, String> {
    let auth_path = get_auth_file_path()?;
    logout_nexus_with(&auth_path, remove_auth_file)
}

#[cfg(target_os = "linux")]
pub(crate) fn unregister_nxm_protocol_command_linux_with(
    get_home: &(dyn Fn() -> Result<String, String> + '_),
    remove_file: &RemoveHomeFileFn<'_>,
) -> Result<(), String> {
    unregister_nxm_protocol_linux_default_with(get_home, remove_file)
}

#[cfg(target_os = "linux")]
pub(crate) fn is_protocol_handler_registered_command_linux_with(
    get_home: &(dyn Fn() -> Option<String> + '_),
    check_registered: &HomeCheckFn<'_>,
) -> bool {
    is_protocol_handler_registered_linux_with(get_home, check_registered)
}

#[cfg(target_os = "linux")]
pub(crate) fn register_nxm_protocol_command_linux_with(
    get_exe_path: &(dyn Fn() -> Result<PathBuf, String> + '_),
    get_home: &(dyn Fn() -> Result<String, String> + '_),
    create_dir_all: &CreateDirFn<'_>,
    write_file: &WriteFileFn<'_>,
    run_xdg_mime: &(dyn Fn() -> Result<(), String> + '_),
) -> Result<(), String> {
    register_nxm_protocol_linux_default_with(
        get_exe_path,
        get_home,
        create_dir_all,
        write_file,
        run_xdg_mime,
    )
}

pub(crate) fn parse_login_message_for_api_key(text: &str) -> Result<Option<String>, String> {
    parse_api_key_message(text)
}

pub(crate) fn nexus_ws_url() -> &'static str {
    "wss://sso.nexusmods.com"
}

pub(crate) fn parse_nexus_ws_url_with(
    ws_url: &str,
    parse_url: &ParseUrlFn<'_>,
) -> Result<Url, String> {
    parse_url(ws_url)
}

pub(crate) fn linux_home_from_env(get_env: &GetEnvFn<'_>) -> Result<String, String> {
    get_env("HOME").ok_or_else(|| "Could not find HOME".to_string())
}

fn is_protocol_handler_registered_linux_with(
    get_home: &(dyn Fn() -> Option<String> + '_),
    check_registered: &HomeCheckFn<'_>,
) -> bool {
    let home = get_home().unwrap_or_default();
    check_registered(&home)
}

fn unregister_nxm_protocol_linux_with(
    get_home: &(dyn Fn() -> Result<String, String> + '_),
    remove_file: &RemoveHomeFileFn<'_>,
) -> Result<(), String> {
    let home = get_home()?;
    remove_file(&home)
}

fn unregister_nxm_protocol_linux_default_with(
    get_home: &(dyn Fn() -> Result<String, String> + '_),
    remove_file_impl: &RemoveHomeFileFn<'_>,
) -> Result<(), String> {
    unregister_nxm_protocol_linux_with(get_home, remove_file_impl)
}

fn register_nxm_protocol_linux_with(
    get_exe_path: &(dyn Fn() -> Result<PathBuf, String> + '_),
    get_home: &(dyn Fn() -> Result<String, String> + '_),
    create_dir_all: &CreateDirFn<'_>,
    write_file: &WriteFileFn<'_>,
    run_xdg_mime: &(dyn Fn() -> Result<(), String> + '_),
) -> Result<(), String> {
    let exe_path = get_exe_path()?;
    let home = get_home()?;
    let desktop_content = linux_desktop_entry(&exe_path);
    linux_register_nxm_protocol_with(
        &home,
        &desktop_content,
        &create_dir_all,
        &write_file,
        &run_xdg_mime,
    )
}

fn register_nxm_protocol_linux_default_with(
    get_exe_path: &(dyn Fn() -> Result<PathBuf, String> + '_),
    get_home: &(dyn Fn() -> Result<String, String> + '_),
    create_dir_all: &CreateDirFn<'_>,
    write_file: &WriteFileFn<'_>,
    run_xdg_mime: &(dyn Fn() -> Result<(), String> + '_),
) -> Result<(), String> {
    register_nxm_protocol_linux_with(
        get_exe_path,
        get_home,
        create_dir_all,
        write_file,
        run_xdg_mime,
    )
}

pub(crate) fn handle_login_text_with(
    text: &str,
    parse: &ParseLoginFn<'_>,
    on_api_key: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<Option<String>, String> {
    if let Some(api_key) = parse(text)? {
        on_api_key(&api_key)?;
        return Ok(Some(api_key));
    }
    Ok(None)
}

pub(crate) fn persist_api_key_for_login_with(
    api_key: &str,
    get_auth_path: &PathResultFn<'_>,
    save_api_key: &SaveApiKeyFn<'_>,
) -> Result<(), String> {
    let auth_path = get_auth_path()?;
    save_api_key(&auth_path, api_key)?;
    println!("API Key saved to: {:?}", auth_path);
    Ok(())
}

pub(crate) fn handle_login_ws_message_with(
    message: Message,
    on_text: &ParseLoginFn<'_>,
) -> Result<Option<String>, String> {
    if let Message::Text(text) = message {
        return on_text(text.as_ref());
    }
    Ok(None)
}

pub(crate) async fn send_handshake_with<F, Fut>(
    msg: String,
    mut log: impl FnMut(&str, &str),
    send: F,
) -> Result<(), String>
where
    F: FnOnce(Message) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    log("INFO", &format!("Sending handshake: {}", msg));
    send(Message::Text(msg.into())).await.map_err(|e| {
        let err = format!("Failed to send handshake: {}", e);
        log("ERROR", &err);
        err
    })
}

pub(crate) fn open_auth_url_with(
    uuid: &str,
    open_url: &mut dyn FnMut(String) -> Result<(), String>,
) -> Result<(), String> {
    open_url(auth_url(uuid)).map_err(|e| format!("Failed to open Nexus auth URL: {}", e))
}

pub(crate) async fn await_api_key_from_messages_with<S, F>(
    mut messages: S,
    mut handle_message: F,
) -> Result<String, String>
where
    S: Stream<Item = Result<Message, String>> + Unpin,
    F: FnMut(Message) -> Result<Option<String>, String>,
{
    while let Some(message) = messages.next().await {
        if let Some(api_key) = handle_message(message?)? {
            return Ok(api_key);
        }
    }
    Err("Connection closed before authentication finished.".to_string())
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
