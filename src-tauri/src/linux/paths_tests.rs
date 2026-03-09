use super::*;

#[test]
fn parse_empty_vdf_returns_empty() {
    let out = parse_steam_library_folders("");
    assert!(out.is_empty());
}

#[test]
fn parse_new_format_path_key() {
    let vdf = r#"
"libraryfolders"
{
  "0"
  {
    "path" "/home/deck/.local/share/Steam"
  }
}
"#;
    let out = parse_steam_library_folders(vdf);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0], PathBuf::from("/home/deck/.local/share/Steam"));
}

#[test]
fn parse_old_numeric_key_format() {
    let vdf = r#"
"libraryfolders"
{
  "0" "/mnt/games/SteamLibrary"
}
"#;
    let out = parse_steam_library_folders(vdf);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0], PathBuf::from("/mnt/games/SteamLibrary"));
}

#[test]
fn parse_windows_escaped_paths_normalizes_slashes() {
    let vdf = r#"
"libraryfolders"
{
  "1"
  {
    "path" "D:\\SteamLibrary"
  }
  "2" "E:\\AnotherLibrary"
}
"#;
    let out = parse_steam_library_folders(vdf);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0], PathBuf::from("D:/SteamLibrary"));
    assert_eq!(out[1], PathBuf::from("E:/AnotherLibrary"));
}

#[test]
fn parse_ignores_non_numeric_non_path_entries() {
    let vdf = r#"
"libraryfolders"
{
  "contentstatsid" "123456789"
  "something" "else"
}
"#;
    let out = parse_steam_library_folders(vdf);
    assert!(out.is_empty());
}

#[test]
fn parse_collects_multiple_entries() {
    let vdf = r#"
"libraryfolders"
{
  "0" "/one"
  "1" "/two"
  "2"
  {
    "path" "/three"
  }
}
"#;
    let out = parse_steam_library_folders(vdf);
    assert_eq!(out.len(), 3);
    assert_eq!(out[0], PathBuf::from("/one"));
    assert_eq!(out[1], PathBuf::from("/two"));
    assert_eq!(out[2], PathBuf::from("/three"));
}

#[test]
fn extract_installdir_returns_none_when_missing() {
    let acf = r#"
"AppState"
{
  "appid" "275850"
}
"#;
    assert_eq!(extract_installdir_from_manifest(acf), None);
}

#[test]
fn extract_installdir_returns_value_when_present() {
    let acf = r#"
"AppState"
{
  "appid" "275850"
  "installdir" "No Man's Sky"
}
"#;
    assert_eq!(
        extract_installdir_from_manifest(acf),
        Some("No Man's Sky".to_string())
    );
}
