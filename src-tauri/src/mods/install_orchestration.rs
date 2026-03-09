use crate::models::{InstallProgressPayload, InstallationAnalysis};
use crate::mods::install_planning::ArchiveDecision;

pub fn make_progress_payload(
    id: &str,
    step: String,
    progress: Option<u64>,
) -> InstallProgressPayload {
    InstallProgressPayload {
        id: id.to_string(),
        step,
        progress,
    }
}

pub fn extraction_progress_step(pct: u64) -> String {
    format!("Extracting: {}%", pct)
}

fn apply_archive_decision_with(
    library_id: String,
    decision: ArchiveDecision,
    final_archive_path_str: String,
    emit_progress: &mut dyn FnMut(&str),
    finalize: &mut dyn FnMut(String, Vec<String>, bool) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    match decision {
        ArchiveDecision::WaitForSelection(analysis) => {
            emit_progress("Waiting for selection...");
            Ok(analysis)
        }
        ArchiveDecision::Finalize(req) => {
            emit_progress("Finalizing...");
            let mut analysis = finalize(library_id, req.selected_folders, req.flatten_paths)?;
            analysis.active_archive_path = Some(final_archive_path_str);
            Ok(analysis)
        }
    }
}

pub fn apply_archive_decision<F>(
    library_id: String,
    decision: ArchiveDecision,
    final_archive_path_str: String,
    mut emit_progress: impl FnMut(&str),
    finalize: F,
) -> Result<InstallationAnalysis, String>
where
    F: FnOnce(String, Vec<String>, bool) -> Result<InstallationAnalysis, String>,
{
    let mut finalize = Some(finalize);
    apply_archive_decision_with(
        library_id,
        decision,
        final_archive_path_str,
        &mut emit_progress,
        &mut |library_id, selected_folders, flatten_paths| {
            finalize.take().expect("finalize called once")(
                library_id,
                selected_folders,
                flatten_paths,
            )
        },
    )
}

#[cfg(test)]
#[path = "install_orchestration_tests.rs"]
mod tests;
