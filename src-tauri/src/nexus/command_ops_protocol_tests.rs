use super::*;

#[test]
fn linux_desktop_file_path_builds_expected_location() {
    let home = Path::new("/home/tester");
    let path = linux_desktop_file_path(home);
    assert_eq!(
        path,
        PathBuf::from("/home/tester/.local/share/applications/nxm-handler.desktop")
    );
}

#[test]
fn linux_desktop_entry_contains_required_fields() {
    let entry = linux_desktop_entry(Path::new("/opt/Pulsar/Pulsar"));
    assert!(entry.contains("Type=Application"));
    assert!(entry.contains("MimeType=x-scheme-handler/nxm;"));
    assert!(entry.contains("Exec=\"/opt/Pulsar/Pulsar\" %u"));
}

#[test]
fn windows_nxm_command_quotes_path_and_argument() {
    let cmd = windows_nxm_command(Path::new("C:\\Program Files\\Pulsar\\Pulsar.exe"));
    assert_eq!(cmd, "\"C:\\Program Files\\Pulsar\\Pulsar.exe\" \"%1\"");
}
