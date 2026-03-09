use super::{
    library_rename_paths, maybe_remove_mod_folder, mod_folder_path, mods_root_from_game_path,
    settings_file_from_game_path, sync_library_folder_rename, validate_rename_paths,
    LibraryRenameSync,
};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_mod_command_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn path_builders_append_expected_segments() {
    let base = Path::new("/tmp/NMS");
    assert_eq!(
        mods_root_from_game_path(base),
        PathBuf::from("/tmp/NMS/GAMEDATA/MODS")
    );
    assert_eq!(
        mod_folder_path(base, "MyMod"),
        PathBuf::from("/tmp/NMS/GAMEDATA/MODS/MyMod")
    );
    let settings = settings_file_from_game_path(base);
    assert!(settings.ends_with("Binaries/SETTINGS/GCMODSETTINGS.MXML"));
}

#[test]
fn validate_rename_paths_covers_errors_and_success() {
    assert_eq!(
        validate_rename_paths(false, false).expect_err("expected missing source"),
        "Original mod folder not found."
    );
    assert_eq!(
        validate_rename_paths(true, true).expect_err("expected duplicate target"),
        "A mod with the new name already exists."
    );
    validate_rename_paths(true, false).expect("expected valid rename paths");
}

#[test]
fn maybe_remove_mod_folder_removes_existing_and_reports_missing() {
    let dir = temp_test_dir("remove_mod");
    let mod_dir = dir.join("ModA");
    fs::create_dir_all(&mod_dir).expect("create mod dir");

    assert!(maybe_remove_mod_folder(&mod_dir, "ModA").expect("remove should succeed"));
    assert!(!mod_dir.exists());
    assert!(!maybe_remove_mod_folder(&mod_dir, "ModA").expect("missing should be ok"));

    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn maybe_remove_mod_folder_errors_for_non_directory_path() {
    let dir = temp_test_dir("remove_mod_error");
    let mod_file = dir.join("ModA");
    fs::write(&mod_file, "not a dir").expect("create file");

    let err = maybe_remove_mod_folder(&mod_file, "ModA").expect_err("file target should fail");
    assert!(err.contains("Failed to delete mod folder 'ModA'"));

    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn library_rename_helpers_build_paths_and_decide_sync() {
    let (old_path, new_path) =
        library_rename_paths(Path::new("/lib"), "source.zip", "OldMod", "NewMod");
    assert!(old_path.ends_with("source.zip_unpacked/OldMod"));
    assert!(new_path.ends_with("source.zip_unpacked/NewMod"));
}

#[test]
fn sync_library_folder_rename_covers_missing_conflict_and_success() {
    let dir = temp_test_dir("sync_rename");
    let unpacked = dir.join("archive.zip_unpacked");
    fs::create_dir_all(&unpacked).expect("create unpacked");

    let missing =
        sync_library_folder_rename(&dir, "archive.zip", "Old", "New").expect("missing old");
    assert_eq!(missing, LibraryRenameSync::SourceMissing);

    let old_dir = unpacked.join("Old");
    let new_dir = unpacked.join("New");
    fs::create_dir_all(&old_dir).expect("create old");
    fs::create_dir_all(&new_dir).expect("create new");
    let conflict =
        sync_library_folder_rename(&dir, "archive.zip", "Old", "New").expect("target exists");
    assert_eq!(conflict, LibraryRenameSync::TargetExists);
    assert!(old_dir.exists());
    assert!(new_dir.exists());

    fs::remove_dir_all(&new_dir).expect("remove conflict dir");
    let renamed =
        sync_library_folder_rename(&dir, "archive.zip", "Old", "New").expect("rename should work");
    assert_eq!(renamed, LibraryRenameSync::Renamed);
    assert!(!old_dir.exists());
    assert!(new_dir.exists());

    fs::remove_dir_all(dir).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn sync_library_folder_rename_propagates_rename_error() {
    let dir = temp_test_dir("sync_rename_error");
    let unpacked = dir.join("archive.zip_unpacked");
    let old_dir = unpacked.join("Old");
    fs::create_dir_all(&old_dir).expect("create old");

    let original_perms = fs::metadata(&unpacked).expect("metadata").permissions();
    fs::set_permissions(&unpacked, Permissions::from_mode(0o500)).expect("chmod unpacked");

    let err = sync_library_folder_rename(&dir, "archive.zip", "Old", "New")
        .expect_err("rename should fail without parent write permission");
    assert!(err.contains("Failed to sync rename to Library:"));

    fs::set_permissions(&unpacked, original_perms).expect("restore perms");
    fs::remove_dir_all(dir).expect("cleanup");
}
