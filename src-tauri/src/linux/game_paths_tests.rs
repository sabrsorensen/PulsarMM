use super::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[test]
fn manual_path_requires_binaries_dir() {
    let dirs: HashSet<PathBuf> = [PathBuf::from("/games/nms/Binaries")].into_iter().collect();
    let found = linux_manual_game_path(Some("/games/nms".to_string()), |p| dirs.contains(p));
    assert_eq!(found, Some(PathBuf::from("/games/nms")));

    let not_found = linux_manual_game_path(Some("/games/other".to_string()), |p| dirs.contains(p));
    assert!(not_found.is_none());
}

#[test]
fn steam_path_helpers_build_expected_roots_and_fallbacks() {
    let home = Path::new("/home/test");

    assert_eq!(
        linux_steam_roots(home),
        vec![
            home.join(".steam/steam"),
            home.join(".local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
        ]
    );
    assert_eq!(
        linux_fallback_game_paths(home),
        vec![
            home.join(".steam/steam/steamapps/common/No Man's Sky"),
            home.join(".local/share/Steam/steamapps/common/No Man's Sky"),
            home.join(".var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/No Man's Sky"),
        ]
    );
}

#[test]
fn finds_game_path_from_manifest_in_primary_root() {
    let home = Path::new("/home/test");
    let primary_root = home.join(".steam/steam");
    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(primary_root.clone());
    dirs.insert(primary_root.join("steamapps/common/No Man's Sky/Binaries"));

    let mut files: HashMap<PathBuf, String> = HashMap::new();
    files.insert(
        primary_root.join("steamapps/appmanifest_275850.acf"),
        "\"installdir\" \"No Man's Sky\"".to_string(),
    );

    let found = find_linux_steam_game_path(home, |p| files.get(p).cloned(), |p| dirs.contains(p));
    assert_eq!(
        found,
        Some(primary_root.join("steamapps/common/No Man's Sky"))
    );
}

#[test]
fn finds_game_path_from_libraryfolders_vdf_entry() {
    let home = Path::new("/home/test");
    let primary_root = home.join(".steam/steam");
    let extra_library = PathBuf::from("/mnt/steam2");

    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(primary_root.clone());
    dirs.insert(extra_library.join("steamapps/common/NoMans/Binaries"));

    let mut files: HashMap<PathBuf, String> = HashMap::new();
    files.insert(
        primary_root.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n  \"1\"\n  {{\n    \"path\"    \"{}\"\n  }}\n}}",
            extra_library.display()
        ),
    );
    files.insert(
        extra_library.join("steamapps/appmanifest_275850.acf"),
        "\"installdir\" \"NoMans\"".to_string(),
    );

    let found = find_linux_steam_game_path(home, |p| files.get(p).cloned(), |p| dirs.contains(p));

    assert_eq!(found, Some(extra_library.join("steamapps/common/NoMans")));
}

#[test]
fn deduplicates_library_folders_before_reading_manifests() {
    let home = Path::new("/home/test");
    let primary_root = home.join(".steam/steam");
    let extra_library = PathBuf::from("/mnt/steam2");
    let manifest_path = extra_library.join("steamapps/appmanifest_275850.acf");

    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(primary_root.clone());
    dirs.insert(extra_library.join("steamapps/common/NoMans/Binaries"));

    let mut files: HashMap<PathBuf, String> = HashMap::new();
    files.insert(
        primary_root.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n  \"1\"\n  {{\n    \"path\"    \"{}\"\n  }}\n  \"2\"\n  {{\n    \"path\"    \"{}\"\n  }}\n}}",
            extra_library.display(),
            extra_library.display()
        ),
    );
    files.insert(
        manifest_path.clone(),
        "\"installdir\" \"NoMans\"".to_string(),
    );

    let reads = Arc::new(Mutex::new(HashMap::<PathBuf, usize>::new()));
    let reads_out = reads.clone();
    let found = find_linux_steam_game_path(
        home,
        move |p| {
            let mut guard = reads_out.lock().expect("read counter lock");
            *guard.entry(p.to_path_buf()).or_insert(0) += 1;
            files.get(p).cloned()
        },
        |p| dirs.contains(p),
    );

    assert_eq!(found, Some(extra_library.join("steamapps/common/NoMans")));
    assert_eq!(
        reads
            .lock()
            .expect("read counter lock")
            .get(&manifest_path)
            .copied(),
        Some(1),
        "duplicate library entries should not cause duplicate manifest reads"
    );
}

