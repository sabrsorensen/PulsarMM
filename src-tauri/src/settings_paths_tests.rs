use super::*;

#[test]
fn builds_binaries_dir() {
    let root = Path::new("/games/nms");
    assert_eq!(binaries_dir(root), PathBuf::from("/games/nms/Binaries"));
}

#[test]
fn builds_settings_dir() {
    let root = Path::new("/games/nms");
    assert_eq!(
        settings_dir(root),
        PathBuf::from("/games/nms/Binaries/SETTINGS")
    );
}

#[test]
fn builds_mod_settings_file() {
    let root = Path::new("/games/nms");
    assert_eq!(
        mod_settings_file(root),
        PathBuf::from("/games/nms/Binaries/SETTINGS/GCMODSETTINGS.MXML")
    );
}
