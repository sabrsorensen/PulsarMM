use super::engine;
use super::{
    apply_profile_command_entry_with, apply_profile_entries_with, apply_profile_impl_with,
    build_profile_data_from_entries, emit_entry_progress, library_folder_name_for_profile_entry,
    load_profile_json_content_with, maybe_backup_live_mxml_with, prepare_profile_entry_with,
    profile_entry_paths, profile_paths, profile_progress_payload,
    save_active_profile_command_entry_with, save_active_profile_impl_with, serialize_profile_data,
    should_extract_archive, write_profile_json_with, write_profile_snapshot_with, ApplyProfileDeps,
    SaveActiveProfileDeps,
};
use crate::models::{ModProfileData, ProfileModEntry, ProfileSwitchProgress};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_profiles_apply_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn empty_profile_maps_and_metadata(
    _mods_path: &Path,
) -> (
    HashMap<String, Vec<String>>,
    HashMap<String, engine::ModMetadata>,
) {
    (HashMap::new(), HashMap::new())
}

fn mod_settings_path(game_path: &Path) -> PathBuf {
    game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
}

fn ok_backup(_current: &Path, _backup: &Path) -> Result<(), String> {
    Ok(())
}

fn ok_snapshot(
    _name: &str,
    _json_path: &Path,
    _entries: Vec<ProfileModEntry>,
) -> Result<(), String> {
    Ok(())
}

fn load_empty_profile(
    name: &str,
    _exists: bool,
    _content: Option<&str>,
) -> Result<ModProfileData, String> {
    Ok(ModProfileData {
        name: name.to_string(),
        mods: Vec::new(),
    })
}

fn ok_clear_mods(_mods_dir: &Path) -> Result<(), String> {
    Ok(())
}

fn ok_restore_or_create_live_mxml(_backup: &Path, _live: &Path) -> Result<(), String> {
    Ok(())
}

fn dummy_downloads_dir() -> Result<PathBuf, String> {
    Ok(PathBuf::from("/tmp/pulsarmm_dummy_downloads"))
}

fn dummy_library_dir() -> Result<PathBuf, String> {
    Ok(PathBuf::from("/tmp/pulsarmm_dummy_library"))
}

fn ok_apply_entries(
    _profile_data: &ModProfileData,
    _downloads_dir: &Path,
    _library_dir: &Path,
    _mods_dir: &Path,
) -> Result<(), String> {
    Ok(())
}

#[test]
fn shared_profile_apply_test_helpers_are_callable() {
    let (profile_map, metadata) = empty_profile_maps_and_metadata(Path::new("/tmp/mods"));
    assert!(profile_map.is_empty());
    assert!(metadata.is_empty());
    assert!(
        mod_settings_path(Path::new("/tmp/game")).ends_with("Binaries/SETTINGS/GCMODSETTINGS.MXML")
    );
    ok_backup(Path::new("/tmp/a"), Path::new("/tmp/b")).expect("backup helper");
    ok_snapshot("Deck", Path::new("/tmp/Deck.json"), Vec::new()).expect("snapshot helper");

    let loaded = load_empty_profile("Deck", false, None).expect("load helper");
    assert_eq!(loaded.name, "Deck");
    assert!(loaded.mods.is_empty());

    ok_clear_mods(Path::new("/tmp/mods")).expect("clear helper");
    ok_restore_or_create_live_mxml(Path::new("/tmp/backup"), Path::new("/tmp/live"))
        .expect("restore helper");
    assert!(dummy_downloads_dir()
        .expect("downloads helper")
        .ends_with("pulsarmm_dummy_downloads"));
    assert!(dummy_library_dir()
        .expect("library helper")
        .ends_with("pulsarmm_dummy_library"));
    ok_apply_entries(
        &ModProfileData {
            name: "Deck".to_string(),
            mods: Vec::new(),
        },
        Path::new("/tmp/downloads"),
        Path::new("/tmp/library"),
        Path::new("/tmp/mods"),
    )
    .expect("apply helper");
}

