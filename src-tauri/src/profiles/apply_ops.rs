use crate::fs_ops::{deploy_structure_recursive, find_folder_in_tree};
use crate::models::{ModProfileData, ProfileModEntry, CLEAN_MXML_TEMPLATE};
use std::fs;
use std::io;
use std::path::Path;

fn fs_error_to_string(error: io::Error) -> String {
    error.to_string()
}

fn json_copy_error(error: io::Error) -> String {
    format!("Failed to copy JSON: {}", error)
}

fn mxml_copy_error(error: io::Error) -> String {
    format!("Failed to copy MXML: {}", error)
}

pub(crate) fn has_specific_options(entry: &ProfileModEntry) -> bool {
    entry
        .installed_options
        .as_ref()
        .map(|o| !o.is_empty())
        .unwrap_or(false)
}

pub(crate) fn build_mod_info_json(entry: &ProfileModEntry) -> serde_json::Value {
    serde_json::json!({
        "modId": entry.mod_id,
        "fileId": entry.file_id,
        "version": entry.version,
        "installSource": entry.filename
    })
}

fn is_pak_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("pak"))
}

fn remove_mod_artifact(path: &Path) {
    if path.is_dir() {
        fs::remove_dir_all(path).ok();
    } else {
        fs::remove_file(path).ok();
    }
}

pub fn clear_mods_dir(mods_dir: &Path) -> Result<(), String> {
    if !mods_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(mods_dir).map_err(fs_error_to_string)? {
        let entry = entry.map_err(fs_error_to_string)?;
        let path = entry.path();
        if path.is_dir() || is_pak_file(&path) {
            remove_mod_artifact(&path);
        }
    }

    Ok(())
}

pub fn restore_or_create_live_mxml(
    mxml_backup_path: &Path,
    live_mxml: &Path,
) -> Result<(), String> {
    if mxml_backup_path.exists() {
        let mut src = fs::File::open(mxml_backup_path).map_err(fs_error_to_string)?;
        let mut dst = fs::File::create(live_mxml).map_err(fs_error_to_string)?;
        io::copy(&mut src, &mut dst).map_err(fs_error_to_string)?;
    } else {
        fs::write(live_mxml, CLEAN_MXML_TEMPLATE).map_err(fs_error_to_string)?;
    }
    Ok(())
}

fn serialize_mod_info(entry: &ProfileModEntry) -> String {
    let info_json = build_mod_info_json(entry);
    serde_json::to_string_pretty(&info_json).expect("serializing profile mod info should not fail")
}

fn write_mod_info(dest: &Path, entry: &ProfileModEntry) {
    let info_path = dest.join("mod_info.json");
    let _ = fs::write(info_path, serialize_mod_info(entry));
}

fn selected_options(entry: &ProfileModEntry) -> &[String] {
    entry
        .installed_options
        .as_deref()
        .expect("selected_options requires a non-empty installed_options list")
}

pub fn deploy_profile_entry(
    entry: &ProfileModEntry,
    library_mod_path: &Path,
    mods_dir: &Path,
) -> Result<(), String> {
    if !library_mod_path.exists() {
        return Ok(());
    }

    if has_specific_options(entry) {
        for target_folder_name in selected_options(entry) {
            let Some(source_path) = find_folder_in_tree(library_mod_path, target_folder_name)
            else {
                continue;
            };

            let dest = mods_dir.join(target_folder_name);
            if let Err(e) = deploy_structure_recursive(&source_path, &dest) {
                eprintln!("Failed to deploy {}: {}", target_folder_name, e);
                continue;
            }
            write_mod_info(&dest, entry);
        }
        return Ok(());
    }

    let Ok(entries) = fs::read_dir(library_mod_path) else {
        return Ok(());
    };

    for fs_entry in entries.flatten() {
        let folder_name = fs_entry.file_name().to_string_lossy().into_owned();
        let dest = mods_dir.join(&folder_name);
        let src = fs_entry.path();
        let _ = deploy_structure_recursive(&src, &dest);
        write_mod_info(&dest, entry);
    }
    Ok(())
}

fn rewrite_copied_profile_name(new_json: &Path, new_name: &str) -> Result<(), String> {
    let content = fs::read_to_string(new_json).map_err(fs_error_to_string)?;
    let mut data: ModProfileData = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    data.name = new_name.to_string();

    let new_content =
        serde_json::to_string_pretty(&data).expect("serializing ModProfileData should not fail");
    fs::write(new_json, new_content).map_err(fs_error_to_string)?;
    Ok(())
}

fn copy_or_create_profile_mxml(source_mxml: &Path, new_mxml: &Path) -> Result<(), String> {
    if source_mxml.exists() {
        fs::copy(source_mxml, new_mxml)
            .map(|_| ())
            .map_err(mxml_copy_error)?;
    } else {
        fs::write(new_mxml, CLEAN_MXML_TEMPLATE).map_err(fs_error_to_string)?;
    }
    Ok(())
}

pub fn copy_profile_from_dir(dir: &Path, source_name: &str, new_name: &str) -> Result<(), String> {
    let source_json = dir.join(format!("{}.json", source_name));
    let source_mxml = dir.join(format!("{}.mxml", source_name));

    let new_json = dir.join(format!("{}.json", new_name));
    let new_mxml = dir.join(format!("{}.mxml", new_name));

    if new_json.exists() {
        return Err("A profile with that name already exists.".to_string());
    }
    if !source_json.exists() {
        return Err("Source profile not found.".to_string());
    }

    fs::copy(&source_json, &new_json)
        .map(|_| ())
        .map_err(json_copy_error)?;
    rewrite_copied_profile_name(&new_json, new_name)?;
    copy_or_create_profile_mxml(&source_mxml, &new_mxml)?;

    Ok(())
}

#[cfg(test)]
#[path = "apply_ops_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "apply_ops_model_tests.rs"]
mod apply_ops_model_tests;
