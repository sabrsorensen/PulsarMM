use crate::mods::ordering;
use crate::mods::settings_store;
use std::path::Path;

pub fn reorder_mods_from_settings(
    settings_file_path: &Path,
    ordered_mod_names: &[String],
) -> Result<String, String> {
    let mut root = settings_store::load_settings_file(settings_file_path)?;
    ordering::reorder_mods(&mut root, ordered_mod_names);
    settings_store::to_formatted_xml(&root)
}

pub fn rename_mod_in_settings(
    settings_file_path: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<String, String> {
    let mut root = settings_store::load_settings_file(settings_file_path)?;
    ordering::rename_mod_in_xml(&mut root, old_name, new_name)?;
    settings_store::to_formatted_xml(&root)
}

pub fn delete_mod_and_save_settings(
    settings_file_path: &Path,
    mod_name: &str,
) -> Result<(), String> {
    let mut root = settings_store::load_settings_file(settings_file_path)?;
    ordering::delete_mod_and_reindex(&mut root, mod_name);
    settings_store::save_settings_file(settings_file_path, &root)
}

#[cfg(test)]
#[path = "command_mutations_tests.rs"]
mod tests;
