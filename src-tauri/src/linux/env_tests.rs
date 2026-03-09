use super::*;
use std::collections::HashMap;

fn env_from(map: HashMap<String, String>) -> impl Fn(&str) -> Option<String> {
    move |key| map.get(key).cloned()
}

#[test]
fn flatpak_detection_works_for_both_markers() {
    let env = env_from(HashMap::from([(
        "FLATPAK_ID".to_string(),
        "com.example.App".to_string(),
    )]));
    assert!(is_flatpak_runtime(env));

    let env = env_from(HashMap::from([(
        "PULSAR_FLATPAK".to_string(),
        "1".to_string(),
    )]));
    assert!(is_flatpak_runtime(env));

    let env = env_from(HashMap::new());
    assert!(!is_flatpak_runtime(env));
}

#[test]
fn steam_deck_detection_checks_env_and_product_name() {
    let env = env_from(HashMap::from([("STEAM_DECK".to_string(), "1".to_string())]));
    assert!(is_steam_deck(env, None));

    let env = env_from(HashMap::new());
    assert!(is_steam_deck(env, Some("Jupiter")));

    let env = env_from(HashMap::new());
    assert!(!is_steam_deck(env, Some("Desktop")));
}

#[test]
fn x11_backend_is_not_forced_inside_flatpak() {
    assert!(should_force_x11_backend(false, false));
    assert!(!should_force_x11_backend(true, false));
    assert!(!should_force_x11_backend(false, true));
}

#[test]
fn linux_webkit_updates_are_minimal() {
    let updates = linux_webkit_env_updates(false);
    assert!(updates.contains(&("WEBKIT_DISABLE_DMABUF_RENDERER", "1")));
    assert!(updates.contains(&("G_TLS_GNUTLS_PRIORITY", "NORMAL:%COMPAT")));

    let updates = linux_webkit_env_updates(true);
    assert!(!updates.contains(&("WEBKIT_DISABLE_DMABUF_RENDERER", "1")));
    assert!(updates.contains(&("G_TLS_GNUTLS_PRIORITY", "NORMAL:%COMPAT")));
}

#[test]
fn steam_deck_updates_respect_existing_vars_and_flatpak() {
    let updates = steam_deck_env_updates(false, false, false, false, false);
    assert!(updates.contains(&("LIBGL_ALWAYS_SOFTWARE", "1")));
    assert!(updates.contains(&("WEBKIT_DISABLE_COMPOSITING_MODE", "1")));
    assert!(updates.contains(&("EGL_PLATFORM", "x11")));
    assert!(updates.contains(&("NO_AT_BRIDGE", "1")));
    assert!(updates.contains(&("GDK_BACKEND", "x11")));

    let flatpak_updates = steam_deck_env_updates(true, true, true, true, false);
    assert!(!flatpak_updates.contains(&("GDK_BACKEND", "x11")));
    assert_eq!(flatpak_updates, vec![("NO_AT_BRIDGE", "1")]);
}
