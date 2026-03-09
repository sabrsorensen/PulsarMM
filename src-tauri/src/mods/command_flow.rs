use crate::models::{DownloadResult, ModInfo, ModRenderData};
use crate::mods::command_ops::LibraryRenameSync;
use crate::mods::info_ops::EnsureModInfoInput;
use std::future::Future;
use std::path::{Path, PathBuf};

fn maybe_sync_library_rename_for_mod_with(
    old_path: &Path,
    old_name: &str,
    new_name: &str,
    read_mod_info: &dyn Fn(&Path) -> Option<ModInfo>,
    get_library_dir: &dyn Fn() -> Result<PathBuf, String>,
    sync_library_folder_rename: &dyn Fn(
        &Path,
        &str,
        &str,
        &str,
    ) -> Result<LibraryRenameSync, String>,
    log: &dyn Fn(&str, &str),
) {
    let Some(info) = read_mod_info(old_path) else {
        return;
    };
    let Some(source_zip) = info.install_source else {
        return;
    };
    let Ok(library_dir) = get_library_dir() else {
        return;
    };

    match sync_library_folder_rename(&library_dir, &source_zip, old_name, new_name) {
        Ok(LibraryRenameSync::Renamed) => {
            log("INFO", "Synced folder rename to Library successfully.");
        }
        Ok(LibraryRenameSync::SourceMissing | LibraryRenameSync::TargetExists) => {}
        Err(e) => {
            log("WARN", &e);
        }
    }
}

pub fn maybe_sync_library_rename_for_mod(
    old_path: &Path,
    old_name: &str,
    new_name: &str,
    read_mod_info: impl Fn(&Path) -> Option<ModInfo>,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    sync_library_folder_rename: impl Fn(&Path, &str, &str, &str) -> Result<LibraryRenameSync, String>,
    log: impl Fn(&str, &str),
) {
    maybe_sync_library_rename_for_mod_with(
        old_path,
        old_name,
        new_name,
        &read_mod_info,
        &get_library_dir,
        &sync_library_folder_rename,
        &log,
    );
}

fn maybe_persist_renamed_mod_settings_with(
    settings_file: &Path,
    old_name: String,
    new_name: String,
    update_mod_name_in_xml: &dyn Fn(String, String) -> Result<String, String>,
    save_file: &dyn Fn(String, String) -> Result<(), String>,
    log: &dyn Fn(&str, &str),
) {
    if !settings_file.exists() {
        return;
    }

    match update_mod_name_in_xml(old_name, new_name) {
        Ok(new_xml) => {
            let _ = save_file(settings_file.to_string_lossy().to_string(), new_xml);
        }
        Err(e) => {
            log(
                "WARN",
                &format!("Folder renamed, but XML update failed: {}", e),
            );
        }
    }
}

pub fn maybe_persist_renamed_mod_settings(
    settings_file: &Path,
    old_name: String,
    new_name: String,
    update_mod_name_in_xml: impl Fn(String, String) -> Result<String, String>,
    save_file: impl Fn(String, String) -> Result<(), String>,
    log: impl Fn(&str, &str),
) {
    maybe_persist_renamed_mod_settings_with(
        settings_file,
        old_name,
        new_name,
        &update_mod_name_in_xml,
        &save_file,
        &log,
    );
}

fn rename_mod_folder_flow_with(
    old_name: String,
    new_name: String,
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    mod_folder_path: &dyn Fn(&Path, &str) -> PathBuf,
    validate_rename_paths: &dyn Fn(bool, bool) -> Result<(), String>,
    maybe_sync_library_rename: &dyn Fn(&Path, &str, &str),
    rename_dir: &dyn Fn(&Path, &Path) -> Result<(), String>,
    settings_file_from_game_path: &dyn Fn(&Path) -> PathBuf,
    persist_renamed_settings: &dyn Fn(&Path, String, String),
    get_all_mods_for_render: &dyn Fn() -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    let game_path = find_game_path().ok_or_else(|| "Could not find game path.".to_string())?;
    let old_path = mod_folder_path(&game_path, &old_name);
    let new_path = mod_folder_path(&game_path, &new_name);
    validate_rename_paths(old_path.exists(), new_path.exists())?;

    maybe_sync_library_rename(&old_path, &old_name, &new_name);
    rename_dir(&old_path, &new_path)?;

    let settings_file = settings_file_from_game_path(&game_path);
    persist_renamed_settings(&settings_file, old_name, new_name);

    get_all_mods_for_render()
}

