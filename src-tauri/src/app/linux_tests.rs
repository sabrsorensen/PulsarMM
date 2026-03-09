use super::{
    apply_linux_backend_config_with, configure_linux_environment_with, is_running_on_steam_deck,
    is_running_on_steam_deck_with,
};

#[test]
fn is_running_on_steam_deck_with_uses_env_or_product_name() {
    let from_env = is_running_on_steam_deck_with(
        |key| {
            if key == "SteamDeck" {
                return Some("1".to_string());
            }
            None
        },
        None,
    );
    assert!(from_env);

    let from_product = is_running_on_steam_deck_with(|_key| None, Some("Jupiter"));
    assert!(from_product);

    let no_match = is_running_on_steam_deck_with(|_key| None, Some("Desktop"));
    assert!(!no_match);
}

#[test]
fn is_running_on_steam_deck_wrapper_is_callable() {
    let _ = is_running_on_steam_deck();
}

#[test]
fn configure_linux_environment_with_sets_expected_keys_when_missing() {
    let mut set_calls = Vec::new();
    let mut logs = Vec::new();
    configure_linux_environment_with(
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        &mut |k, v| set_calls.push((k.to_string(), v.to_string())),
        &mut |msg| logs.push(msg.to_string()),
    );

    assert!(set_calls
        .iter()
        .any(|(k, _)| k == "WEBKIT_DISABLE_DMABUF_RENDERER"));
    assert!(set_calls.iter().any(|(k, _)| k == "LIBGL_ALWAYS_SOFTWARE"));
    assert!(set_calls
        .iter()
        .any(|(k, _)| k == "WEBKIT_DISABLE_COMPOSITING_MODE"));
    assert!(set_calls.iter().any(|(k, _)| k == "EGL_PLATFORM"));
    assert!(set_calls.iter().any(|(k, _)| k == "GDK_BACKEND"));
    assert!(!logs.is_empty());
}

#[test]
fn configure_linux_environment_with_skips_updates_when_already_configured() {
    let mut set_calls = Vec::new();
    let mut logs = Vec::new();
    configure_linux_environment_with(
        false,
        true,
        true,
        true,
        true,
        true,
        true,
        &mut |k, v| set_calls.push((k.to_string(), v.to_string())),
        &mut |msg| logs.push(msg.to_string()),
    );

    assert_eq!(
        set_calls,
        vec![(
            "G_TLS_GNUTLS_PRIORITY".to_string(),
            "NORMAL:%COMPAT".to_string()
        )]
    );
    assert_eq!(
        logs,
        vec!["[INFO] Linux WebKit network compatibility configured"]
    );
}

#[test]
fn apply_linux_backend_config_with_sets_or_skips_x11_based_on_inputs() {
    let mut set_calls = Vec::new();
    let mut logs = Vec::new();
    apply_linux_backend_config_with(
        false,
        false,
        &mut |k, v| set_calls.push((k.to_string(), v.to_string())),
        &mut |msg| logs.push(msg.to_string()),
    );
    assert_eq!(
        set_calls,
        vec![("GDK_BACKEND".to_string(), "x11".to_string())]
    );
    assert!(logs.iter().any(|m| m.contains("Forced GDK_BACKEND=x11")));

    let mut logs = Vec::new();
    apply_linux_backend_config_with(true, false, &mut |_, _| {}, &mut |msg| {
        logs.push(msg.to_string())
    });
    assert!(logs
        .iter()
        .any(|m| m.contains("Running in Flatpak - using native display backend")));
}
