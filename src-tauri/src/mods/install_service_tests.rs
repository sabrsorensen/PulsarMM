use super::{
    create_staging_dir, ensure_existing_source_root, finalize_installation_with,
    get_all_mods_for_render_with, get_staging_dir_with, io_error_to_string,
    missing_game_installation_path_error, missing_game_path_error, parse_settings_xml,
    read_mod_id_for_candidate, read_settings_xml, select_install_items,
};
use crate::models::{ModEntry, ModProperty, SettingsData, TopLevelProperty};
use crate::mods::settings_store::{load_settings_file, save_settings_file};
use crate::services::runtime::AppRuntime;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct FakeRuntime {
    game_path: Option<PathBuf>,
    library_dir: Option<PathBuf>,
    pulsar_root: Option<PathBuf>,
    logs: Mutex<Vec<String>>,
}

impl AppRuntime for FakeRuntime {
    fn find_game_path(&self) -> Option<PathBuf> {
        self.game_path.clone()
    }
    fn get_library_dir(&self) -> Result<PathBuf, String> {
        self.library_dir
            .clone()
            .ok_or_else(|| "missing library_dir".to_string())
    }
    fn get_pulsar_root(&self) -> Result<PathBuf, String> {
        self.pulsar_root
            .clone()
            .ok_or_else(|| "missing pulsar_root".to_string())
    }
    fn log(&self, level: &str, message: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("[{}] {}", level, message));
    }
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_install_service_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn mk_mod_entry(name: &str, enabled: &str, priority: &str) -> ModEntry {
    ModEntry {
        entry_name: "Mod".to_string(),
        entry_value: "".to_string(),
        index: priority.to_string(),
        properties: vec![
            ModProperty {
                name: "Name".to_string(),
                value: Some(name.to_string()),
            },
            ModProperty {
                name: "Enabled".to_string(),
                value: Some(enabled.to_string()),
            },
            ModProperty {
                name: "ModPriority".to_string(),
                value: Some(priority.to_string()),
            },
        ],
    }
}

#[test]
fn install_service_helper_messages_and_io_conversion_are_stable() {
    assert_eq!(
        missing_game_installation_path_error(),
        "Could not find game installation path."
    );
    assert_eq!(missing_game_path_error(), "Could not find game path.");
    assert_eq!(io_error_to_string(std::io::Error::other("boom")), "boom");
}