pub fn rename_mod_folder_flow(
    old_name: String,
    new_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    validate_rename_paths: impl Fn(bool, bool) -> Result<(), String>,
    maybe_sync_library_rename: impl Fn(&Path, &str, &str),
    rename_dir: impl Fn(&Path, &Path) -> Result<(), String>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    persist_renamed_settings: impl Fn(&Path, String, String),
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    rename_mod_folder_flow_with(
        old_name,
        new_name,
        &find_game_path,
        &mod_folder_path,
        &validate_rename_paths,
        &maybe_sync_library_rename,
        &rename_dir,
        &settings_file_from_game_path,
        &persist_renamed_settings,
        &get_all_mods_for_render,
    )
}

fn delete_mod_flow_with(
    mod_name: String,
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    settings_file_from_game_path: &dyn Fn(&Path) -> PathBuf,
    mod_folder_path: &dyn Fn(&Path, &str) -> PathBuf,
    maybe_remove_mod_folder: &dyn Fn(&Path, &str) -> Result<bool, String>,
    delete_mod_and_save_settings: &dyn Fn(&Path, &str) -> Result<(), String>,
    log: &dyn Fn(&str, &str),
    get_all_mods_for_render: &dyn Fn() -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    let game_path =
        find_game_path().ok_or_else(|| "Could not find game installation path.".to_string())?;
    let settings_file_path = settings_file_from_game_path(&game_path);
    let mod_to_delete_path = mod_folder_path(&game_path, &mod_name);

    if maybe_remove_mod_folder(&mod_to_delete_path, &mod_name)? {
        log("INFO", &format!("Deleted folder: {:?}", mod_to_delete_path));
    } else {
        log(
            "WARN",
            &format!("Folder not found for deletion: {:?}", mod_to_delete_path),
        );
    }

    delete_mod_and_save_settings(&settings_file_path, &mod_name)?;
    get_all_mods_for_render()
}

pub fn delete_mod_flow(
    mod_name: String,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    mod_folder_path: impl Fn(&Path, &str) -> PathBuf,
    maybe_remove_mod_folder: impl Fn(&Path, &str) -> Result<bool, String>,
    delete_mod_and_save_settings: impl Fn(&Path, &str) -> Result<(), String>,
    log: impl Fn(&str, &str),
    get_all_mods_for_render: impl Fn() -> Result<Vec<ModRenderData>, String>,
) -> Result<Vec<ModRenderData>, String> {
    delete_mod_flow_with(
        mod_name,
        &find_game_path,
        &settings_file_from_game_path,
        &mod_folder_path,
        &maybe_remove_mod_folder,
        &delete_mod_and_save_settings,
        &log,
        &get_all_mods_for_render,
    )
}

fn reorder_mods_flow_with(
    ordered_mod_names: &[String],
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    settings_file_from_game_path: &dyn Fn(&Path) -> PathBuf,
    reorder_mods_from_settings: &dyn Fn(&Path, &[String]) -> Result<String, String>,
) -> Result<String, String> {
    let game_path =
        find_game_path().ok_or_else(|| "Could not find game installation path.".to_string())?;
    let settings_file_path = settings_file_from_game_path(&game_path);
    reorder_mods_from_settings(&settings_file_path, ordered_mod_names)
}

