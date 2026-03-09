use crate::models::{InstallProgressPayload, InstallationAnalysis};
use crate::mods::install_command_core::{archive_path_from_input, build_install_archive_context};
use crate::mods::install_command_runtime::{
    copy_archive_to_downloads_async, extract_archive_if_needed_async, scan_library_mod_path_async,
};
use crate::mods::install_orchestration::apply_archive_decision;
use crate::mods::install_orchestration::{extraction_progress_step, make_progress_payload};
use crate::mods::install_planning::decide_archive_flow;
use std::path::PathBuf;

pub(crate) fn install_step_payload(download_id: &str, step: &str) -> InstallProgressPayload {
    make_progress_payload(download_id, step.to_string(), None)
}

pub(crate) fn extraction_step_payload(download_id: &str, pct: u64) -> InstallProgressPayload {
    make_progress_payload(download_id, extraction_progress_step(pct), Some(pct))
}

pub(crate) fn emit_install_step_with(
    download_id: &str,
    step: &str,
    emit_event: impl Fn(InstallProgressPayload),
) {
    emit_event(install_step_payload(download_id, step));
}

pub(crate) fn emit_extraction_step_with(
    download_id: &str,
    pct: u64,
    emit_event: impl Fn(InstallProgressPayload),
) {
    emit_event(extraction_step_payload(download_id, pct));
}

pub async fn install_mod_from_archive_with(
    archive_path_str: String,
    get_downloads_dir: impl Fn() -> Result<PathBuf, String>,
    get_library_dir: impl Fn() -> Result<PathBuf, String>,
    emit_progress: impl Fn(&str),
    extract_progress_callback: impl Fn(u64) + Send + 'static,
    finalize_installation: impl FnOnce(
        String,
        Vec<String>,
        bool,
    ) -> Result<InstallationAnalysis, String>,
) -> Result<InstallationAnalysis, String> {
    emit_progress("Initializing...");

    let archive_path = archive_path_from_input(&archive_path_str);
    let downloads_dir = get_downloads_dir()?;
    let library_dir = get_library_dir()?;

    emit_progress("Copying to library...");
    let final_archive_path = copy_archive_to_downloads_async(archive_path, downloads_dir).await?;

    let archive_context = build_install_archive_context(&final_archive_path, &library_dir)?;
    let library_mod_path = archive_context.library_mod_path.clone();
    extract_archive_if_needed_async(
        final_archive_path,
        library_mod_path.clone(),
        Box::new(extract_progress_callback),
    )
    .await?;

    emit_progress("Analyzing structure...");
    let (folder_names, installable_paths) = scan_library_mod_path_async(library_mod_path).await?;

    let decision = decide_archive_flow(
        &installable_paths,
        &folder_names,
        &archive_context.library_id,
        &archive_context.final_archive_path_str,
    );

    apply_archive_decision(
        archive_context.library_id,
        decision,
        archive_context.final_archive_path_str,
        emit_progress,
        finalize_installation,
    )
}

#[cfg(test)]
#[path = "install_command_flow_tests.rs"]
mod tests;
