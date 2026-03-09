use super::*;
use crate::fs_ops::copy_dir_recursive;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_conflict_resolution_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn move_dir_safely_moves_into_existing_destination_when_rename_fails() {
    let root = temp_test_dir("move_dir_safely");
    let src = root.join("src_mod");
    let dest = root.join("dest_mod");

    fs::create_dir_all(src.join("nested")).expect("failed to create src");
    write_file(&src.join("nested/new_file.txt"), "new");

    fs::create_dir_all(&dest).expect("failed to create dest");
    write_file(&dest.join("existing.txt"), "existing");

    move_dir_safely(&src, &dest).expect("move_dir_safely failed");

    assert!(!src.exists(), "source directory should be removed");
    assert!(dest.join("existing.txt").exists());
    assert!(dest.join("nested/new_file.txt").exists());

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn move_dir_safely_with_covers_rename_copy_and_remove_paths() {
    let root = temp_test_dir("move_dir_safely_with");
    let src = root.join("src_mod");
    let dest = root.join("dest_mod");
    fs::create_dir_all(&src).expect("create src");

    let mut renamed = false;
    let mut rename_ok = |_: &Path, _: &Path| {
        renamed = true;
        Ok(())
    };
    let mut create_unreachable = |_: &Path| -> io::Result<()> {
        panic!("create_dir_all should not run after successful rename")
    };
    let mut copy_unreachable = |_: &Path, _: &Path| -> Result<(), String> {
        panic!("copy_dir should not run after successful rename")
    };
    let mut remove_unreachable = |_: &Path| -> io::Result<()> {
        panic!("remove_dir_all should not run after successful rename")
    };
    move_dir_safely_with(
        &src,
        &dest,
        &mut rename_ok,
        &mut create_unreachable,
        &mut copy_unreachable,
        &mut remove_unreachable,
    )
    .expect("rename success should short-circuit");
    assert!(renamed);

    let fallback_root = temp_test_dir("move_dir_safely_with_fallback");
    let fallback_src = fallback_root.join("src_mod");
    let fallback_dest = fallback_root.join("dest_mod");
    fs::create_dir_all(&fallback_src).expect("create fallback src");

    let mut created = false;
    let mut copied = false;
    let mut removed = false;
    let mut rename_fail = |_: &Path, _: &Path| Err(io::Error::other("rename failed"));
    let mut create_dest = |path: &Path| {
        created = true;
        fs::create_dir_all(path)
    };
    let mut copy_into_dest = |src: &Path, dest: &Path| {
        copied = true;
        copy_dir_recursive(src, dest)
    };
    let mut remove_src = |path: &Path| {
        removed = true;
        fs::remove_dir_all(path)
    };
    move_dir_safely_with(
        &fallback_src,
        &fallback_dest,
        &mut rename_fail,
        &mut create_dest,
        &mut copy_into_dest,
        &mut remove_src,
    )
    .expect("fallback path should succeed");
    assert!(created);
    assert!(copied);
    assert!(removed);
    assert!(!fallback_src.exists());
    assert!(fallback_dest.exists());

    fs::remove_dir_all(root).expect("cleanup");
    fs::remove_dir_all(fallback_root).expect("cleanup");
}

#[test]
fn move_dir_safely_renames_when_destination_missing() {
    let root = temp_test_dir("move_dir_rename");
    let src = root.join("src_mod");
    let dest = root.join("dest_mod");

    fs::create_dir_all(&src).expect("failed to create src");
    write_file(&src.join("file.txt"), "content");

    move_dir_safely(&src, &dest).expect("move_dir_safely failed");

    assert!(!src.exists(), "source should be moved");
    assert!(dest.join("file.txt").exists());

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn resolve_conflict_replace_true_replaces_old_mod_and_moves_new() {
    let root = temp_test_dir("resolve_replace");
    let mods_path = root.join("mods");
    let temp_parent = root.join("staging/conflict_1");
    let temp_mod_path = temp_parent.join("NewMod");
    fs::create_dir_all(mods_path.join("OldMod")).expect("create old mod");
    fs::create_dir_all(&temp_mod_path).expect("create temp mod");
    write_file(&temp_mod_path.join("file.pak"), "data");

    resolve_conflict_in_paths(&mods_path, "NewMod", "OldMod", &temp_mod_path, true)
        .expect("replace should succeed");

    assert!(!mods_path.join("OldMod").exists());
    assert!(mods_path.join("NewMod/file.pak").exists());
    assert!(!temp_mod_path.exists());

    fs::remove_dir_all(root).expect("cleanup should succeed");
}

#[test]
fn move_dir_safely_reports_destination_create_errors() {
    let root = temp_test_dir("move_dir_dest_error");
    let src = root.join("src_mod");
    let bad_parent = root.join("not_a_dir");
    let dest = bad_parent.join("dest_mod");

    fs::create_dir_all(&src).expect("failed to create src");
    write_file(&src.join("file.txt"), "content");
    fs::write(&bad_parent, "file").expect("failed to create parent file");

    let err = move_dir_safely(&src, &dest).expect_err("expected create dir error");
    assert!(err.contains("Failed to create dest dir"));
    assert!(src.exists(), "source should remain when move fails");

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn move_dir_safely_with_propagates_copy_and_remove_errors() {
    let root = temp_test_dir("move_dir_safely_with_errors");
    let src = root.join("src_mod");
    let dest = root.join("dest_mod");
    fs::create_dir_all(&src).expect("create src");

    let mut rename_fail = |_: &Path, _: &Path| Err(io::Error::other("rename failed"));
    let mut create_dest = |path: &Path| fs::create_dir_all(path);
    let mut copy_fail = |_: &Path, _: &Path| Err("copy failed".to_string());
    let mut remove_ok = |_: &Path| Ok(());
    let err = move_dir_safely_with(
        &src,
        &dest,
        &mut rename_fail,
        &mut create_dest,
        &mut copy_fail,
        &mut remove_ok,
    )
    .expect_err("copy failure should bubble");
    assert_eq!(err, "copy failed");

    let mut rename_fail = |_: &Path, _: &Path| Err(io::Error::other("rename failed"));
    let mut create_dest = |path: &Path| fs::create_dir_all(path);
    let mut copy_ok = |src: &Path, dest: &Path| copy_dir_recursive(src, dest);
    let mut remove_fail = |_: &Path| Err(io::Error::other("remove failed"));
    let err = move_dir_safely_with(
        &src,
        &dest,
        &mut rename_fail,
        &mut create_dest,
        &mut copy_ok,
        &mut remove_fail,
    )
    .expect_err("remove failure should bubble");
    assert!(err.contains("Failed to remove source after copy"));

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn resolve_conflict_replace_false_removes_temp_mod_and_empty_parent() {
    let root = temp_test_dir("resolve_no_replace_cleanup");
    let mods_path = root.join("mods");
    let temp_parent = root.join("staging/conflict_1");
    let temp_mod_path = temp_parent.join("NewMod");

    fs::create_dir_all(&mods_path).expect("failed to create mods path");
    fs::create_dir_all(&temp_mod_path).expect("failed to create temp mod");
    write_file(&temp_mod_path.join("file.pak"), "data");

    resolve_conflict_in_paths(&mods_path, "NewMod", "OldMod", &temp_mod_path, false)
        .expect("expected temp cleanup success");

    assert!(!temp_mod_path.exists(), "temp mod folder should be removed");
    assert!(
        !temp_parent.exists(),
        "empty staging parent should be removed"
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn resolve_conflict_replace_true_reports_old_mod_removal_errors() {
    let root = temp_test_dir("resolve_old_mod_error");
    let mods_path = root.join("mods");
    let temp_parent = root.join("staging/conflict_1");
    let temp_mod_path = temp_parent.join("NewMod");

    fs::create_dir_all(&mods_path).expect("failed to create mods path");
    fs::write(mods_path.join("OldMod"), "file").expect("failed to create file old mod");
    fs::create_dir_all(&temp_mod_path).expect("failed to create temp mod");

    let err = resolve_conflict_in_paths(&mods_path, "NewMod", "OldMod", &temp_mod_path, true)
        .expect_err("expected old mod removal error");
    assert!(err.contains("Failed to remove old mod"));
    assert!(temp_mod_path.exists(), "temp mod should remain on failure");

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn resolve_conflict_replace_false_reports_temp_cleanup_errors() {
    let root = temp_test_dir("resolve_temp_cleanup_error");
    let mods_path = root.join("mods");
    let temp_parent = root.join("staging/conflict_1");
    let temp_mod_path = temp_parent.join("NewMod");

    fs::create_dir_all(&mods_path).expect("failed to create mods path");
    fs::create_dir_all(&temp_parent).expect("failed to create temp parent");
    fs::write(&temp_mod_path, "file").expect("failed to create temp file");

    let err = resolve_conflict_in_paths(&mods_path, "NewMod", "OldMod", &temp_mod_path, false)
        .expect_err("expected temp cleanup error");
    assert!(err.contains("Failed to cleanup temp mod folder"));
    assert!(
        temp_parent.exists(),
        "parent should remain on cleanup failure"
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn resolve_conflict_replace_true_handles_missing_old_mod_and_keeps_nonempty_parent() {
    let root = temp_test_dir("resolve_missing_old");
    let mods_path = root.join("mods");
    let temp_parent = root.join("staging/conflict_1");
    let temp_mod_path = temp_parent.join("NewMod");

    fs::create_dir_all(&mods_path).expect("failed to create mods path");
    fs::create_dir_all(&temp_mod_path).expect("failed to create temp mod");
    write_file(&temp_mod_path.join("file.pak"), "data");
    write_file(&temp_parent.join("keep.txt"), "keep");

    resolve_conflict_in_paths(&mods_path, "NewMod", "OldMod", &temp_mod_path, true)
        .expect("replace should succeed without old mod");

    assert!(mods_path.join("NewMod/file.pak").exists());
    assert!(
        temp_parent.exists(),
        "non-empty staging parent should not be removed"
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn resolve_conflict_replace_true_propagates_move_errors() {
    let root = temp_test_dir("resolve_move_error");
    let mods_path = root.join("mods");
    let temp_parent = root.join("staging/conflict_1");
    let temp_mod_path = temp_parent.join("NewMod");

    fs::create_dir_all(&mods_path).expect("create mods path");
    fs::write(mods_path.join("NewMod"), "conflicting file").expect("create conflicting file");
    fs::create_dir_all(&temp_mod_path).expect("create temp mod");
    write_file(&temp_mod_path.join("file.pak"), "data");

    let err = resolve_conflict_in_paths(&mods_path, "NewMod", "OldMod", &temp_mod_path, true)
        .expect_err("move failure should bubble");
    assert!(!err.is_empty());
    assert!(
        temp_mod_path.exists(),
        "temp mod should remain when move fails"
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn cleanup_empty_parent_dir_ignores_parentless_and_missing_parents() {
    cleanup_empty_parent_dir(&PathBuf::new());

    let root = temp_test_dir("cleanup_missing_parent");
    let missing_child = root.join("missing/child");
    cleanup_empty_parent_dir(&missing_child);
    assert!(!root.join("missing").exists());

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn cleanup_empty_parent_dir_ignores_non_directory_parent() {
    let root = temp_test_dir("cleanup_file_parent");
    let parent_file = root.join("not-a-dir");
    fs::write(&parent_file, "file").expect("failed to create parent file");

    cleanup_empty_parent_dir(&parent_file.join("child"));
    assert!(parent_file.exists(), "file parent should be left untouched");

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}