#[test]
fn apply_wrapper_uses_expected_logic_contracts() {
    let dir = PathBuf::from("/tmp/profiles");
    let (json, mxml) = profile_paths(&dir, "Deck");
    assert_eq!(json, dir.join("Deck.json"));
    assert_eq!(mxml, dir.join("Deck.mxml"));

    assert_eq!(
        library_folder_name_for_profile_entry("MyMod.zip"),
        "MyMod.zip_unpacked"
    );

    let payload = profile_progress_payload(1, 3, "MyMod.zip".to_string(), 50);
    assert_eq!(payload.current, 1);
    assert_eq!(payload.total, 3);
    assert_eq!(payload.current_mod, "MyMod.zip");
    assert_eq!(payload.file_progress, 50);

    assert!(should_extract_archive(false, true));
    let data = build_profile_data_from_entries("Deck", vec![]);
    assert_eq!(data.name, "Deck");
}

#[test]
fn apply_helpers_cover_serialization_backup_and_io_seams() {
    let dir = temp_test_dir("helpers");
    let json_path = dir.join("Deck.json");
    let live = dir.join("live.mxml");
    let backup = dir.join("Deck.mxml");
    fs::write(&live, "<Data/>").unwrap();

    let copy_file =
        |src: &Path, dst: &Path| fs::copy(src, dst).map(|_| ()).map_err(|e| e.to_string());
    maybe_backup_live_mxml_with(&live, &backup, &copy_file).expect("backup should succeed");
    assert!(backup.exists());

    let data = ModProfileData {
        name: "Deck".to_string(),
        mods: Vec::new(),
    };
    let json = serialize_profile_data(&data);
    let write_file =
        |path: &Path, content: &str| fs::write(path, content).map_err(|e| e.to_string());
    write_profile_json_with(&json_path, &json, &write_file).expect("write json");
    assert!(json_path.exists());

    let read_file = |path: &Path| fs::read_to_string(path).map_err(|e| e.to_string());
    let loaded = load_profile_json_content_with(&json_path, &read_file)
        .expect("load json")
        .expect("json content expected");
    assert!(loaded.contains("\"name\": \"Deck\""));

    let missing_reader = |_path: &Path| Ok(String::new());
    let missing = load_profile_json_content_with(&dir.join("missing.json"), &missing_reader)
        .expect("missing ok");
    assert!(missing.is_none());

    let no_live = dir.join("no-live.mxml");
    let no_backup = dir.join("no-backup.mxml");
    let skipped_copy =
        |_src: &Path, _dst: &Path| Err("copy should not run when source is missing".to_string());
    maybe_backup_live_mxml_with(&no_live, &no_backup, &skipped_copy)
        .expect("missing live file should be ignored");
    assert!(!no_backup.exists());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn maybe_backup_live_mxml_with_propagates_copy_error() {
    let dir = temp_test_dir("backup_copy_error");
    let live = dir.join("live.mxml");
    let backup = dir.join("Deck.mxml");
    fs::write(&live, "<Data/>").unwrap();

    let copy_error = |_src: &Path, _dst: &Path| Err("copy-failed".to_string());
    let err = maybe_backup_live_mxml_with(&live, &backup, &copy_error)
        .expect_err("copy error should bubble");
    assert_eq!(err, "copy-failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn profile_entry_paths_build_expected_locations() {
    let (archive, library) = profile_entry_paths(
        std::path::Path::new("/downloads"),
        std::path::Path::new("/lib"),
        "Mod.zip",
    );
    assert_eq!(archive, PathBuf::from("/downloads/Mod.zip"));
    assert_eq!(library, PathBuf::from("/lib/Mod.zip_unpacked"));
}

#[test]
fn emit_entry_progress_builds_expected_payload() {
    let events = Mutex::new(Vec::new());

    emit_entry_progress(
        &mut |payload| events.lock().expect("events").push(payload),
        2,
        5,
        "A.zip",
        75,
    );

    let events = events.lock().expect("events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].current, 2);
    assert_eq!(events[0].total, 5);
    assert_eq!(events[0].current_mod, "A.zip");
    assert_eq!(events[0].file_progress, 75);
}

#[test]
fn prepare_profile_entry_with_covers_skip_extract_success_and_failure() {
    let dir = temp_test_dir("prepare_entry");
    let downloads = dir.join("downloads");
    let library = dir.join("library");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&library).unwrap();

    let entry = ProfileModEntry {
        filename: "A.zip".to_string(),
        mod_id: None,
        file_id: None,
        version: None,
        installed_options: None,
    };
    let archive_path = downloads.join("A.zip");
    let library_mod_path = library.join("A.zip_unpacked");
    let events = Mutex::new(Vec::new());

    let ready = prepare_profile_entry_with(
        &entry,
        &archive_path,
        &library_mod_path,
        1,
        2,
        &mut |payload| events.lock().expect("events").push(payload.file_progress),
        &mut |_archive, _library_mod_path, _progress_cb| {
            panic!("extract should not run when archive and library are both missing")
        },
    );
    assert!(!ready);
    assert_eq!(*events.lock().expect("events"), vec![0]);

    fs::write(&archive_path, "zip").unwrap();
    let ready = prepare_profile_entry_with(
        &entry,
        &archive_path,
        &library_mod_path,
        1,
        2,
        &mut |payload| events.lock().expect("events").push(payload.file_progress),
        &mut |_archive, library_mod_path, progress_cb| {
            fs::create_dir_all(library_mod_path).unwrap();
            progress_cb(40);
            Ok(())
        },
    );
    assert!(ready);
    assert_eq!(*events.lock().expect("events"), vec![0, 0, 40]);

    fs::remove_dir_all(&library_mod_path).unwrap();
    let ready = prepare_profile_entry_with(
        &entry,
        &archive_path,
        &library_mod_path,
        2,
        2,
        &mut |_payload| {},
        &mut |_archive, _library_mod_path, _progress_cb| Err("extract-failed".to_string()),
    );
    assert!(!ready);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn write_profile_snapshot_with_serializes_and_writes() {
    let dir = temp_test_dir("snapshot");
    let json_path = dir.join("Deck.json");
    let entries: Vec<ProfileModEntry> = Vec::new();
    let write_file =
        |path: &Path, content: &str| fs::write(path, content).map_err(|e| e.to_string());
    write_profile_snapshot_with("Deck", &json_path, entries, &write_file).expect("snapshot write");
    let saved = fs::read_to_string(&json_path).expect("json exists");
    assert!(saved.contains("\"name\": \"Deck\""));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_helpers_propagate_read_and_write_errors() {
    let dir = temp_test_dir("helpers_errors");
    let json_path = dir.join("Deck.json");
    fs::write(&json_path, "{ invalid json").unwrap();

    let write_fail = |_path: &Path, _content: &str| Err("write-failed".to_string());
    let err =
        write_profile_json_with(&json_path, "{}", &write_fail).expect_err("expected write error");
    assert_eq!(err, "write-failed");

    let read_fail = |_path: &Path| Err("read-failed".to_string());
    let err =
        load_profile_json_content_with(&json_path, &read_fail).expect_err("expected read error");
    assert_eq!(err, "read-failed");

    let snapshot_fail = |_path: &Path, _content: &str| Err("snapshot-write-failed".to_string());
    let err = write_profile_snapshot_with("Deck", &json_path, Vec::new(), &snapshot_fail)
        .expect_err("expected snapshot write error");
    assert_eq!(err, "snapshot-write-failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_entries_with_emits_progress_and_deploys_existing_library_mod() {
    let dir = temp_test_dir("entries_apply");
    let downloads = dir.join("downloads");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(&mods).unwrap();
    fs::write(downloads.join("A.zip"), b"zip-placeholder").unwrap();

    let entry = ProfileModEntry {
        filename: "A.zip".to_string(),
        mod_id: None,
        file_id: None,
        version: None,
        installed_options: None,
    };
    let data = ModProfileData {
        name: "Deck".to_string(),
        mods: vec![entry.clone()],
    };

    let events = Mutex::new(Vec::new());
    let deployed = Mutex::new(Vec::new());
    let mut emit_progress = |payload: ProfileSwitchProgress| {
        events
            .lock()
            .expect("lock events")
            .push(payload.file_progress)
    };
    let mut extract_archive =
        |_: &Path, library_mod_path: &Path, progress_cb: &mut dyn FnMut(u64)| {
            fs::create_dir_all(library_mod_path).unwrap();
            progress_cb(50);
            Ok(())
        };
    let mut deploy_entry = |entry: &ProfileModEntry, library_mod_path: &Path, mods_path: &Path| {
        deployed.lock().expect("lock deploy").push((
            entry.filename.clone(),
            library_mod_path.to_path_buf(),
            mods_path.to_path_buf(),
        ));
        Ok(())
    };
    apply_profile_entries_with(
        &data,
        &downloads,
        &library,
        &mods,
        &mut emit_progress,
        &mut extract_archive,
        &mut deploy_entry,
    )
    .expect("apply entries");

    assert_eq!(*events.lock().expect("events"), vec![0, 50, 100]);
    let deployed = deployed.lock().expect("deployed");
    assert_eq!(deployed.len(), 1);
    assert_eq!(deployed[0].0, "A.zip");
    assert!(deployed[0].1.ends_with("A.zip_unpacked"));
    assert_eq!(deployed[0].2, mods);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_entries_with_continues_when_extract_fails_and_library_missing() {
    let dir = temp_test_dir("entries_extract_fail");
    let downloads = dir.join("downloads");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(&mods).unwrap();
    fs::write(downloads.join("A.zip"), b"zip-placeholder").unwrap();

    let data = ModProfileData {
        name: "Deck".to_string(),
        mods: vec![ProfileModEntry {
            filename: "A.zip".to_string(),
            mod_id: None,
            file_id: None,
            version: None,
            installed_options: None,
        }],
    };

    let events = Mutex::new(Vec::new());
    let mut deploy_called = false;
    let mut emit_progress = |payload: ProfileSwitchProgress| {
        events
            .lock()
            .expect("lock events")
            .push(payload.file_progress)
    };
    let mut extract_archive =
        |_archive: &Path, _library_mod_path: &Path, _progress_cb: &mut dyn FnMut(u64)| {
            Err("extract failed".to_string())
        };
    let mut deploy_entry =
        |_entry: &ProfileModEntry, _library_mod_path: &Path, _mods_path: &Path| {
            deploy_called = true;
            Ok(())
        };
    apply_profile_entries_with(
        &data,
        &downloads,
        &library,
        &mods,
        &mut emit_progress,
        &mut extract_archive,
        &mut deploy_entry,
    )
    .expect("extract failures should be skipped");

    assert_eq!(*events.lock().expect("events"), vec![0]);
    assert!(!deploy_called, "deploy should not run if extraction failed");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_entries_with_propagates_deploy_errors() {
    let dir = temp_test_dir("entries_deploy_fail");
    let downloads = dir.join("downloads");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(&mods).unwrap();
    fs::write(downloads.join("A.zip"), b"zip-placeholder").unwrap();

    let data = ModProfileData {
        name: "Deck".to_string(),
        mods: vec![ProfileModEntry {
            filename: "A.zip".to_string(),
            mod_id: None,
            file_id: None,
            version: None,
            installed_options: None,
        }],
    };

    let mut emit_progress = |_payload| {};
    let mut extract_archive =
        |_archive: &Path, library_mod_path: &Path, _progress_cb: &mut dyn FnMut(u64)| {
            fs::create_dir_all(library_mod_path).unwrap();
            Ok(())
        };
    let mut deploy_entry =
        |_entry: &ProfileModEntry, _library_mod_path: &Path, _mods_path: &Path| {
            Err("deploy failed".to_string())
        };
    let err = apply_profile_entries_with(
        &data,
        &downloads,
        &library,
        &mods,
        &mut emit_progress,
        &mut extract_archive,
        &mut deploy_entry,
    )
    .expect_err("deploy error should bubble");
    assert_eq!(err, "deploy failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_entries_with_skips_when_no_extract_and_library_missing() {
    let dir = temp_test_dir("entries_skip_missing_library");
    let downloads = dir.join("downloads");
    let library = dir.join("library");
    let mods = dir.join("mods");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(&mods).unwrap();

    let data = ModProfileData {
        name: "Deck".to_string(),
        mods: vec![ProfileModEntry {
            filename: "A.zip".to_string(),
            mod_id: None,
            file_id: None,
            version: None,
            installed_options: None,
        }],
    };

    let events = Mutex::new(Vec::new());
    let mut extract_called = false;
    let mut deploy_called = false;
    let mut emit_progress = |payload: ProfileSwitchProgress| {
        events
            .lock()
            .expect("events lock")
            .push(payload.file_progress)
    };
    let mut extract_archive =
        |_archive: &Path, _library_mod_path: &Path, _progress_cb: &mut dyn FnMut(u64)| {
            extract_called = true;
            Ok(())
        };
    let mut deploy_entry =
        |_entry: &ProfileModEntry, _library_mod_path: &Path, _mods_path: &Path| {
            deploy_called = true;
            Ok(())
        };
    apply_profile_entries_with(
        &data,
        &downloads,
        &library,
        &mods,
        &mut emit_progress,
        &mut extract_archive,
        &mut deploy_entry,
    )
    .expect("missing archive+library should be skipped");

    assert!(
        !extract_called,
        "extract should not run when archive missing"
    );
    assert!(!deploy_called, "deploy should not run when library missing");
    assert_eq!(*events.lock().expect("events lock"), vec![0]);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_active_profile_impl_with_covers_game_and_no_game_paths() {
    let dir = temp_test_dir("save_impl_with");
    let game = dir.join("game");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(game.join("Binaries/SETTINGS")).unwrap();
    fs::write(game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"), "<Data/>").unwrap();

    let save_with_game = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        find_game_path: Box::new(|| Some(game.clone())),
        collect_profile_map_and_metadata: Box::new(|_mods_path| (HashMap::new(), HashMap::new())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        maybe_backup_live_mxml: Box::new(|current, backup| {
            fs::copy(current, backup)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(|name, json_path, entries| {
            let write_file =
                |path: &Path, content: &str| fs::write(path, content).map_err(|e| e.to_string());
            write_profile_snapshot_with(name, json_path, entries, &write_file)
        }),
    };
    save_active_profile_impl_with("Deck", &save_with_game)
        .expect("save with game path should succeed");
    assert!(dir.join("Deck.json").exists());
    assert!(dir.join("Deck.mxml").exists());

    let save_without_game = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        find_game_path: Box::new(|| None),
        collect_profile_map_and_metadata: Box::new(|_mods_path| (HashMap::new(), HashMap::new())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        maybe_backup_live_mxml: Box::new(|_current, _backup| Ok(())),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(|name, json_path, entries| {
            let write_file =
                |path: &Path, content: &str| fs::write(path, content).map_err(|e| e.to_string());
            write_profile_snapshot_with(name, json_path, entries, &write_file)
        }),
    };
    save_active_profile_impl_with("DeckNoGame", &save_without_game)
        .expect("save without game path should still write snapshot");
    assert!(dir.join("DeckNoGame.json").exists());

    let save_err = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Err("profiles-dir-failed".to_string())),
        find_game_path: Box::new(|| None),
        collect_profile_map_and_metadata: Box::new(empty_profile_maps_and_metadata),
        mod_settings_file: Box::new(mod_settings_path),
        maybe_backup_live_mxml: Box::new(ok_backup),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(ok_snapshot),
    };
    let err = save_active_profile_impl_with("DeckErr", &save_err)
        .err()
        .expect("get profiles dir error should bubble");
    assert_eq!(err, "profiles-dir-failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_impl_with_covers_success_and_errors() {
    let dir = temp_test_dir("apply_impl_with");
    let game = dir.join("game");
    let mods = game.join("GAMEDATA/MODS");
    fs::create_dir_all(&mods).unwrap();
    let backup_mxml = dir.join("Deck.mxml");
    fs::write(&backup_mxml, "<Data/>").unwrap();
    let profile_json = dir.join("Deck.json");
    fs::write(&profile_json, r#"{"name":"Deck","mods":[]}"#).unwrap();

    let mut apply_success = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(|_mods_dir| Ok(())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        restore_or_create_live_mxml: Box::new(|_backup, _live| Ok(())),
        get_downloads_dir: Box::new(|| Ok(dir.join("downloads"))),
        get_library_dir: Box::new(|| Ok(dir.join("library"))),
        apply_profile_entries: Box::new(|profile_data, downloads_dir, library_dir, mods_dir| {
            assert_eq!(profile_data.name, "Deck");
            assert!(downloads_dir.ends_with("downloads"));
            assert!(library_dir.ends_with("library"));
            assert!(mods_dir.ends_with("GAMEDATA/MODS"));
            Ok(())
        }),
    };
    apply_profile_impl_with("Deck", &mut apply_success).expect("apply impl wrapper success");

    let mut missing_game = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| None),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let err = apply_profile_impl_with("Deck", &mut missing_game)
        .err()
        .expect("missing game path should error");
    assert_eq!(err, "Game path not found");

    let mut load_error = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|_name, _exists, _content| {
            Err("load-profile-failed".to_string())
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let err = apply_profile_impl_with("Deck", &mut load_error)
        .err()
        .expect("load profile error should bubble");
    assert_eq!(err, "load-profile-failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_active_profile_impl_with_propagates_backup_and_snapshot_errors() {
    let dir = temp_test_dir("save_impl_errors");
    let game = dir.join("game");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(game.join("Binaries/SETTINGS")).unwrap();
    fs::write(game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"), "<Data/>").unwrap();

    let backup_deps = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        find_game_path: Box::new(|| Some(game.clone())),
        collect_profile_map_and_metadata: Box::new(|_mods_path| (HashMap::new(), HashMap::new())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        maybe_backup_live_mxml: Box::new(|_current, _backup| Err("backup-failed".to_string())),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(|_name, _json_path, _entries| Ok(())),
    };
    let backup_err = save_active_profile_impl_with("Deck", &backup_deps)
        .expect_err("backup error should bubble");
    assert_eq!(backup_err, "backup-failed");

    let snapshot_deps = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        find_game_path: Box::new(|| Some(game.clone())),
        collect_profile_map_and_metadata: Box::new(|_mods_path| (HashMap::new(), HashMap::new())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        maybe_backup_live_mxml: Box::new(|_current, _backup| Ok(())),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(|_name, _json_path, _entries| {
            Err("snapshot-failed".to_string())
        }),
    };
    let snapshot_err = save_active_profile_impl_with("Deck", &snapshot_deps)
        .expect_err("snapshot error should bubble");
    assert_eq!(snapshot_err, "snapshot-failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_active_profile_impl_with_propagates_profiles_dir_error() {
    let deps = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Err("profiles-dir-failed".to_string())),
        find_game_path: Box::new(|| None),
        collect_profile_map_and_metadata: Box::new(empty_profile_maps_and_metadata),
        mod_settings_file: Box::new(mod_settings_path),
        maybe_backup_live_mxml: Box::new(ok_backup),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(ok_snapshot),
    };
    let err =
        save_active_profile_impl_with("Deck", &deps).expect_err("profiles dir error should bubble");
    assert_eq!(err, "profiles-dir-failed");
}

#[test]
fn apply_profile_impl_with_propagates_stage_specific_errors() {
    let dir = temp_test_dir("apply_impl_errors");
    let game = dir.join("game");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();

    let mut json_read_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| Err("json-read-failed".to_string())),
        load_profile_for_apply: Box::new(load_empty_profile),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let json_read_err = apply_profile_impl_with("Deck", &mut json_read_deps)
        .expect_err("json read error should bubble");
    assert_eq!(json_read_err, "json-read-failed");

    let mut clear_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(|_mods_dir| Err("clear-mods-failed".to_string())),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let clear_err = apply_profile_impl_with("Deck", &mut clear_deps)
        .expect_err("clear mods error should bubble");
    assert_eq!(clear_err, "clear-mods-failed");

    let mut restore_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(|_backup, _live| {
            Err("restore-mxml-failed".to_string())
        }),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let restore_err = apply_profile_impl_with("Deck", &mut restore_deps)
        .expect_err("restore mxml error should bubble");
    assert_eq!(restore_err, "restore-mxml-failed");

    let mut downloads_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(|| Err("downloads-dir-failed".to_string())),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let downloads_err = apply_profile_impl_with("Deck", &mut downloads_deps)
        .expect_err("downloads dir error should bubble");
    assert_eq!(downloads_err, "downloads-dir-failed");

    let mut library_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(|| Err("library-dir-failed".to_string())),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let library_err = apply_profile_impl_with("Deck", &mut library_deps)
        .expect_err("library dir error should bubble");
    assert_eq!(library_err, "library-dir-failed");

    let mut apply_error_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(
            |_profile_data, _downloads_dir, _library_dir, _mods_dir| {
                Err("apply-entries-failed".to_string())
            },
        ),
    };
    let apply_err = apply_profile_impl_with("Deck", &mut apply_error_deps)
        .expect_err("apply entries error should bubble");
    assert_eq!(apply_err, "apply-entries-failed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_impl_with_propagates_profiles_dir_error() {
    let mut deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Err("profiles-dir-failed".to_string())),
        load_profile_json_content: Box::new(|_json_path| Ok(None)),
        load_profile_for_apply: Box::new(load_empty_profile),
        find_game_path: Box::new(|| Some(PathBuf::from("/tmp/game"))),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let err =
        apply_profile_impl_with("Deck", &mut deps).expect_err("profiles dir error should bubble");
    assert_eq!(err, "profiles-dir-failed");
}

#[test]
fn apply_profile_impl_with_propagates_profile_load_and_missing_game_errors() {
    let dir = temp_test_dir("apply_impl_load_and_game_errors");
    let game = dir.join("game");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();

    let mut load_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|_name, _exists, _content| {
            Err("load-profile-failed".to_string())
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let load_err = apply_profile_impl_with("Deck", &mut load_deps)
        .expect_err("profile load error should bubble");
    assert_eq!(load_err, "load-profile-failed");

    let mut missing_game_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"Deck","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| None),
        clear_mods_dir: Box::new(ok_clear_mods),
        mod_settings_file: Box::new(mod_settings_path),
        restore_or_create_live_mxml: Box::new(ok_restore_or_create_live_mxml),
        get_downloads_dir: Box::new(dummy_downloads_dir),
        get_library_dir: Box::new(dummy_library_dir),
        apply_profile_entries: Box::new(ok_apply_entries),
    };
    let missing_game_err = apply_profile_impl_with("Deck", &mut missing_game_deps)
        .expect_err("missing game path should bubble");
    assert_eq!(missing_game_err, "Game path not found");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn profile_command_entry_helpers_forward_dependencies() {
    let dir = temp_test_dir("command_entry_helpers");
    let game = dir.join("game");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(game.join("Binaries/SETTINGS")).unwrap();
    fs::write(game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"), "<Data/>").unwrap();

    let save_deps = SaveActiveProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        find_game_path: Box::new(|| Some(game.clone())),
        collect_profile_map_and_metadata: Box::new(|_mods_path| (HashMap::new(), HashMap::new())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        maybe_backup_live_mxml: Box::new(|_current, _backup| Ok(())),
        build_profile_entries: Box::new(engine::build_profile_entries),
        write_profile_snapshot: Box::new(|name, json_path, entries| {
            let write_file =
                |path: &Path, content: &str| fs::write(path, content).map_err(|e| e.to_string());
            write_profile_snapshot_with(name, json_path, entries, &write_file)
        }),
    };
    save_active_profile_command_entry_with("DeckEntry", &save_deps)
        .expect("save command entry should succeed");
    assert!(dir.join("DeckEntry.json").exists());

    let profile_json = dir.join("DeckEntryApply.json");
    fs::write(&profile_json, r#"{"name":"DeckEntryApply","mods":[]}"#).unwrap();
    let mut apply_deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| {
            Ok(Some(r#"{"name":"DeckEntryApply","mods":[]}"#.to_string()))
        }),
        load_profile_for_apply: Box::new(|name, _exists, content| {
            engine::load_profile_for_apply(name, true, content)
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(|_mods_dir| Ok(())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        restore_or_create_live_mxml: Box::new(|_backup, _live| Ok(())),
        get_downloads_dir: Box::new(|| Ok(dir.join("downloads"))),
        get_library_dir: Box::new(|| Ok(dir.join("library"))),
        apply_profile_entries: Box::new(|profile_data, _downloads_dir, _library_dir, mods_dir| {
            assert_eq!(profile_data.name, "DeckEntryApply");
            assert!(mods_dir.ends_with("GAMEDATA/MODS"));
            Ok(())
        }),
    };
    apply_profile_command_entry_with("DeckEntryApply", &mut apply_deps)
        .expect("apply command entry should succeed");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn apply_profile_impl_with_passes_profile_exists_and_optional_content() {
    let dir = temp_test_dir("apply_impl_flags");
    let game = dir.join("game");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();

    let captured = Mutex::new(Vec::<(bool, Option<String>)>::new());
    let mut deps = ApplyProfileDeps {
        get_profiles_dir: Box::new(|| Ok(dir.clone())),
        load_profile_json_content: Box::new(|_json_path| Ok(None)),
        load_profile_for_apply: Box::new(|_name, exists, content| {
            captured
                .lock()
                .expect("captured lock")
                .push((exists, content.map(|s| s.to_string())));
            Ok(ModProfileData {
                name: "DeckMissingFile".to_string(),
                mods: Vec::new(),
            })
        }),
        find_game_path: Box::new(|| Some(game.clone())),
        clear_mods_dir: Box::new(|_mods_dir| Ok(())),
        mod_settings_file: Box::new(|game_path| {
            game_path.join("Binaries/SETTINGS/GCMODSETTINGS.MXML")
        }),
        restore_or_create_live_mxml: Box::new(|_backup, _live| Ok(())),
        get_downloads_dir: Box::new(|| Ok(dir.join("downloads"))),
        get_library_dir: Box::new(|| Ok(dir.join("library"))),
        apply_profile_entries: Box::new(
            |_profile_data, _downloads_dir, _library_dir, _mods_dir| Ok(()),
        ),
    };
    apply_profile_impl_with("DeckMissingFile", &mut deps)
        .expect("apply should succeed with missing profile file content");
    assert_eq!(
        *captured.lock().expect("captured lock"),
        vec![(false, None)]
    );

    fs::remove_dir_all(&dir).unwrap();
}
