use super::{
    copy_profile_command_with, copy_profile_with, create_empty_profile_command_with,
    create_empty_profile_with, delete_profile_command_with, delete_profile_with,
    get_profile_mod_list_command_with, get_profile_mod_list_with, list_profiles_command_with,
    list_profiles_with, profile_json_path, rename_profile_command_with, rename_profile_with,
    with_profiles_dir,
};
use std::path::{Path, PathBuf};

#[test]
fn profile_json_path_appends_profile_name_with_json_extension() {
    let base = Path::new("/tmp/profiles");
    let path = profile_json_path(base, "Deck");
    assert_eq!(path, PathBuf::from("/tmp/profiles/Deck.json"));
}

#[test]
fn profile_command_seams_forward_arguments() {
    let dir = Path::new("/tmp/profiles");

    let listed = list_profiles_with(dir, |_d| vec!["Default".to_string(), "Deck".to_string()]);
    assert_eq!(listed, vec!["Default".to_string(), "Deck".to_string()]);

    delete_profile_with(dir, "Deck", |d, name| {
        assert_eq!(d, dir);
        assert_eq!(name, "Deck");
        Ok(())
    })
    .expect("delete seam");

    rename_profile_with(dir, "Old", "New", |d, old, new| {
        assert_eq!(d, dir);
        assert_eq!(old, "Old");
        assert_eq!(new, "New");
        Ok(())
    })
    .expect("rename seam");

    create_empty_profile_with(dir, "Fresh", |d, name| {
        assert_eq!(d, dir);
        assert_eq!(name, "Fresh");
        Ok(())
    })
    .expect("create seam");

    let mods = get_profile_mod_list_with(dir, "Deck", |json_path| {
        assert_eq!(json_path, Path::new("/tmp/profiles/Deck.json"));
        Ok(vec!["a.zip".to_string()])
    })
    .expect("mod list seam");
    assert_eq!(mods, vec!["a.zip".to_string()]);

    copy_profile_with(dir, "A", "B", |d, src, dst| {
        assert_eq!(d, dir);
        assert_eq!(src, "A");
        assert_eq!(dst, "B");
        Ok(())
    })
    .expect("copy seam");
}

#[test]
fn profile_command_wrappers_cover_dir_resolution_and_errors() {
    let dir = PathBuf::from("/tmp/profiles");

    let listed = list_profiles_command_with(
        || Ok(dir.clone()),
        |_d| vec!["Default".to_string(), "Deck".to_string()],
    )
    .expect("list wrapper");
    assert_eq!(listed, vec!["Default".to_string(), "Deck".to_string()]);

    delete_profile_command_with(
        "Deck",
        || Ok(dir.clone()),
        |d, name| {
            assert_eq!(d, Path::new("/tmp/profiles"));
            assert_eq!(name, "Deck");
            Ok(())
        },
    )
    .expect("delete wrapper");

    rename_profile_command_with(
        "Old",
        "New",
        || Ok(dir.clone()),
        |d, old, new| {
            assert_eq!(d, Path::new("/tmp/profiles"));
            assert_eq!(old, "Old");
            assert_eq!(new, "New");
            Ok(())
        },
    )
    .expect("rename wrapper");

    create_empty_profile_command_with(
        "Fresh",
        || Ok(dir.clone()),
        |d, name| {
            assert_eq!(d, Path::new("/tmp/profiles"));
            assert_eq!(name, "Fresh");
            Ok(())
        },
    )
    .expect("create wrapper");

    let mod_list = get_profile_mod_list_command_with(
        "Deck",
        || Ok(dir.clone()),
        |json_path| {
            assert_eq!(json_path, Path::new("/tmp/profiles/Deck.json"));
            Ok(vec!["a.zip".to_string()])
        },
    )
    .expect("mod list wrapper");
    assert_eq!(mod_list, vec!["a.zip".to_string()]);

    copy_profile_command_with(
        "A",
        "B",
        || Ok(dir.clone()),
        |d, src, dst| {
            assert_eq!(d, Path::new("/tmp/profiles"));
            assert_eq!(src, "A");
            assert_eq!(dst, "B");
            Ok(())
        },
    )
    .expect("copy wrapper");

    let err = with_profiles_dir::<()>(|| Err("profiles dir unavailable".to_string()), |_d| Ok(()))
        .expect_err("expected dir resolution error");
    assert_eq!(err, "profiles dir unavailable");
}
