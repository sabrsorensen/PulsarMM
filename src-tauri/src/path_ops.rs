use crate::models::FileNode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn downloads_target_from_root(root: &Path) -> PathBuf {
    root.join("downloads")
}

pub(crate) fn library_target_from_root(root: &Path) -> PathBuf {
    root.join("Library")
}

pub(crate) fn validate_move_target(old_path: &Path, target_path: &Path) -> Result<(), String> {
    if old_path == target_path {
        return Ok(());
    }
    if target_path.starts_with(old_path) {
        return Err(
            "Cannot move the folder inside itself. Please select a different location.".to_string(),
        );
    }
    Ok(())
}

pub(crate) fn sort_file_nodes(nodes: &mut [FileNode]) {
    nodes.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
}

pub(crate) fn check_library_existence_map(
    library_dir: &Path,
    filenames: Vec<String>,
) -> HashMap<String, bool> {
    let mut results = HashMap::new();
    for name in filenames {
        let folder_name = format!("{}_unpacked", name);
        let path = library_dir.join(folder_name);
        results.insert(name, path.exists());
    }
    results
}

#[cfg(test)]
#[path = "path_ops_tests.rs"]
mod tests;
