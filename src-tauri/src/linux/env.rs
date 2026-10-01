pub fn is_flatpak_runtime<FGetEnv>(get_env: FGetEnv) -> bool
where
    FGetEnv: Fn(&str) -> Option<String>,
{
    get_env("FLATPAK_ID").is_some() || get_env("PULSAR_FLATPAK").is_some()
}

pub fn is_steam_deck<FGetEnv>(get_env: FGetEnv, product_name: Option<&str>) -> bool
where
    FGetEnv: Fn(&str) -> Option<String>,
{
    get_env("STEAM_DECK").as_deref() == Some("1")
        || get_env("SteamDeck").as_deref() == Some("1")
        || product_name
            .map(|s| {
                let s = s.trim();
                s.contains("Jupiter") || s.contains("Galileo") || s.contains("Steam Deck")
            })
            .unwrap_or(false)
}

pub fn should_force_x11_backend(is_flatpak: bool, gdk_backend_present: bool) -> bool {
    !is_flatpak && !gdk_backend_present
}

// The Flatpak runs native Wayland with the app's own titlebar; forcing a second one is redundant.
pub fn should_force_window_decorations(is_steam_deck: bool, is_flatpak: bool) -> bool {
    is_steam_deck && !is_flatpak
}

// Verified on Jovian/AMD (kernel 7, Wayland) and NVIDIA/Wayland: disabling the DMABUF renderer
// leaves the window unpainted on the current WebKitGTK/wry stack. Stock rendering works on both,
// so this is never forced; it's left up to the user to opt into if some future hardware needs it.
pub fn linux_webkit_env_updates() -> Vec<(&'static str, &'static str)> {
    vec![("G_TLS_GNUTLS_PRIORITY", "NORMAL:%COMPAT")]
}

pub fn steam_deck_env_updates(
    is_flatpak: bool,
    libgl_present: bool,
    webkit_compositing_present: bool,
    egl_platform_present: bool,
    gdk_backend_present: bool,
) -> Vec<(&'static str, &'static str)> {
    let mut updates = Vec::new();
    if !is_flatpak {
        if !libgl_present {
            updates.push(("LIBGL_ALWAYS_SOFTWARE", "1"));
        }
        if !webkit_compositing_present {
            updates.push(("WEBKIT_DISABLE_COMPOSITING_MODE", "1"));
        }
        if !egl_platform_present {
            updates.push(("EGL_PLATFORM", "x11"));
        }
    }

    updates.push(("NO_AT_BRIDGE", "1"));

    if should_force_x11_backend(is_flatpak, gdk_backend_present) {
        updates.push(("GDK_BACKEND", "x11"));
    }

    updates
}

#[cfg(test)]
#[path = "env_tests.rs"]
mod tests;
