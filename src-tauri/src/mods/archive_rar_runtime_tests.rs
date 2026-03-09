use super::run_rar_processing_loop;
use super::{current_dir_locked, extract_rar_archive_with, with_destination_current_dir};
use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_archive_rar_runtime_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn run_rar_processing_loop_emits_progress_until_complete() {
    let steps = RefCell::new(vec![true, true, false].into_iter());
    let progress = RefCell::new(Vec::<u64>::new());
    let mut step = || Ok(steps.borrow_mut().next().expect("step should exist"));
    let mut on_progress = |pct| progress.borrow_mut().push(pct);

    run_rar_processing_loop(&mut step, &mut on_progress).expect("rar loop should complete");

    assert_eq!(*progress.borrow(), vec![0, 0]);
}

#[test]
fn run_rar_processing_loop_propagates_errors_after_prior_progress() {
    let calls = Cell::new(0usize);
    let progress = RefCell::new(Vec::<u64>::new());
    let mut step = || {
        let call = calls.get();
        calls.set(call + 1);
        match call {
            0 => Ok(true),
            1 => Err("rar-step-failed".to_string()),
            _ => Ok(false),
        }
    };
    let mut on_progress = |pct| progress.borrow_mut().push(pct);

    let err =
        run_rar_processing_loop(&mut step, &mut on_progress).expect_err("rar loop should fail");

    assert_eq!(err, "rar-step-failed");
    assert_eq!(*progress.borrow(), vec![0]);
}

#[test]
fn extract_rar_archive_with_runs_processing_in_destination_and_reports_completion() {
    let root = temp_test_dir("rar_wrapper_ok");
    let destination = root.join("dest");
    fs::create_dir_all(&destination).expect("failed to create destination");
    let progress = RefCell::new(Vec::<u64>::new());
    let mut on_progress = |pct| progress.borrow_mut().push(pct);
    let mut run_processing = |cb: &mut dyn FnMut(u64)| {
        cb(0);
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        if cwd != destination {
            return Err("wrong-destination".to_string());
        }
        Ok(())
    };

    extract_rar_archive_with(&destination, &mut on_progress, &mut run_processing)
        .expect("rar wrapper should succeed");

    assert_eq!(*progress.borrow(), vec![0, 100]);

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn extract_rar_archive_with_restores_working_dir_and_propagates_processing_error() {
    let root = temp_test_dir("rar_wrapper_err");
    let destination = root.join("dest");
    fs::create_dir_all(&destination).expect("failed to create destination");
    let cwd_before = current_dir_locked().expect("cwd should be readable");
    let progress = RefCell::new(Vec::<u64>::new());
    let mut on_progress = |pct| progress.borrow_mut().push(pct);
    let mut run_processing = |_cb: &mut dyn FnMut(u64)| Err("rar-processing-failed".to_string());

    let err = extract_rar_archive_with(&destination, &mut on_progress, &mut run_processing)
        .expect_err("rar wrapper should fail");

    assert_eq!(err, "rar-processing-failed");
    assert!(progress.borrow().is_empty());
    assert_eq!(
        current_dir_locked().expect("cwd should still be readable"),
        cwd_before
    );

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn with_destination_current_dir_restores_working_dir_on_success_and_error() {
    let root = temp_test_dir("cwd_restore");
    let destination = root.join("dest");
    fs::create_dir_all(&destination).expect("failed to create destination");

    let cwd_before = current_dir_locked().expect("cwd should be readable");
    let seen = with_destination_current_dir(&destination, || {
        std::env::current_dir()
            .map_err(|e| e.to_string())
            .map(|p| p == destination)
    })
    .expect("cwd wrapper should succeed");
    assert!(seen);
    assert_eq!(
        current_dir_locked().expect("cwd should still be readable"),
        cwd_before
    );

    let err = with_destination_current_dir(&destination, || Err::<(), String>("boom".to_string()))
        .expect_err("cwd wrapper should propagate error");
    assert_eq!(err, "boom");
    assert_eq!(
        current_dir_locked().expect("cwd should still be readable"),
        cwd_before
    );

    fs::remove_dir_all(root).expect("cleanup should succeed");
}
