use crate::mods::command_logic::{map_update_mod_id_error, mod_info_path_for};
use crate::mods::info_ops::{ensure_mod_info_file, update_mod_id_in_json_file, EnsureModInfoInput};
use std::path::Path;

pub fn update_mod_id_in_game_path(
    game_path: &Path,
    mod_folder_name: &str,
    new_mod_id: &str,
) -> Result<(), String> {
    let mod_info_path = mod_info_path_for(game_path, mod_folder_name);
    update_mod_id_in_json_file(&mod_info_path, new_mod_id)
        .map_err(|e| map_update_mod_id_error(mod_folder_name, e))
}

pub fn ensure_mod_info_in_game_path(
    game_path: &Path,
    mod_folder_name: &str,
    input: &EnsureModInfoInput,
) -> Result<(), String> {
    let mod_info_path = mod_info_path_for(game_path, mod_folder_name);
    ensure_mod_info_file(&mod_info_path, input)
}

#[cfg(test)]
#[path = "command_info_ops_tests.rs"]
mod tests;
