use super::{
    emit_extraction_step_with, emit_install_step_with, extraction_step_payload,
    install_step_payload,
};
use crate::models::InstallProgressPayload;
use std::sync::{Arc, Mutex};

#[test]
fn install_progress_payload_helpers_build_expected_events() {
    let step = install_step_payload("download-1", "Initializing...");
    assert_eq!(step.id, "download-1");
    assert_eq!(step.step, "Initializing...");
    assert_eq!(step.progress, None);

    let extract = extraction_step_payload("download-1", 42);
    assert_eq!(extract.id, "download-1");
    assert_eq!(extract.step, "Extracting: 42%");
    assert_eq!(extract.progress, Some(42));

    let emitted = Arc::new(Mutex::new(Vec::<InstallProgressPayload>::new()));
    let emitted_step = emitted.clone();
    emit_install_step_with("download-2", "Copying to library...", move |payload| {
        emitted_step
            .lock()
            .expect("lock should succeed")
            .push(payload);
    });

    let emitted_extract = emitted.clone();
    emit_extraction_step_with("download-2", 77, move |payload| {
        emitted_extract
            .lock()
            .expect("lock should succeed")
            .push(payload);
    });

    let events = emitted.lock().expect("lock should succeed");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].step, "Copying to library...");
    assert_eq!(events[0].progress, None);
    assert_eq!(events[1].step, "Extracting: 77%");
    assert_eq!(events[1].progress, Some(77));
}
