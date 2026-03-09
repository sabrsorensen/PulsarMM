use std::fs;
use std::path::{Path, PathBuf};

pub fn linux_desktop_file_path(home: &Path) -> PathBuf {
    home.join(".local/share/applications/nxm-handler.desktop")
}

pub fn linux_desktop_entry(exe_path: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
            Type=Application\n\
            Name=Pulsar Mod Manager\n\
            Exec=\"{}\" %u\n\
            StartupNotify=false\n\
            MimeType=x-scheme-handler/nxm;\n",
        exe_path.to_string_lossy()
    )
}

#[cfg(any(test, target_os = "windows"))]
pub fn windows_nxm_command(exe_path: &Path) -> String {
    format!("\"{}\" \"%1\"", exe_path.to_string_lossy())
}

pub fn ensure_auth_file_path(
    app_data_dir: &Path,
    create_dir_all: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    if !app_data_dir.exists() {
        create_dir_all(app_data_dir)?;
    }
    Ok(app_data_dir.join("auth.json"))
}

pub fn linux_unregister_nxm_protocol_with(
    home: &str,
    remove_file: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let desktop_file = linux_desktop_file_path(Path::new(home));
    if desktop_file.exists() {
        remove_file(&desktop_file)?;
    }
    Ok(())
}

pub fn linux_protocol_handler_registered(home: &str) -> bool {
    let desktop_file = linux_desktop_file_path(Path::new(home));
    desktop_file.exists()
}

pub fn linux_register_nxm_protocol_with(
    home: &str,
    desktop_content: &str,
    create_dir_all: &dyn Fn(&Path) -> Result<(), String>,
    write_file: &dyn Fn(&Path, &str) -> Result<(), String>,
    run_xdg_mime: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    let apps_dir = PathBuf::from(home).join(".local/share/applications");
    if !apps_dir.exists() {
        create_dir_all(&apps_dir)?;
    }

    let desktop_file = linux_desktop_file_path(Path::new(home));
    write_file(&desktop_file, desktop_content)?;
    run_xdg_mime()?;
    Ok(())
}

pub fn save_api_key_to_auth_path(auth_path: &Path, api_key: &str) -> Result<(), String> {
    let auth_data = serde_json::json!({ "apikey": api_key });
    let auth_json =
        serde_json::to_string_pretty(&auth_data).expect("serializing auth data should not fail");
    fs::write(auth_path, auth_json).map_err(|e| format!("Failed to save auth file: {}", e))
}

pub fn remove_auth_file_if_exists(auth_path: &Path) -> Result<bool, String> {
    if !auth_path.exists() {
        return Ok(false);
    }
    fs::remove_file(auth_path).map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)]
#[path = "command_ops_protocol_tests.rs"]
mod command_ops_protocol_tests;