pub fn reorder_mods_flow(
    ordered_mod_names: &[String],
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    reorder_mods_from_settings: impl Fn(&Path, &[String]) -> Result<String, String>,
) -> Result<String, String> {
    reorder_mods_flow_with(
        ordered_mod_names,
        &find_game_path,
        &settings_file_from_game_path,
        &reorder_mods_from_settings,
    )
}

fn update_mod_name_in_xml_flow_with(
    old_name: &str,
    new_name: &str,
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    settings_file_from_game_path: &dyn Fn(&Path) -> PathBuf,
    rename_mod_in_settings: &dyn Fn(&Path, &str, &str) -> Result<String, String>,
) -> Result<String, String> {
    let game_path =
        find_game_path().ok_or_else(|| "Could not find game installation path.".to_string())?;
    let settings_file_path = settings_file_from_game_path(&game_path);
    rename_mod_in_settings(&settings_file_path, old_name, new_name)
}

pub fn update_mod_name_in_xml_flow(
    old_name: &str,
    new_name: &str,
    find_game_path: impl Fn() -> Option<PathBuf>,
    settings_file_from_game_path: impl Fn(&Path) -> PathBuf,
    rename_mod_in_settings: impl Fn(&Path, &str, &str) -> Result<String, String>,
) -> Result<String, String> {
    update_mod_name_in_xml_flow_with(
        old_name,
        new_name,
        &find_game_path,
        &settings_file_from_game_path,
        &rename_mod_in_settings,
    )
}

fn update_mod_id_in_json_flow_with(
    mod_folder_name: &str,
    new_mod_id: &str,
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    update_mod_id_in_game_path: &dyn Fn(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    let game_path =
        find_game_path().ok_or_else(|| "Could not find game installation path.".to_string())?;
    update_mod_id_in_game_path(&game_path, mod_folder_name, new_mod_id)
}

pub fn update_mod_id_in_json_flow(
    mod_folder_name: &str,
    new_mod_id: &str,
    find_game_path: impl Fn() -> Option<PathBuf>,
    update_mod_id_in_game_path: impl Fn(&Path, &str, &str) -> Result<(), String>,
) -> Result<(), String> {
    update_mod_id_in_json_flow_with(
        mod_folder_name,
        new_mod_id,
        &find_game_path,
        &update_mod_id_in_game_path,
    )
}

fn ensure_mod_info_flow_with(
    mod_folder_name: &str,
    input: &EnsureModInfoInput,
    find_game_path: &dyn Fn() -> Option<PathBuf>,
    ensure_mod_info_in_game_path: &dyn Fn(&Path, &str, &EnsureModInfoInput) -> Result<(), String>,
) -> Result<(), String> {
    let game_path =
        find_game_path().ok_or_else(|| "Could not find game installation path.".to_string())?;
    ensure_mod_info_in_game_path(&game_path, mod_folder_name, input)
}

pub fn ensure_mod_info_flow(
    mod_folder_name: &str,
    input: &EnsureModInfoInput,
    find_game_path: impl Fn() -> Option<PathBuf>,
    ensure_mod_info_in_game_path: impl Fn(&Path, &str, &EnsureModInfoInput) -> Result<(), String>,
) -> Result<(), String> {
    ensure_mod_info_flow_with(
        mod_folder_name,
        input,
        &find_game_path,
        &ensure_mod_info_in_game_path,
    )
}

pub async fn download_mod_archive_flow<F>(
    file_name: &str,
    download_url: &str,
    download_id: Option<&str>,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String>,
    download_archive_to_path: impl Fn(String, PathBuf, Option<String>) -> F,
    log: impl Fn(&str, &str),
) -> Result<DownloadResult, String>
where
    F: Future<Output = Result<DownloadResult, String>>,
{
    let downloads_path = get_downloads_dir()?;
    let final_archive_path = downloads_path.join(file_name);
    let result = download_archive_to_path(
        download_url.to_string(),
        final_archive_path,
        download_id.map(|s| s.to_string()),
    )
    .await;
    if let Err(ref err) = result {
        log("ERROR", err);
    }
    result
}
