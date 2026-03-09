use super::run_legacy_migration_in_paths;
use crate::utils::config::load_config_or_default;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_app_migration_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn run_legacy_migration_marks_config_done_without_game_path() {
    let dir = temp_test_dir("no_game");
    let config_path = dir.join("config.json");
    let profiles_dir = dir.join("profiles");
    fs::create_dir_all(&profiles_dir).expect("create profiles dir should succeed");

    let ran = run_legacy_migration_in_paths(&config_path, &profiles_dir, None)
        .expect("migration should succeed");
    assert!(ran);

    let cfg = load_config_or_default(&config_path, false);
    assert!(cfg.legacy_migration_done);

    let ran_again = run_legacy_migration_in_paths(&config_path, &profiles_dir, None)
        .expect("second migration should succeed");
    assert!(!ran_again);

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}

#[test]
fn run_legacy_migration_heals_install_source_when_profile_matches() {
    let dir = temp_test_dir("heal");
    let config_path = dir.join("config.json");
    let profiles_dir = dir.join("profiles");
    let game_path = dir.join("game");
    let mods_dir = game_path.join("GAMEDATA/MODS/MyMod");

    fs::create_dir_all(&profiles_dir).expect("create profiles dir should succeed");
    fs::create_dir_all(&mods_dir).expect("create mods dir should succeed");

    fs::write(
        profiles_dir.join("p1.json"),
        r#"{"name":"p1","mods":[{"filename":"archive.zip","mod_id":"123","file_id":"456","version":null,"installed_options":null}]}"#,
    )
    .expect("write profile should succeed");

    fs::write(
        mods_dir.join("mod_info.json"),
        r#"{"modId":"123","fileId":"456","installSource":""}"#,
    )
    .expect("write mod info should succeed");

    let ran = run_legacy_migration_in_paths(&config_path, &profiles_dir, Some(&game_path))
        .expect("migration should succeed");
    assert!(ran);

    let healed =
        fs::read_to_string(mods_dir.join("mod_info.json")).expect("mod info should be readable");
    assert!(healed.contains("\"installSource\": \"archive.zip\""));

    fs::remove_dir_all(dir).expect("cleanup should succeed");
}
