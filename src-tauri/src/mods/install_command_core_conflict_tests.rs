use super::resolve_conflict_with;
use std::cell::RefCell;
use std::path::PathBuf;

#[test]
fn resolve_conflict_with_requires_game_path() {
    let out = resolve_conflict_with(
        || None,
        "new",
        "old",
        "/tmp/mod",
        true,
        |_mods, _new, _old, _temp, _replace| Ok(()),
    );
    assert_eq!(
        out.expect_err("missing game path should error"),
        "Could not find game path."
    );
}

#[test]
fn resolve_conflict_with_passes_expected_arguments() {
    let called = RefCell::new(None::<(PathBuf, String, String, PathBuf, bool)>);
    let out = resolve_conflict_with(
        || Some(PathBuf::from("/game")),
        "new_mod",
        "old_mod",
        "/tmp/new_mod",
        false,
        |mods, new_name, old_name, temp_path, replace| {
            *called.borrow_mut() = Some((
                mods.to_path_buf(),
                new_name.to_string(),
                old_name.to_string(),
                temp_path.to_path_buf(),
                replace,
            ));
            Ok(())
        },
    );
    assert!(out.is_ok());

    let args = called
        .into_inner()
        .expect("expected resolver to receive arguments");
    assert_eq!(args.0, PathBuf::from("/game/GAMEDATA/MODS"));
    assert_eq!(args.1, "new_mod");
    assert_eq!(args.2, "old_mod");
    assert_eq!(args.3, PathBuf::from("/tmp/new_mod"));
    assert!(!args.4);
}

#[test]
fn resolve_conflict_with_propagates_resolver_error() {
    let out = resolve_conflict_with(
        || Some(PathBuf::from("/game")),
        "new_mod",
        "old_mod",
        "/tmp/new_mod",
        true,
        |_mods, _new_name, _old_name, _temp_path, _replace| Err("resolver failed".to_string()),
    );
    assert_eq!(
        out.expect_err("resolver error should bubble"),
        "resolver failed"
    );
}
