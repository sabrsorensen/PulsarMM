use std::fs;
use std::path::{Path, PathBuf};

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

fn ensure_parent_dir_exists(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(io_error_to_string)?;
        }
    }

    Ok(())
}

fn smart_deploy_file(source: &Path, dest: &Path) -> Result<(), String> {
    ensure_parent_dir_exists(dest)?;

    if dest.exists() {
        fs::remove_file(dest).map_err(io_error_to_string)?;
    }

    if fs::hard_link(source, dest).is_ok() {
        return Ok(());
    }

    fs::copy(source, dest).map_err(io_error_to_string)?;
    Ok(())
}

pub(crate) fn deploy_structure_recursive(source: &Path, dest: &Path) -> Result<(), String> {
    if !dest.exists() {
        fs::create_dir_all(dest).map_err(io_error_to_string)?;
    }

    for entry in fs::read_dir(source).map_err(io_error_to_string)? {
        let entry = entry.map_err(io_error_to_string)?;
        let file_type = entry.file_type().map_err(io_error_to_string)?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            deploy_structure_recursive(&src_path, &dest_path)?;
        } else {
            smart_deploy_file(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

pub(crate) fn find_folder_in_tree(root: &Path, target_name: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case(target_name) {
                        return Some(path);
                    }
                }
                if let Some(found) = find_folder_in_tree(&path, target_name) {
                    return Some(found);
                }
            }
        }
    }
    None
}

pub(crate) fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(io_error_to_string)? {
        let entry = entry.map_err(io_error_to_string)?;
        let file_type = entry.file_type().map_err(io_error_to_string)?;
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path).map_err(io_error_to_string)?;
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path).map_err(io_error_to_string)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "fs_ops_tests.rs"]
mod tests;
