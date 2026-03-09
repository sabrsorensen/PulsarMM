use super::*;
use crate::models::ProfileModEntry;
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_profiles_apply_ops_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn sample_entry(options: Option<Vec<&str>>) -> ProfileModEntry {
    ProfileModEntry {
        filename: "example.zip".to_string(),
        mod_id: Some("123".to_string()),
        file_id: Some("456".to_string()),
        version: Some("1.0".to_string()),
        installed_options: options.map(|v| v.into_iter().map(str::to_string).collect()),
    }
}

#[test]
fn selected_options_returns_slice_for_populated_entry() {
    let entry = sample_entry(Some(vec!["A", "B"]));
    assert_eq!(selected_options(&entry), ["A".to_string(), "B".to_string()]);
}

#[test]
#[should_panic(expected = "selected_options requires a non-empty installed_options list")]
fn selected_options_panics_when_options_are_missing() {
    let entry = sample_entry(None);
    let _ = selected_options(&entry);
}

#[test]
fn write_mod_info_is_best_effort_for_invalid_destinations() {
    let dir = temp_test_dir("write_mod_info");
    let dest = dir.join("not-a-dir");
    fs::write(&dest, "file").unwrap();

    write_mod_info(&dest, &sample_entry(None));

    assert!(dest.is_file());
    assert!(!dest.join("mod_info.json").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_mods_dir_removes_only_dirs_and_pak_files() {
    let dir = temp_test_dir("clear_mods");
    let mods_dir = dir.join("mods");
    fs::create_dir_all(mods_dir.join("FolderMod")).unwrap();
    fs::write(mods_dir.join("addon.pak"), "pak").unwrap();
    fs::write(mods_dir.join("keep.txt"), "keep").unwrap();

    clear_mods_dir(&mods_dir).unwrap();

    assert!(!mods_dir.join("FolderMod").exists());
    assert!(!mods_dir.join("addon.pak").exists());
    assert!(mods_dir.join("keep.txt").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_mods_dir_is_ok_when_target_missing() {
    let dir = temp_test_dir("clear_mods_missing");
    let missing = dir.join("does-not-exist");
    clear_mods_dir(&missing).expect("missing mods dir should be ignored");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn clear_mods_dir_errors_when_target_is_not_directory() {
    let dir = temp_test_dir("clear_mods_not_dir");
    let file_path = dir.join("mods-file");
    fs::write(&file_path, "not a directory").unwrap();
    let err = clear_mods_dir(&file_path).expect_err("file target should fail read_dir");
    assert!(
        !err.is_empty(),
        "expected non-empty filesystem error when read_dir fails"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn restore_or_create_live_mxml_copies_backup_or_writes_template() {
    let dir = temp_test_dir("mxml_restore");
    let backup = dir.join("source.mxml");
    let live = dir.join("live.mxml");

    fs::write(&backup, "<backup/>").unwrap();
    restore_or_create_live_mxml(&backup, &live).unwrap();
    assert_eq!(fs::read_to_string(&live).unwrap(), "<backup/>");

    fs::remove_file(&backup).unwrap();
    restore_or_create_live_mxml(&backup, &live).unwrap();
    assert_eq!(fs::read_to_string(&live).unwrap(), CLEAN_MXML_TEMPLATE);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn restore_or_create_live_mxml_surfaces_backup_open_and_live_write_errors() {
    let dir = temp_test_dir("mxml_restore_errors");
    let backup_dir = dir.join("backup-dir");
    fs::create_dir_all(&backup_dir).unwrap();

    let err = restore_or_create_live_mxml(&backup_dir, &dir.join("live.mxml"))
        .expect_err("directory backup should fail to open");
    assert!(!err.is_empty(), "expected non-empty backup open error");

    let missing_backup = dir.join("missing-source.mxml");
    let live_in_missing_parent = dir.join("missing").join("live.mxml");
    let err = restore_or_create_live_mxml(&missing_backup, &live_in_missing_parent)
        .expect_err("missing parent should fail template write");
    assert!(!err.is_empty(), "expected non-empty live write error");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn deploy_profile_entry_deploys_selected_options() {
    let dir = temp_test_dir("deploy_selected");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(library.join("A")).unwrap();
    fs::create_dir_all(library.join("B")).unwrap();
    fs::write(library.join("A").join("payload.pak"), "pak").unwrap();
    fs::write(library.join("B").join("payload2.pak"), "pak").unwrap();
    fs::create_dir_all(&mods).unwrap();

    let entry = sample_entry(Some(vec!["A"]));
    deploy_profile_entry(&entry, &library, &mods).unwrap();

    assert!(mods.join("A").join("payload.pak").exists());
    assert!(!mods.join("B").exists());
    let info = fs::read_to_string(mods.join("A").join("mod_info.json")).unwrap();
    let parsed: Value = serde_json::from_str(&info).unwrap();
    assert_eq!(parsed["modId"], "123");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn deploy_profile_entry_deploys_all_top_level_when_no_specific_options() {
    let dir = temp_test_dir("deploy_all");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(library.join("A")).unwrap();
    fs::create_dir_all(library.join("B")).unwrap();
    fs::write(library.join("A").join("a.pak"), "pak").unwrap();
    fs::write(library.join("B").join("b.pak"), "pak").unwrap();
    fs::create_dir_all(&mods).unwrap();

    let entry = sample_entry(None);
    deploy_profile_entry(&entry, &library, &mods).unwrap();

    assert!(mods.join("A").join("a.pak").exists());
    assert!(mods.join("B").join("b.pak").exists());
    assert!(mods.join("A").join("mod_info.json").exists());
    assert!(mods.join("B").join("mod_info.json").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn deploy_profile_entry_continues_when_one_top_level_item_fails() {
    let dir = temp_test_dir("deploy_all_partial_fail");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(library.join("A")).unwrap();
    fs::create_dir_all(library.join("B")).unwrap();
    fs::write(library.join("A").join("a.pak"), "pak").unwrap();
    fs::write(library.join("B").join("b.pak"), "pak").unwrap();
    fs::create_dir_all(&mods).unwrap();
    fs::write(mods.join("A"), "conflict-file").unwrap();

    let entry = sample_entry(None);
    deploy_profile_entry(&entry, &library, &mods).unwrap();

    assert!(mods.join("A").is_file(), "conflicting file should remain");
    assert!(!mods.join("A").join("mod_info.json").exists());
    assert!(mods.join("B").join("b.pak").exists());
    assert!(mods.join("B").join("mod_info.json").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn deploy_profile_entry_handles_missing_and_unusable_inputs() {
    let dir = temp_test_dir("deploy_missing_or_unusable");
    let mods = dir.join("mods");
    fs::create_dir_all(&mods).unwrap();
    let entry = sample_entry(Some(vec!["MissingOption"]));

    let missing_library = dir.join("missing-library");
    deploy_profile_entry(&entry, &missing_library, &mods)
        .expect("missing library should be ignored");

    let file_as_library = dir.join("not-a-dir");
    fs::write(&file_as_library, "file").unwrap();
    deploy_profile_entry(&sample_entry(None), &file_as_library, &mods)
        .expect("non-directory library should be ignored");

    let library = dir.join("library");
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(library.join("A")).unwrap();
    fs::write(library.join("A").join("a.pak"), "pak").unwrap();
    deploy_profile_entry(&entry, &library, &mods)
        .expect("missing selected options should be skipped");
    assert!(!mods.join("MissingOption").exists());
    assert!(!mods.join("A").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn deploy_profile_entry_continues_when_one_selected_option_fails() {
    let dir = temp_test_dir("deploy_selected_partial_fail");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(library.join("A")).unwrap();
    fs::create_dir_all(library.join("B")).unwrap();
    fs::write(library.join("A").join("a.pak"), "pak").unwrap();
    fs::write(library.join("B").join("b.pak"), "pak").unwrap();
    fs::create_dir_all(&mods).unwrap();

    let entry = sample_entry(Some(vec!["A", "B"]));
    fs::write(mods.join("A"), "not-a-dir").unwrap();

    deploy_profile_entry(&entry, &library, &mods).unwrap();

    assert!(
        !mods.join("A").join("a.pak").exists(),
        "failed selected option should be skipped"
    );
    assert!(mods.join("B").join("b.pak").exists());
    assert!(mods.join("B").join("mod_info.json").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn copy_profile_from_dir_copies_and_rewrites_name_and_template_fallback() {
    let dir = temp_test_dir("copy_profile");
    fs::write(
            dir.join("Source.json"),
            r#"{"name":"Source","mods":[{"filename":"a.zip","mod_id":null,"file_id":null,"version":null,"installed_options":[]}]}"#,
        )
        .unwrap();

    copy_profile_from_dir(&dir, "Source", "NewOne").unwrap();

    let json = fs::read_to_string(dir.join("NewOne.json")).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["name"], "NewOne");
    assert_eq!(
        fs::read_to_string(dir.join("NewOne.mxml")).unwrap(),
        CLEAN_MXML_TEMPLATE
    );

    assert!(copy_profile_from_dir(&dir, "Missing", "Another").is_err());
    assert!(copy_profile_from_dir(&dir, "Source", "NewOne").is_err());

    fs::write(dir.join("Bad.json"), "{ bad json").unwrap();
    let parse_err = copy_profile_from_dir(&dir, "Bad", "BadCopy")
        .expect_err("invalid source JSON should error");
    assert!(
        parse_err.contains("line"),
        "unexpected parse error text: {parse_err}"
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn copy_profile_from_dir_surfaces_json_copy_and_template_write_errors() {
    let dir = temp_test_dir("copy_profile_errors");
    fs::create_dir_all(dir.join("DirSource.json")).unwrap();

    let err = copy_profile_from_dir(&dir, "DirSource", "DirDest")
        .expect_err("directory source json should fail copy");
    assert!(
        err.contains("Failed to copy JSON"),
        "unexpected json copy error: {err}"
    );

    fs::write(
        dir.join("TemplateSource.json"),
        r#"{"name":"TemplateSource","mods":[]}"#,
    )
    .unwrap();
    fs::create_dir_all(dir.join("TemplateDest.mxml")).unwrap();

    let err = copy_profile_from_dir(&dir, "TemplateSource", "TemplateDest")
        .expect_err("directory destination mxml should fail template write");
    assert!(!err.is_empty(), "expected non-empty template write error");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn copy_profile_from_dir_copies_existing_mxml_and_surfaces_mxml_copy_error() {
    let dir = temp_test_dir("copy_profile_mxml");
    fs::write(
            dir.join("Source.json"),
            r#"{"name":"Source","mods":[{"filename":"a.zip","mod_id":null,"file_id":null,"version":null,"installed_options":[]}]}"#,
        )
        .unwrap();
    fs::write(dir.join("Source.mxml"), "<Data from source />").unwrap();

    copy_profile_from_dir(&dir, "Source", "WithMxml").expect("copy with mxml should succeed");
    assert_eq!(
        fs::read_to_string(dir.join("WithMxml.mxml")).unwrap(),
        "<Data from source />"
    );

    fs::write(
        dir.join("BrokenMxmlSource.json"),
        r#"{"name":"BrokenMxmlSource","mods":[]}"#,
    )
    .unwrap();
    fs::create_dir_all(dir.join("BrokenMxmlSource.mxml")).unwrap();
    let err = copy_profile_from_dir(&dir, "BrokenMxmlSource", "BrokenMxmlDest")
        .expect_err("directory mxml source should fail copy");
    assert!(
        err.contains("Failed to copy MXML"),
        "unexpected error: {err}"
    );

    fs::remove_dir_all(dir).unwrap();
}
