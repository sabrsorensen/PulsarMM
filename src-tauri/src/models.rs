use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModProperty {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value", default)]
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Property")]
pub struct ModEntry {
    #[serde(rename = "@name")]
    pub entry_name: String,
    #[serde(rename = "@value")]
    pub entry_value: String,
    #[serde(rename = "@_index")]
    pub index: String,
    #[serde(rename = "Property", default)]
    pub properties: Vec<ModProperty>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopLevelProperty {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value", default)]
    pub value: Option<String>,
    #[serde(rename = "Property", default)]
    pub mods: Vec<ModEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Data")]
pub struct SettingsData {
    #[serde(rename = "@template")]
    pub template: String,
    #[serde(rename = "Property")]
    pub properties: Vec<TopLevelProperty>,
}

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct ModInfo {
    #[serde(rename = "modId", alias = "id")]
    pub mod_id: Option<String>,
    #[serde(rename = "fileId")]
    pub file_id: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "installSource")]
    pub install_source: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct ModInstallInfo {
    pub name: String,
    pub temp_path: String,
}

#[derive(serde::Serialize, Clone)]
pub struct ModConflictInfo {
    pub new_mod_name: String,
    pub temp_path: String,
    pub old_mod_folder_name: String,
}

#[derive(serde::Serialize)]
pub struct InstallationAnalysis {
    pub successes: Vec<ModInstallInfo>,
    pub conflicts: Vec<ModConflictInfo>,
    pub messy_archive_path: Option<String>,
    pub active_archive_path: Option<String>,
    pub selection_needed: bool,
    pub temp_id: Option<String>,
    pub available_folders: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
pub struct LocalModInfo {
    pub folder_name: String,
    pub mod_id: Option<String>,
    pub file_id: Option<String>,
    pub version: Option<String>,
    pub install_source: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ModRenderData {
    pub folder_name: String,
    pub enabled: bool,
    pub priority: u32,
    pub local_info: Option<LocalModInfo>,
}

#[derive(Serialize, Clone)]
pub struct DownloadResult {
    pub path: String,
    pub size: u64,
    pub created_at: u64,
}

#[derive(Serialize, Clone)]
pub struct GamePaths {
    pub game_root_path: String,
    pub settings_root_path: String,
    pub version_type: String,
    pub settings_initialized: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileModEntry {
    pub filename: String,
    pub mod_id: Option<String>,
    pub file_id: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub installed_options: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModProfileData {
    pub name: String,
    pub mods: Vec<ProfileModEntry>,
}

#[derive(Serialize, Clone)]
pub struct ProfileSwitchProgress {
    pub current: usize,
    pub total: usize,
    pub current_mod: String,
    pub file_progress: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalAppConfig {
    pub custom_download_path: Option<String>,
    pub custom_library_path: Option<String>,
    #[serde(default)]
    pub custom_game_path: Option<String>,
    #[serde(default)]
    pub legacy_migration_done: bool,
}

#[derive(Serialize, Clone)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Serialize, Clone)]
pub struct InstallProgressPayload {
    pub id: String,
    pub step: String,
    pub progress: Option<u64>,
}

pub const CLEAN_MXML_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Data template="GcModSettings">
  <Property name="DisableAllMods" value="false" />
  <Property name="Data">
  </Property>
</Data>"#;

pub static DIR_LOCK: Mutex<()> = Mutex::new(());

pub struct StartupState {
    pub pending_nxm: Mutex<Option<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}
