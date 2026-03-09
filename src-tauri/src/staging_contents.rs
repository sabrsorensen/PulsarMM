use crate::models::FileNode;
use crate::path_ops::sort_file_nodes;
use std::fs;
use std::path::{Component, Path, PathBuf};

fn io_error_to_string(error: std::io::Error) -> String {
    error.to_string()
}

fn is_forbidden_component(component: Component<'_>) -> bool {
    matches!(
        component,
        Component::ParentDir | Component::RootDir | Component::Prefix(_)
    )
}

pub(crate) fn target_path_from_relative(
    root_path: &Path,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(relative_path);
    if relative.components().any(is_forbidden_component) {
        return Err("Invalid path access".to_string());
    }

    let target_path = if relative_path.is_empty() {
        root_path.to_path_buf()
    } else {
        root_path.join(relative)
    };
    Ok(target_path)
}

pub(crate) fn collect_nodes(
    root_path: &Path,
    relative_path: &str,
) -> Result<Vec<FileNode>, String> {
    let target_path = target_path_from_relative(root_path, relative_path)?;

    let mut nodes = Vec::new();
    if target_path.is_dir() {
        for entry in fs::read_dir(target_path).map_err(io_error_to_string)? {
            let entry = entry.map_err(io_error_to_string)?;
            let meta = entry.metadata().map_err(io_error_to_string)?;
            nodes.push(FileNode {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: meta.is_dir(),
            });
        }
    }

    sort_file_nodes(&mut nodes);
    Ok(nodes)
}

#[cfg(test)]
#[path = "staging_contents_tests.rs"]
mod tests;
