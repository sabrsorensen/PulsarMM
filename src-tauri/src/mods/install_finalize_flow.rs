use crate::mods::install_planning::DeployCandidate;
use crate::mods::install_scan::DeployOp;
use std::path::{Path, PathBuf};

pub fn is_scan_all_selection(selected_folders: &[String]) -> bool {
    selected_folders.is_empty() || (selected_folders.len() == 1 && selected_folders[0] == ".")
}

pub fn conflict_staging_path(staging_dir: &Path, timestamp_millis: i64) -> PathBuf {
    staging_dir.join(format!("conflict_{}", timestamp_millis))
}

pub fn build_deploy_candidates_with<FReadModId>(
    ops: Vec<DeployOp>,
    mods_path: &Path,
    read_mod_id: FReadModId,
) -> Vec<DeployCandidate>
where
    FReadModId: Fn(&Path) -> Option<String>,
{
    ops.into_iter()
        .map(|op| DeployCandidate {
            mod_id: read_mod_id(&op.source),
            dest_exists: mods_path.join(&op.dest_name).exists(),
            source: op.source,
            dest_name: op.dest_name,
        })
        .collect()
}

#[cfg(test)]
#[path = "install_finalize_flow_tests.rs"]
mod tests;
