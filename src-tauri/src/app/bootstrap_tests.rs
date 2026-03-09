use super::{
    apply_linux_backend_config_with, configure_linux_environment_with, should_force_x11_for_runtime,
};
use std::collections::HashMap;

#[test]
fn should_force_x11_for_runtime_matches_linux_env_logic() {
    assert!(should_force_x11_for_runtime(false, false));
    assert!(!should_force_x11_for_runtime(true, false));
    assert!(!should_force_x11_for_runtime(false, true));
}

#[test]
fn apply_linux_backend_config_with_sets_x11_when_needed() {
    let mut env = HashMap::new();
    let mut logs = Vec::new();
    apply_linux_backend_config_with(
        false,
        false,
        &mut |k, v| {
            env.insert(k.to_string(), v.to_string());
        },
        &mut |m| logs.push(m.to_string()),
    );
    assert_eq!(env.get("GDK_BACKEND").map(String::as_str), Some("x11"));
    assert!(logs.iter().any(|m| m.contains("Forced GDK_BACKEND=x11")));
}

#[test]
fn apply_linux_backend_config_with_logs_flatpak_branch() {
    let mut logs = Vec::new();
    apply_linux_backend_config_with(true, false, &mut |_, _| {}, &mut |m| {
        logs.push(m.to_string())
    });
    assert!(logs
        .iter()
        .any(|m| m.contains("Running in Flatpak - using native display backend")));
}

#[test]
fn apply_linux_backend_config_with_is_noop_when_backend_already_set() {
    let mut env = HashMap::new();
    let mut logs = Vec::new();
    apply_linux_backend_config_with(
        false,
        true,
        &mut |k, v| {
            env.insert(k.to_string(), v.to_string());
        },
        &mut |m| logs.push(m.to_string()),
    );
    assert!(env.is_empty());
    assert!(logs.is_empty());
}

#[test]
fn configure_linux_environment_with_applies_webkit_and_steam_deck_updates() {
    let mut env = HashMap::new();
    let mut logs = Vec::new();
    configure_linux_environment_with(
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        &mut |k, v| {
            env.insert(k.to_string(), v.to_string());
        },
        &mut |m| logs.push(m.to_string()),
    );

    assert_eq!(
        env.get("WEBKIT_DISABLE_DMABUF_RENDERER")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        env.get("WEBKIT_DISABLE_COMPOSITING_MODE")
            .map(String::as_str),
        Some("1")
    );
    assert!(logs.iter().any(|m| m.contains("Linux WebKit network")));
    assert!(logs.iter().any(|m| m.contains("Steam Deck compatibility")));
}

#[test]
fn configure_linux_environment_with_skips_steam_deck_updates_when_not_needed() {
    let mut env = HashMap::new();
    let mut logs = Vec::new();
    configure_linux_environment_with(
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        &mut |k, v| {
            env.insert(k.to_string(), v.to_string());
        },
        &mut |m| logs.push(m.to_string()),
    );

    assert_eq!(
        env.get("WEBKIT_DISABLE_DMABUF_RENDERER")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(env.get("LIBGL_ALWAYS_SOFTWARE").map(String::as_str), None);
    assert!(logs.iter().any(|m| m.contains("Linux WebKit network")));
    assert!(!logs.iter().any(|m| m.contains("Steam Deck detected")));
}

#[test]
fn configure_linux_environment_with_logs_steam_deck_without_reapplying_existing_vars() {
    let mut env = HashMap::new();
    let mut logs = Vec::new();
    configure_linux_environment_with(
        true,
        false,
        true,
        true,
        true,
        true,
        true,
        &mut |k, v| {
            env.insert(k.to_string(), v.to_string());
        },
        &mut |m| logs.push(m.to_string()),
    );

    assert_eq!(
        env.get("G_TLS_GNUTLS_PRIORITY").map(String::as_str),
        Some("NORMAL:%COMPAT")
    );
    assert_eq!(env.get("NO_AT_BRIDGE").map(String::as_str), Some("1"));
    assert_eq!(
        env.get("WEBKIT_DISABLE_DMABUF_RENDERER")
            .map(String::as_str),
        None
    );
    assert_eq!(env.get("LIBGL_ALWAYS_SOFTWARE").map(String::as_str), None);
    assert_eq!(
        env.get("WEBKIT_DISABLE_COMPOSITING_MODE")
            .map(String::as_str),
        None
    );
    assert_eq!(env.get("EGL_PLATFORM").map(String::as_str), None);
    assert_eq!(env.get("GDK_BACKEND").map(String::as_str), None);
    assert!(logs.iter().any(|m| m.contains("Steam Deck detected")));
    assert!(logs
        .iter()
        .any(|m| m.contains("Steam Deck compatibility environment configured")));
}
