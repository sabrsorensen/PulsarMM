use crate::fs_ops::copy_dir_recursive;
use std::fs;
use std::path::Path;

fn move_dir_safely_with(
    src: &Path,
    dest: &Path,
    rename_dir: &mut dyn FnMut(&Path, &Path) -> std::io::Result<()>,
    create_dir_all: &mut dyn FnMut(&Path) -> std::io::Result<()>,
    copy_dir: &mut dyn FnMut(&Path, &Path) -> Result<(), String>,
    remove_dir_all: &mut dyn FnMut(&Path) -> std::io::Result<()>,
) -> Result<(), String> {
    if rename_dir(src, dest).is_ok() {
        return Ok(());
    }

    if !dest.exists() {
        create_dir_all(dest).map_err(|e| format!("Failed to create dest dir: {}", e))?;
    }

    copy_dir(src, dest)?;
    remove_dir_all(src).map_err(|e| format!("Failed to remove source after copy: {}", e))?;
    Ok(())
}

pub fn move_dir_safely(src: &Path, dest: &Path) -> Result<(), String> {
    let mut rename_dir = |from: &Path, to: &Path| fs::rename(from, to);
    let mut create_dir = |path: &Path| fs::create_dir_all(path);
    let mut copy_dir = |from: &Path, to: &Path| copy_dir_recursive(from, to);
    let mut remove_dir = |path: &Path| fs::remove_dir_all(path);
    move_dir_safely_with(
        src,
        dest,
        &mut rename_dir,
        &mut create_dir,
        &mut copy_dir,
        &mut remove_dir,
    )
}

fn cleanup_empty_parent_dir(path: &Path) {
    let Some(parent) = path.parent() else {
        return;
    };
    if !parent.exists() {
        return;
    }
    if parent
        .read_dir()
        .map_or(false, |mut entries| entries.next().is_none())
    {
        let _ = fs::remove_dir(parent);
    }
}

pub fn resolve_conflict_in_paths(
    mods_path: &Path,
    new_mod_name: &str,
    old_mod_folder_name: &str,
    temp_mod_path: &Path,
    replace: bool,
) -> Result<(), String> {
    let old_mod_path = mods_path.join(old_mod_folder_name);
    let final_new_mod_path = mods_path.join(new_mod_name);

    if replace {
        if old_mod_path.exists() {
            fs::remove_dir_all(&old_mod_path)
                .map_err(|e| format!("Failed to remove old mod: {}", e))?;
        }
        move_dir_safely(temp_mod_path, &final_new_mod_path)?;
    } else {
        fs::remove_dir_all(temp_mod_path)
            .map_err(|e| format!("Failed to cleanup temp mod folder: {}", e))?;
    }

    cleanup_empty_parent_dir(temp_mod_path);
    Ok(())
}

#[cfg(test)]
#[path = "conflict_resolution_tests.rs"]
mod tests;
