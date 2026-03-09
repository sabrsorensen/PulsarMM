use super::{
    apply_archive_decision, extraction_progress_step, make_progress_payload, ArchiveDecision,
};
use crate::models::InstallationAnalysis;
use crate::mods::install_planning::FinalizeRequest;

fn empty_analysis() -> InstallationAnalysis {
    InstallationAnalysis {
        successes: Vec::new(),
        conflicts: Vec::new(),
        messy_archive_path: None,
        active_archive_path: None,
        selection_needed: false,
        temp_id: None,
        available_folders: None,
    }
}

#[test]
fn make_progress_payload_sets_fields() {
    let payload = make_progress_payload("d1", "Initializing...".to_string(), Some(10));
    assert_eq!(payload.id, "d1");
    assert_eq!(payload.step, "Initializing...");
    assert_eq!(payload.progress, Some(10));
}

#[test]
fn make_progress_payload_allows_missing_progress() {
    let payload = make_progress_payload("d2", "Waiting...".to_string(), None);
    assert_eq!(payload.id, "d2");
    assert_eq!(payload.step, "Waiting...");
    assert_eq!(payload.progress, None);
}

#[test]
fn extraction_progress_step_formats_percent() {
    assert_eq!(extraction_progress_step(42), "Extracting: 42%");
}

#[test]
fn apply_archive_decision_wait_for_selection_emits_and_returns_analysis() {
    let mut analysis = empty_analysis();
    analysis.selection_needed = true;
    let mut steps = Vec::new();

    let out =
        apply_archive_decision(
            "lib-id".to_string(),
            ArchiveDecision::WaitForSelection(analysis),
            "archive.zip".to_string(),
            |step| steps.push(step.to_string()),
            |_library_id, _selected_folders, _flatten| {
                Err("finalizer should not be called".to_string())
            },
        )
        .expect("wait-for-selection should return analysis");

    assert_eq!(steps, vec!["Waiting for selection..."]);
    assert!(out.selection_needed);
    assert!(out.active_archive_path.is_none());
}

#[test]
fn apply_archive_decision_finalize_calls_finalizer_and_sets_archive_path() {
    let decision = ArchiveDecision::Finalize(FinalizeRequest {
        selected_folders: vec!["folder-a".to_string()],
        flatten_paths: true,
    });
    let mut steps = Vec::new();

    let out = apply_archive_decision(
        "lib-id".to_string(),
        decision,
        "/tmp/archive.zip".to_string(),
        |step| steps.push(step.to_string()),
        |library_id, selected_folders, flatten_paths| {
            assert_eq!(library_id, "lib-id");
            assert_eq!(selected_folders, vec!["folder-a".to_string()]);
            assert!(flatten_paths);
            Ok(empty_analysis())
        },
    )
    .expect("finalize should succeed");

    assert_eq!(steps, vec!["Finalizing..."]);
    assert_eq!(out.active_archive_path.as_deref(), Some("/tmp/archive.zip"));
}

#[test]
fn apply_archive_decision_propagates_finalizer_error() {
    let decision = ArchiveDecision::Finalize(FinalizeRequest {
        selected_folders: vec![],
        flatten_paths: false,
    });

    let result = apply_archive_decision(
        "lib-id".to_string(),
        decision,
        "archive.zip".to_string(),
        |_step| {},
        |_library_id, _selected_folders, _flatten| Err("boom".to_string()),
    );

    match result {
        Err(err) => assert_eq!(err, "boom"),
        Ok(_) => panic!("expected finalizer error to propagate"),
    }
}
