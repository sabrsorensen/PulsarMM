use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DeployOp {
    pub source: PathBuf,
    pub dest_name: String,
}

fn folder_name_for_path(path: &Path) -> Result<String, String> {
    path.file_name()
        .ok_or_else(|| "Invalid path".to_string())
        .map(|name| name.to_string_lossy().into_owned())
}

fn push_unique_op(ops: &mut Vec<DeployOp>, source: PathBuf, dest_name: String) {
    if ops
        .iter()
        .any(|op| op.dest_name.eq_ignore_ascii_case(&dest_name))
    {
        return;
    }

    ops.push(DeployOp { source, dest_name });
}

fn add_unique_op(ops: &mut Vec<DeployOp>, source: PathBuf) -> Result<(), String> {
    let dest_name = folder_name_for_path(&source)?;
    push_unique_op(ops, source, dest_name);
    Ok(())
}

pub fn scan_for_installable_mods(dir: &Path, base_dir: &Path) -> Vec<String> {
    let mut candidates = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        let mut is_mod_root = false;
        let mut subdirs = Vec::new();

        let game_structure_folders = [
            "AUDIO",
            "FONTS",
            "GLOBALS",
            "INPUT",
            "LANGUAGE",
            "MATERIALS",
            "METADATA",
            "MODELS",
            "MUSIC",
            "PIPELINES",
            "SCENES",
            "SHADERS",
            "TEXTURES",
            "TPFSDICT",
            "UI",
        ];

        let game_file_extensions = ["exml", "mbin", "dds", "mxml"];

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if game_structure_folders
                        .iter()
                        .any(|gf| name.eq_ignore_ascii_case(gf))
                    {
                        is_mod_root = true;
                    }
                }
                subdirs.push(path);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if game_file_extensions
                    .iter()
                    .any(|ge| ext.eq_ignore_ascii_case(ge))
                {
                    is_mod_root = true;
                }
            }
        }

        if is_mod_root {
            if let Ok(rel) = dir.strip_prefix(base_dir) {
                let rel_str = rel.to_string_lossy().replace("\\", "/");
                if !rel_str.is_empty() {
                    candidates.push(rel_str);
                } else {
                    candidates.push(".".to_string());
                }
            }
            return candidates;
        }

        for subdir in subdirs {
            let sub_candidates = scan_for_installable_mods(&subdir, base_dir);
            candidates.extend(sub_candidates);
        }
    }
    candidates
}

pub fn select_items_to_process(
    source_root: &Path,
    selected_folders: &[String],
) -> Result<Vec<String>, String> {
    if selected_folders.is_empty() || (selected_folders.len() == 1 && selected_folders[0] == ".") {
        return Ok(fs::read_dir(source_root)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect::<Vec<String>>());
    }

    Ok(selected_folders.to_vec())
}

pub fn build_deploy_ops(
    source_root: &Path,
    items_to_process: Vec<String>,
    flatten_paths: bool,
) -> Result<Vec<DeployOp>, String> {
    let mut ops: Vec<DeployOp> = Vec::new();

    for relative_path_str in items_to_process {
        let source_path = if relative_path_str == "." {
            source_root.to_path_buf()
        } else {
            source_root.join(&relative_path_str)
        };

        if !source_path.exists() {
            continue;
        }

        if flatten_paths {
            let deep_candidates = scan_for_installable_mods(&source_path, &source_path);

            if !deep_candidates.is_empty() {
                for deep_rel in deep_candidates {
                    let deep_source = if deep_rel == "." {
                        source_path.clone()
                    } else {
                        source_path.join(&deep_rel)
                    };
                    add_unique_op(&mut ops, deep_source)?;
                }
            } else {
                add_unique_op(&mut ops, source_path)?;
            }
        } else {
            add_unique_op(&mut ops, source_path)?;
        }
    }

    Ok(ops)
}

#[cfg(test)]
#[path = "install_scan_tests.rs"]
mod tests;
