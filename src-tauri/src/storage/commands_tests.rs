use super::{
    check_library_existence_command_with, check_library_existence_with, clean_staging_folder_with,
    clear_downloads_folder_command_with, clear_downloads_folder_with,
    delete_library_folder_command_with, delete_library_folder_with, get_path_string_with,
    get_staging_contents_command_with, get_staging_contents_with, linux_show_in_folder_target,
    open_folder_path_with, open_special_folder_command_with, set_downloads_path_command_with,
    set_library_path_command_with,
};
use crate::adapters::tauri::storage::{delete_archive_file, open_folder_path};
use crate::storage::logic::clean_staging_dir;
use crate::storage::logic::library_folder_name;
use crate::storage::logic::{clear_dirs_in_dir, clear_files_in_dir, select_special_folder_path};
use crate::storage::ops::{
    delete_archive_file_if_exists, delete_library_folder_if_exists, ensure_folder_exists,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pulsarmm_storage_commands_wrapper_{}_{}",
        prefix,
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn storage_commands_wrapper_uses_expected_logic_contracts() {
    assert_eq!(library_folder_name("mod.zip"), "mod.zip_unpacked");
    assert_eq!(
        linux_show_in_folder_target(&PathBuf::from("/tmp/mods/archive.zip")),
        PathBuf::from("/tmp/mods")
    );

    let selected = select_special_folder_path(
        "downloads",
        PathBuf::from("/tmp/downloads"),
        PathBuf::from("/tmp/profiles"),
        PathBuf::from("/tmp/library"),
    )
    .expect("downloads should resolve");
    assert_eq!(selected, PathBuf::from("/tmp/downloads"));
}

#[test]
fn storage_commands_wrapper_exercises_dir_cleanup_helpers() {
    let dir = temp_test_dir("cleanup");
    fs::write(dir.join("a.zip"), "a").unwrap();
    fs::create_dir_all(dir.join("nested")).unwrap();

    assert_eq!(clear_files_in_dir(&dir).unwrap(), 1);
    assert_eq!(clear_dirs_in_dir(&dir).unwrap(), 1);

    fs::write(dir.join("b.zip"), "b").unwrap();
    assert_eq!(clean_staging_dir(&dir).unwrap(), 1);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn storage_commands_wrapper_uses_storage_ops_contracts() {
    let dir = temp_test_dir("ops");
    let archive = dir.join("a.zip");
    fs::write(&archive, "x").unwrap();
    delete_archive_file_if_exists(&archive).unwrap();
    assert!(!archive.exists());

    assert_eq!(ensure_folder_exists(&dir), Ok(()));
    assert!(ensure_folder_exists(&dir.join("missing")).is_err());

    let unpacked = dir.join("mod.zip_unpacked");
    fs::create_dir_all(&unpacked).unwrap();
    delete_library_folder_if_exists(&dir, "mod.zip").unwrap();
    assert!(!unpacked.exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn direct_command_wrappers_cover_delete_archive_and_open_folder_error_path() {
    let dir = temp_test_dir("direct_cmd");
    let archive = dir.join("a.zip");
    fs::write(&archive, "x").expect("failed to write archive");
    delete_archive_file(archive.to_string_lossy().into_owned())
        .expect("delete_archive_file command should succeed");
    assert!(!archive.exists());

    let missing = dir.join("missing");
    let err = open_folder_path(missing.to_string_lossy().into_owned())
        .expect_err("open_folder_path should fail when folder does not exist");
    assert!(
        !err.is_empty(),
        "open_folder_path should return an explanatory error"
    );

    fs::remove_dir_all(dir).expect("failed to clean temp dir");
}

#[test]
fn open_folder_path_with_ensures_then_opens() {
    let dir = temp_test_dir("open_with");
    let target = dir.join("target");
    fs::create_dir_all(&target).unwrap();

    let mut opened: Option<PathBuf> = None;
    open_folder_path_with(&target, &ensure_folder_exists, &mut |p| {
        opened = Some(p.to_path_buf());
        Ok(())
    })
    .expect("open wrapper should succeed");
    assert_eq!(opened.as_deref(), Some(target.as_path()));

    let err = open_folder_path_with(&target, &ensure_folder_exists, &mut |_p| {
        Err("open-failed".to_string())
    })
    .err()
    .expect("open path error should bubble");
    assert_eq!(err, "open-failed");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn get_path_string_with_converts_path() {
    let out = get_path_string_with(&|| Ok(PathBuf::from("/tmp/test"))).expect("path");
    assert_eq!(out, "/tmp/test");
}

#[test]
fn check_library_existence_with_delegates_map_builder() {
    let out = check_library_existence_with(
        Path::new("/library"),
        vec!["a.zip".to_string(), "b.zip".to_string()],
        &|_path, files| {
            let mut map = HashMap::new();
            for file in files {
                map.insert(file.clone(), file.starts_with('a'));
            }
            map
        },
    );
    assert_eq!(out.get("a.zip"), Some(&true));
    assert_eq!(out.get("b.zip"), Some(&false));
}

#[test]
fn delete_library_folder_with_forwards_arguments() {
    let mut called = false;
    delete_library_folder_with(Path::new("/lib"), "archive.zip", &mut |dir, zip| {
        called = true;
        assert_eq!(dir, Path::new("/lib"));
        assert_eq!(zip, "archive.zip");
        Ok(())
    })
    .expect("expected delegated delete");
    assert!(called);

    let err = delete_library_folder_with(Path::new("/lib"), "archive.zip", &mut |_dir, _zip| {
        Err("delete-library-failed".to_string())
    })
    .err()
    .expect("delete library error should bubble");
    assert_eq!(err, "delete-library-failed");
}

#[cfg(target_os = "linux")]
#[test]
fn show_in_folder_linux_with_uses_parent_target() {
    let mut opened: Option<PathBuf> = None;
    super::show_in_folder_linux_with(Path::new("/tmp/mods/archive.zip"), &mut |target| {
        opened = Some(target.to_path_buf());
        Ok(())
    });
    assert_eq!(opened.as_deref(), Some(Path::new("/tmp/mods")));
}

#[cfg(target_os = "linux")]
#[test]
fn show_in_folder_linux_with_ignores_spawn_error() {
    let mut called = false;
    super::show_in_folder_linux_with(Path::new("/tmp/mods/archive.zip"), &mut |target| {
        called = true;
        assert_eq!(target, Path::new("/tmp/mods"));
        Err("spawn-failed".to_string())
    });
    assert!(called);
}

#[test]
fn storage_command_wrappers_forward_paths_and_arguments() {
    clear_downloads_folder_with(
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/library")),
        &|d, l| {
            assert_eq!(d, Path::new("/downloads"));
            assert_eq!(l, Path::new("/library"));
            Ok(())
        },
    )
    .expect("clear wrapper");

    set_downloads_path_command_with(
        &|| Ok(PathBuf::from("/old-dl")),
        &|| Ok(PathBuf::from("/cfg.json")),
        "/new-dl",
        &mut |old, new_path, cfg| {
            assert_eq!(old, Path::new("/old-dl"));
            assert_eq!(new_path, "/new-dl");
            assert_eq!(cfg, Path::new("/cfg.json"));
            Ok(())
        },
    )
    .expect("set downloads wrapper");

    set_library_path_command_with(
        &|| Ok(PathBuf::from("/old-lib")),
        &|| Ok(PathBuf::from("/cfg.json")),
        "/new-lib",
        &mut |old, new_path, cfg| {
            assert_eq!(old, Path::new("/old-lib"));
            assert_eq!(new_path, "/new-lib");
            assert_eq!(cfg, Path::new("/cfg.json"));
            Ok(())
        },
    )
    .expect("set library wrapper");
}

#[test]
fn storage_command_wrappers_cover_staging_and_library_existence() {
    let nodes = get_staging_contents_with(
        &|| Ok(PathBuf::from("/library")),
        "temp123",
        "nested",
        &|root, rel| {
            assert_eq!(root, Path::new("/library/temp123"));
            assert_eq!(rel, "nested");
            Ok(vec![])
        },
    )
    .expect("staging contents wrapper");
    assert!(nodes.is_empty());

    let cleaned = clean_staging_folder_with(&|| Ok(PathBuf::from("/staging")), &|path| {
        assert_eq!(path, Path::new("/staging"));
        Ok(3)
    })
    .expect("clean staging wrapper");
    assert_eq!(cleaned, 3);

    let map = check_library_existence_command_with(
        &|| Ok(PathBuf::from("/library")),
        vec!["A.zip".to_string()],
        &|path, files| {
            assert_eq!(path, Path::new("/library"));
            let mut out = HashMap::new();
            out.insert(files[0].clone(), true);
            out
        },
    )
    .expect("library existence wrapper");
    assert_eq!(map.get("A.zip"), Some(&true));
}

#[test]
fn storage_command_level_wrappers_forward_dependencies() {
    clear_downloads_folder_command_with(
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/library")),
        &|downloads, library| {
            assert_eq!(downloads, Path::new("/downloads"));
            assert_eq!(library, Path::new("/library"));
            Ok(())
        },
    )
    .expect("clear downloads command wrapper");

    open_special_folder_command_with(
        "library",
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/profiles")),
        &|| Ok(PathBuf::from("/library")),
        &|path| {
            assert_eq!(path, PathBuf::from("/library"));
            Ok(())
        },
    )
    .expect("open special folder command wrapper");

    open_special_folder_command_with(
        "downloads",
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/profiles")),
        &|| Ok(PathBuf::from("/library")),
        &|path| {
            assert_eq!(path, PathBuf::from("/downloads"));
            Ok(())
        },
    )
    .expect("open special folder command wrapper (downloads)");

    open_special_folder_command_with(
        "profiles",
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/profiles")),
        &|| Ok(PathBuf::from("/library")),
        &|path| {
            assert_eq!(path, PathBuf::from("/profiles"));
            Ok(())
        },
    )
    .expect("open special folder command wrapper (profiles)");

    delete_library_folder_command_with(
        &|| Ok(PathBuf::from("/library")),
        "archive.zip",
        &mut |library, zip| {
            assert_eq!(library, Path::new("/library"));
            assert_eq!(zip, "archive.zip");
            Ok(())
        },
    )
    .expect("delete library folder command wrapper");

    let nodes = get_staging_contents_command_with(
        &|| Ok(PathBuf::from("/library")),
        "temp-id",
        ".",
        &|root, rel| {
            assert_eq!(root, Path::new("/library/temp-id"));
            assert_eq!(rel, ".");
            Ok(vec![])
        },
    )
    .expect("get staging contents command wrapper");
    assert!(nodes.is_empty());
}

#[test]
fn storage_command_level_wrappers_propagate_errors() {
    let err = clear_downloads_folder_command_with(
        &|| Err("downloads-failed".to_string()),
        &|| Ok(PathBuf::from("/library")),
        &|_downloads, _library| Ok(()),
    )
    .err()
    .expect("clear wrapper should propagate get downloads error");
    assert_eq!(err, "downloads-failed");

    let err = open_special_folder_command_with(
        "downloads",
        &|| Ok(PathBuf::from("/downloads")),
        &|| Err("profiles-failed".to_string()),
        &|| Ok(PathBuf::from("/library")),
        &|_path| Ok(()),
    )
    .err()
    .expect("open special wrapper should propagate profiles error");
    assert_eq!(err, "profiles-failed");

    let err = open_special_folder_command_with(
        "downloads",
        &|| Err("downloads-failed".to_string()),
        &|| Ok(PathBuf::from("/profiles")),
        &|| Ok(PathBuf::from("/library")),
        &|_path| Ok(()),
    )
    .err()
    .expect("open special wrapper should propagate downloads error");
    assert_eq!(err, "downloads-failed");

    let err = open_special_folder_command_with(
        "downloads",
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/profiles")),
        &|| Err("library-failed".to_string()),
        &|_path| Ok(()),
    )
    .err()
    .expect("open special wrapper should propagate library error");
    assert_eq!(err, "library-failed");

    let err = delete_library_folder_command_with(
        &|| Err("library-failed".to_string()),
        "archive.zip",
        &mut |_library, _zip| Ok(()),
    )
    .err()
    .expect("delete library wrapper should propagate dir error");
    assert_eq!(err, "library-failed");

    let err = get_staging_contents_command_with(
        &|| Ok(PathBuf::from("/library")),
        "temp-id",
        ".",
        &|_root, _rel| Err("collect-failed".to_string()),
    )
    .err()
    .expect("staging wrapper should propagate collect error");
    assert_eq!(err, "collect-failed");
}

#[test]
fn storage_command_wrappers_propagate_errors() {
    let err = open_folder_path_with(
        Path::new("/missing"),
        &|_path| Err("ensure-failed".to_string()),
        &mut |_path| Ok(()),
    )
    .err()
    .expect("ensure error should bubble");
    assert_eq!(err, "ensure-failed");

    let err = get_path_string_with(&|| Err("path-failed".to_string()))
        .err()
        .expect("path provider error should bubble");
    assert_eq!(err, "path-failed");

    let err = clear_downloads_folder_with(
        &|| Err("downloads-dir-failed".to_string()),
        &|| Ok(PathBuf::from("/library")),
        &|_d, _l| Ok(()),
    )
    .err()
    .expect("downloads dir error should bubble");
    assert_eq!(err, "downloads-dir-failed");

    let err = clear_downloads_folder_with(
        &|| Ok(PathBuf::from("/downloads")),
        &|| Err("library-dir-failed".to_string()),
        &|_d, _l| Ok(()),
    )
    .err()
    .expect("library dir error should bubble");
    assert_eq!(err, "library-dir-failed");

    let err = clear_downloads_folder_with(
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/library")),
        &|_d, _l| Err("clear-failed".to_string()),
    )
    .err()
    .expect("clear flow error should bubble");
    assert_eq!(err, "clear-failed");

    let err = set_downloads_path_command_with(
        &|| Err("old-downloads-failed".to_string()),
        &|| Ok(PathBuf::from("/cfg.json")),
        "/new",
        &mut |_old, _new_path, _cfg| Ok(()),
    )
    .err()
    .expect("old downloads path error should bubble");
    assert_eq!(err, "old-downloads-failed");

    let err = set_downloads_path_command_with(
        &|| Ok(PathBuf::from("/old")),
        &|| Err("config-failed".to_string()),
        "/new",
        &mut |_old, _new_path, _cfg| Ok(()),
    )
    .err()
    .expect("config path error should bubble");
    assert_eq!(err, "config-failed");

    let err = set_library_path_command_with(
        &|| Err("old-library-failed".to_string()),
        &|| Ok(PathBuf::from("/cfg.json")),
        "/new",
        &mut |_old, _new_path, _cfg| Ok(()),
    )
    .err()
    .expect("old library path error should bubble");
    assert_eq!(err, "old-library-failed");

    let err = set_library_path_command_with(
        &|| Ok(PathBuf::from("/old-library")),
        &|| Err("library-config-failed".to_string()),
        "/new",
        &mut |_old, _new_path, _cfg| Ok(()),
    )
    .err()
    .expect("library config path error should bubble");
    assert_eq!(err, "library-config-failed");

    let err = set_library_path_command_with(
        &|| Ok(PathBuf::from("/old-library")),
        &|| Ok(PathBuf::from("/cfg.json")),
        "/new",
        &mut |_old, _new_path, _cfg| Err("set-library-failed".to_string()),
    )
    .err()
    .expect("set library error should bubble");
    assert_eq!(err, "set-library-failed");

    let err = clean_staging_folder_with(&|| Err("staging-dir-failed".to_string()), &|_path| Ok(0))
        .err()
        .expect("staging dir error should bubble");
    assert_eq!(err, "staging-dir-failed");

    let err = clean_staging_folder_with(&|| Ok(PathBuf::from("/staging")), &|_path| {
        Err("clean-staging-failed".to_string())
    })
    .err()
    .expect("clean staging error should bubble");
    assert_eq!(err, "clean-staging-failed");

    let err = get_staging_contents_with(
        &|| Err("library-dir-failed".to_string()),
        "temp",
        ".",
        &|_root, _rel| Ok(vec![]),
    )
    .err()
    .expect("library dir error should bubble");
    assert_eq!(err, "library-dir-failed");

    let err = check_library_existence_command_with(
        &|| Err("library-dir-failed".to_string()),
        vec!["A.zip".to_string()],
        &|_path, _files| HashMap::new(),
    )
    .err()
    .expect("library existence dir error should bubble");
    assert_eq!(err, "library-dir-failed");

    let err = open_special_folder_command_with(
        "unknown",
        &|| Ok(PathBuf::from("/downloads")),
        &|| Ok(PathBuf::from("/profiles")),
        &|| Ok(PathBuf::from("/library")),
        &|_path| Ok(()),
    )
    .err()
    .expect("unknown folder type should bubble as error");
    assert_eq!(err, "Unknown folder type".to_string());
}