#[test]
fn install_service_helpers_cover_staging_source_selection_and_mod_id_lookup() {
    let root = temp_test_dir("helpers");
    let staging = create_staging_dir(&root).expect("staging should be created");
    assert_eq!(staging, root.join("staging"));

    let library = root.join("library");
    fs::create_dir_all(library.join("Pack")).unwrap();

    let runtime = FakeRuntime::default();
    let source_root =
        ensure_existing_source_root(&runtime, &library, "Pack").expect("source should exist");
    assert_eq!(source_root, library.join("Pack"));

    let err = ensure_existing_source_root(&runtime, &library, "Missing")
        .expect_err("missing source should error");
    assert!(err.contains("Library folder missing"));

    let selected =
        select_install_items(&runtime, &library.join("Pack"), &[]).expect("scan-all should work");
    assert!(selected.is_empty());

    let selected = select_install_items(&runtime, &library.join("Pack"), &["Chosen".to_string()])
        .expect("explicit selection should work");
    assert_eq!(selected, vec![PathBuf::from("Chosen")]);

    let mod_dir = root.join("mod");
    fs::create_dir_all(&mod_dir).unwrap();
    fs::write(mod_dir.join("mod_info.json"), r#"{"modId":"abc"}"#).unwrap();
    assert_eq!(read_mod_id_for_candidate(&mod_dir), Some("abc".to_string()));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_service_helpers_cover_read_and_parse_settings_paths() {
    let root = temp_test_dir("helpers_settings");
    let settings_file = root.join("GCMODSETTINGS.MXML");
    fs::write(&settings_file, "<Data/>").unwrap();

    let runtime = FakeRuntime::default();
    let xml = read_settings_xml(&runtime, &settings_file).expect("read should succeed");
    assert!(xml.contains("<Data/>"));

    let parse_err = parse_settings_xml(&runtime, "not xml").expect_err("parse should fail");
    assert!(parse_err.contains("Failed to parse GCMODSETTINGS.MXML"));

    let parsed = parse_settings_xml(
        &runtime,
        r#"<?xml version="1.0" encoding="utf-8"?><Data template="GcModSettings"><Property name="DisableAllMods" value="false" /><Property name="Data" /></Data>"#,
    )
    .expect("parse should succeed");
    assert_eq!(parsed.template, "GcModSettings");

    let missing_err =
        read_settings_xml(&runtime, &root.join("missing.mxml")).expect_err("read should fail");
    assert!(missing_err.contains("Failed to read GCMODSETTINGS.MXML"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn staging_dir_is_created_under_runtime_root() {
    let root = temp_test_dir("staging");
    let runtime = FakeRuntime {
        pulsar_root: Some(root.clone()),
        ..Default::default()
    };

    let staging = get_staging_dir_with(&runtime).expect("expected staging path");
    assert_eq!(staging, root.join("staging"));
    assert!(staging.exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn staging_dir_propagates_create_error_when_runtime_root_is_not_directory() {
    let base = temp_test_dir("staging_error");
    let blocked_root = base.join("blocked-root");
    fs::write(&blocked_root, "not a directory").unwrap();

    let runtime = FakeRuntime {
        pulsar_root: Some(blocked_root),
        ..Default::default()
    };

    let err = get_staging_dir_with(&runtime).expect_err("expected staging create error");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn staging_dir_errors_when_existing_staging_path_is_file() {
    let base = temp_test_dir("staging_file");
    let root = base.join("pulsar");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("staging"), "not a directory").unwrap();

    let runtime = FakeRuntime {
        pulsar_root: Some(root),
        ..Default::default()
    };

    let err = get_staging_dir_with(&runtime).expect_err("staging file should fail");
    assert!(!err.is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn mods_for_render_returns_empty_when_settings_file_missing() {
    let root = temp_test_dir("render_missing_settings");
    let game = root.join("nms");
    fs::create_dir_all(game.join("Binaries")).unwrap();
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game.clone()),
        ..Default::default()
    };

    let mods = get_all_mods_for_render_with(&runtime).expect("expected success");
    assert!(mods.is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mods_for_render_errors_when_game_path_missing() {
    let runtime = FakeRuntime::default();
    let result = get_all_mods_for_render_with(&runtime);
    assert!(result.is_err(), "expected missing game path error");
    if let Err(err) = result {
        assert_eq!(err, "Could not find game installation path.");
    }
}

#[test]
fn mods_for_render_errors_when_settings_path_is_not_readable_file() {
    let root = temp_test_dir("render_settings_unreadable");
    let game = root.join("nms");
    let settings_file = game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(&settings_file).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game),
        ..Default::default()
    };

    let result = get_all_mods_for_render_with(&runtime);
    assert!(result.is_err(), "expected read error");
    if let Err(err) = result {
        assert!(err.contains("Failed to read GCMODSETTINGS.MXML"));
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mods_for_render_errors_on_invalid_settings_xml() {
    let root = temp_test_dir("render_bad_xml");
    let game = root.join("nms");
    fs::create_dir_all(game.join("Binaries/SETTINGS")).unwrap();
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::write(game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML"), "not xml").unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game),
        ..Default::default()
    };

    let result = get_all_mods_for_render_with(&runtime);
    assert!(result.is_err(), "expected parse error");
    if let Err(err) = result {
        assert!(err.contains("Failed to parse GCMODSETTINGS.MXML"));
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mods_for_render_parses_valid_settings_cleans_orphans_and_returns_renderable_mods() {
    let root = temp_test_dir("render_valid_cleanup");
    let game = root.join("nms");
    let mods_dir = game.join("GAMEDATA/MODS");
    let settings_file = game.join("Binaries/SETTINGS/GCMODSETTINGS.MXML");

    fs::create_dir_all(&mods_dir).unwrap();
    fs::create_dir_all(settings_file.parent().expect("settings parent")).unwrap();
    fs::create_dir_all(mods_dir.join("KeepMe")).unwrap();
    fs::write(mods_dir.join("KeepMe/mod_info.json"), r#"{"id":"keep-id"}"#).unwrap();

    let root_settings = SettingsData {
        template: "GcModSettings".to_string(),
        properties: vec![
            TopLevelProperty {
                name: "DisableAllMods".to_string(),
                value: Some("false".to_string()),
                mods: vec![],
            },
            TopLevelProperty {
                name: "Data".to_string(),
                value: None,
                mods: vec![
                    mk_mod_entry("KeepMe", "true", "9"),
                    mk_mod_entry("DropMe", "false", "10"),
                ],
            },
        ],
    };
    save_settings_file(&settings_file, &root_settings).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game.clone()),
        ..Default::default()
    };

    let rendered = get_all_mods_for_render_with(&runtime).expect("expected render success");
    assert_eq!(rendered.len(), 1);
    assert_eq!(rendered[0].folder_name, "KeepMe");
    assert_eq!(
        rendered[0]
            .local_info
            .as_ref()
            .and_then(|info| info.mod_id.as_deref()),
        Some("keep-id")
    );

    let reloaded = load_settings_file(&settings_file).expect("saved settings should remain valid");
    let data = reloaded
        .properties
        .iter()
        .find(|prop| prop.name == "Data")
        .expect("Data property");
    assert_eq!(data.mods.len(), 1);
    assert_eq!(data.mods[0].index, "0");
    assert_eq!(
        data.mods[0]
            .properties
            .iter()
            .find(|prop| prop.name == "ModPriority")
            .and_then(|prop| prop.value.as_deref()),
        Some("0")
    );

    let logs = runtime.logs.lock().unwrap();
    assert!(logs
        .iter()
        .any(|line| line.contains("Parsed GCMODSETTINGS.MXML successfully.")));
    assert!(logs
        .iter()
        .any(|line| line.contains("Cleaned orphaned mods from GCMODSETTINGS.MXML")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finalize_installation_errors_when_game_path_missing() {
    let runtime = FakeRuntime::default();
    let result = finalize_installation_with(&runtime, "pack_unpacked".to_string(), vec![], false);
    assert!(result.is_err(), "expected missing game path error");
    if let Err(err) = result {
        assert_eq!(err, "Could not find game path.");
    }
}

#[test]
fn finalize_installation_errors_when_library_source_missing() {
    let root = temp_test_dir("finalize_missing_source");
    let game = root.join("game");
    let library = root.join("library");
    let pulsar = root.join("pulsar");

    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(library.clone()).unwrap();
    fs::create_dir_all(pulsar.clone()).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game),
        library_dir: Some(library),
        pulsar_root: Some(pulsar),
        ..Default::default()
    };

    let result = finalize_installation_with(&runtime, "missing_source".to_string(), vec![], false);
    assert!(result.is_err(), "expected missing source error");
    if let Err(err) = result {
        assert!(err.contains("Library folder missing"));
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finalize_installation_propagates_missing_library_dir_configuration() {
    let root = temp_test_dir("finalize_missing_library_dir");
    let game = root.join("game");
    let pulsar = root.join("pulsar");
    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(pulsar).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game),
        library_dir: None,
        pulsar_root: Some(root.join("pulsar-root")),
        ..Default::default()
    };

    let result = finalize_installation_with(&runtime, "pack_unpacked".to_string(), vec![], false);
    assert!(result.is_err(), "expected library dir error");
    if let Err(err) = result {
        assert_eq!(err, "missing library_dir");
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finalize_installation_succeeds_with_single_direct_deploy() {
    let root = temp_test_dir("finalize_success");
    let game = root.join("game");
    let library = root.join("library");
    let pulsar = root.join("pulsar");
    let source_root = library.join("pack_unpacked");

    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(source_root.join("MyMod")).unwrap();
    fs::write(source_root.join("MyMod/test.pak"), "pak").unwrap();
    fs::create_dir_all(pulsar.clone()).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game.clone()),
        library_dir: Some(library),
        pulsar_root: Some(pulsar),
        ..Default::default()
    };

    let result = finalize_installation_with(
        &runtime,
        "pack_unpacked".to_string(),
        vec!["MyMod".to_string()],
        false,
    )
    .expect("expected successful finalize");

    assert_eq!(result.successes.len(), 1);
    assert!(result.conflicts.is_empty());
    assert!(game.join("GAMEDATA/MODS/MyMod/test.pak").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finalize_installation_scan_all_selection_installs_all_top_level_items() {
    let root = temp_test_dir("finalize_scan_all");
    let game = root.join("game");
    let library = root.join("library");
    let pulsar = root.join("pulsar");
    let source_root = library.join("pack_unpacked");

    fs::create_dir_all(game.join("GAMEDATA/MODS")).unwrap();
    fs::create_dir_all(source_root.join("FirstMod")).unwrap();
    fs::create_dir_all(source_root.join("SecondMod")).unwrap();
    fs::write(source_root.join("FirstMod/a.pak"), "pak-a").unwrap();
    fs::write(source_root.join("SecondMod/b.pak"), "pak-b").unwrap();
    fs::create_dir_all(pulsar.clone()).unwrap();

    let runtime = FakeRuntime {
        game_path: Some(game.clone()),
        library_dir: Some(library),
        pulsar_root: Some(pulsar),
        ..Default::default()
    };

    let result = finalize_installation_with(&runtime, "pack_unpacked".to_string(), vec![], false)
        .expect("expected successful finalize using scan-all selection");
    assert_eq!(result.successes.len(), 2);
    assert!(result.conflicts.is_empty());
    assert!(game.join("GAMEDATA/MODS/FirstMod/a.pak").exists());
    assert!(game.join("GAMEDATA/MODS/SecondMod/b.pak").exists());

    fs::remove_dir_all(root).unwrap();
}