#[test]
fn falls_back_to_default_paths_when_manifest_missing() {
    let home = Path::new("/home/test");
    let primary_root = home.join(".steam/steam");
    let fallback = home.join(".local/share/Steam/steamapps/common/No Man's Sky");

    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(primary_root);
    dirs.insert(fallback.join("Binaries"));

    let found = find_linux_steam_game_path(home, |_p| None, |p| dirs.contains(p));

    assert_eq!(found, Some(fallback));
}

#[test]
fn finds_game_path_from_flatpak_steam_root() {
    let home = Path::new("/home/test");
    let flatpak_root = home.join(".var/app/com.valvesoftware.Steam/data/Steam");
    let manifest_path = flatpak_root.join("steamapps/appmanifest_275850.acf");

    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(flatpak_root.clone());
    dirs.insert(flatpak_root.join("steamapps/common/DeckSky/Binaries"));

    let mut files: HashMap<PathBuf, String> = HashMap::new();
    files.insert(manifest_path, "\"installdir\" \"DeckSky\"".to_string());

    let found = find_linux_steam_game_path(home, |p| files.get(p).cloned(), |p| dirs.contains(p));
    assert_eq!(found, Some(flatpak_root.join("steamapps/common/DeckSky")));
}

#[test]
fn skips_manifests_without_installdir_and_with_missing_binaries_then_falls_back() {
    let home = Path::new("/home/test");
    let primary_root = home.join(".steam/steam");
    let fallback = home.join(".local/share/Steam/steamapps/common/No Man's Sky");

    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(primary_root.clone());
    dirs.insert(fallback.join("Binaries"));

    let mut files: HashMap<PathBuf, String> = HashMap::new();
    files.insert(
        primary_root.join("steamapps/appmanifest_275850.acf"),
        "\"appid\" \"275850\"".to_string(),
    );

    let found = find_linux_steam_game_path(home, |p| files.get(p).cloned(), |p| dirs.contains(p));
    assert_eq!(found, Some(fallback.clone()));

    files.insert(
        primary_root.join("steamapps/appmanifest_275850.acf"),
        "\"installdir\" \"No Man's Sky\"".to_string(),
    );
    let found = find_linux_steam_game_path(home, |p| files.get(p).cloned(), |p| dirs.contains(p));
    assert_eq!(found, Some(fallback));
}

#[test]
fn find_linux_game_path_with_prefers_manual_path_and_requires_home_for_auto_detection() {
    let mut dirs: HashSet<PathBuf> = HashSet::new();
    dirs.insert(PathBuf::from("/games/nms/Binaries"));
    dirs.insert(PathBuf::from("/home/test/.steam/steam"));
    dirs.insert(PathBuf::from(
        "/home/test/.steam/steam/steamapps/common/No Man's Sky/Binaries",
    ));

    let auto = find_linux_game_path_with(
        |key| match key {
            "HOME" => Some("/home/test".to_string()),
            _ => None,
        },
        |path| {
            (path == Path::new("/home/test/.steam/steam/steamapps/appmanifest_275850.acf"))
                .then(|| "\"installdir\" \"No Man's Sky\"".to_string())
        },
        |p| dirs.contains(p),
    );
    assert_eq!(
        auto,
        Some(PathBuf::from(
            "/home/test/.steam/steam/steamapps/common/No Man's Sky"
        ))
    );

    let manual = find_linux_game_path_with(
        |key| match key {
            "PULSAR_NMS_PATH" => Some("/games/nms".to_string()),
            "HOME" => Some("/home/test".to_string()),
            _ => None,
        },
        |_path| None,
        |p| dirs.contains(p),
    );
    assert_eq!(manual, Some(PathBuf::from("/games/nms")));

    let missing_home = find_linux_game_path_with(|_| None, |_path| None, |_p| false);
    assert!(missing_home.is_none());
}

#[test]
fn returns_none_when_no_candidates_exist() {
    let home = Path::new("/home/test");
    let found = find_linux_steam_game_path(home, |_p| None, |_p| false);
    assert!(found.is_none());
}
