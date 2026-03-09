use super::*;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_profiles_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn collect_profile_names_sorts_and_filters_expected_files() {
    let dir = temp_test_dir("list");
    fs::write(dir.join("Zulu.json"), "{}").unwrap();
    fs::write(dir.join("alpha.json"), "{}").unwrap();
    fs::write(dir.join("Default.json"), "{}").unwrap();
    fs::write(dir.join("notes.txt"), "x").unwrap();

    let names = collect_profile_names_from_dir(&dir);
    assert_eq!(
        names,
        vec![
            "Default".to_string(),
            "Zulu".to_string(),
            "alpha".to_string()
        ]
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn create_and_read_profile_roundtrip_and_duplicate_guard() {
    let dir = temp_test_dir("create");
    create_empty_profile_in_dir(&dir, "MyProfile").unwrap();

    assert!(dir.join("MyProfile.json").exists());
    assert!(dir.join("MyProfile.mxml").exists());
    assert!(create_empty_profile_in_dir(&dir, "MyProfile").is_err());

    let mods = read_profile_mod_list_from_json_path(&dir.join("MyProfile.json")).unwrap();
    assert!(mods.is_empty());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn empty_profile_json_contains_name_and_empty_mod_list() {
    let json = empty_profile_json("Deck");
    let parsed: ModProfileData = serde_json::from_str(&json).expect("helper json should parse");
    assert_eq!(parsed.name, "Deck");
    assert!(parsed.mods.is_empty());
}

#[test]
fn rename_and_delete_profile_files_handle_missing_paths() {
    let dir = temp_test_dir("rename_delete");
    fs::write(dir.join("Old.json"), "{}").unwrap();
    fs::write(dir.join("Old.mxml"), "mxml").unwrap();

    rename_profile_files_in_dir(&dir, "Old", "New").unwrap();
    assert!(dir.join("New.json").exists());
    assert!(dir.join("New.mxml").exists());
    assert!(!dir.join("Old.json").exists());

    delete_profile_files_in_dir(&dir, "New").unwrap();
    assert!(!dir.join("New.json").exists());
    assert!(!dir.join("New.mxml").exists());

    delete_profile_files_in_dir(&dir, "DoesNotExist").unwrap();

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_profile_mod_list_parses_filenames_and_handles_missing_file() {
    let dir = temp_test_dir("mod_list");
    let json_path = dir.join("Example.json");

    assert_eq!(
        read_profile_mod_list_from_json_path(&json_path).unwrap(),
        Vec::<String>::new()
    );

    let content = r#"{
      "name": "Example",
      "mods": [
        {
          "filename": "a.zip",
          "mod_id": null,
          "file_id": null,
          "version": null,
          "installed_options": []
        },
        {
          "filename": "b.zip",
          "mod_id": "1",
          "file_id": "2",
          "version": "3",
          "installed_options": ["X"]
        }
      ]
    }"#;
    fs::write(&json_path, content).unwrap();

    let mods = read_profile_mod_list_from_json_path(&json_path).unwrap();
    assert_eq!(mods, vec!["a.zip".to_string(), "b.zip".to_string()]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn list_and_rename_helpers_handle_missing_inputs() {
    let dir = temp_test_dir("missing_inputs");

    let names = collect_profile_names_from_dir(&dir);
    assert_eq!(names, vec!["Default".to_string()]);

    rename_profile_files_in_dir(&dir, "Nope", "StillNope").unwrap();

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn collect_profile_names_returns_default_for_non_directory_inputs() {
    let dir = temp_test_dir("non_dir_input");
    let file = dir.join("profiles.txt");
    fs::write(&file, "not a directory").unwrap();

    let names = collect_profile_names_from_dir(&file);
    assert_eq!(names, vec!["Default".to_string()]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_profile_mod_list_returns_error_for_invalid_json() {
    let dir = temp_test_dir("invalid_json");
    let json_path = dir.join("Broken.json");
    fs::write(&json_path, "this is not json").unwrap();

    assert!(read_profile_mod_list_from_json_path(&json_path).is_err());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn create_empty_profile_writes_template_mxml() {
    let dir = temp_test_dir("mxml_template");
    create_empty_profile_in_dir(&dir, "Template").unwrap();

    let mxml = fs::read_to_string(dir.join("Template.mxml")).unwrap();
    assert!(!mxml.trim().is_empty());
    assert_eq!(mxml, CLEAN_MXML_TEMPLATE);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn get_profiles_dir_with_creates_profiles_folder_and_propagates_errors() {
    let base = temp_test_dir("profiles_dir_with");
    let root = base.join("Pulsar");

    let profiles = get_profiles_dir_with(|| Ok(root.clone())).expect("profiles dir");
    assert_eq!(profiles, root.join("profiles"));
    assert!(profiles.exists());

    let err = get_profiles_dir_with(|| Err("root-failed".to_string()))
        .expect_err("root error should bubble");
    assert_eq!(err, "root-failed");

    let blocked_root = base.join("blocked-root");
    fs::write(&blocked_root, "x").expect("blocked root file");
    let err =
        get_profiles_dir_with(|| Ok(blocked_root.clone())).expect_err("file root should fail");
    assert!(!err.is_empty());

    let file_root = base.join("file-root");
    let file_profiles_root = file_root.join("profiles");
    fs::create_dir_all(&file_root).expect("file root dir");
    fs::write(&file_profiles_root, "x").expect("profiles file");
    let err = get_profiles_dir_with(|| Ok(file_root.clone()))
        .expect_err("existing profiles file should fail");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn profile_storage_helpers_propagate_io_errors() {
    let base = temp_test_dir("storage_errors");

    let json_dir = base.join("DirOnly.json");
    fs::create_dir_all(&json_dir).expect("json dir");
    let err = delete_profile_files_in_dir(&base, "DirOnly")
        .expect_err("deleting directory via remove_file should fail");
    assert!(!err.is_empty());
    fs::remove_dir_all(&json_dir).expect("cleanup json dir");

    let old_json = base.join("Old.json");
    fs::write(&old_json, "{}").expect("old json");
    let new_json = base.join("New.json");
    fs::create_dir_all(&new_json).expect("new json dir");
    let err = rename_profile_files_in_dir(&base, "Old", "New")
        .expect_err("renaming file onto directory should fail");
    assert!(!err.is_empty());
    fs::remove_dir_all(&new_json).expect("cleanup new json dir");
    fs::remove_file(&old_json).expect("cleanup old json");

    let err = create_empty_profile_in_dir(&base, "nested/Profile")
        .expect_err("nested profile name without parent should fail");
    assert!(!err.is_empty());

    let read_dir = base.join("ReadAsDir.json");
    fs::create_dir_all(&read_dir).expect("read dir");
    let err = read_profile_mod_list_from_json_path(&read_dir)
        .expect_err("reading directory as file should fail");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}
