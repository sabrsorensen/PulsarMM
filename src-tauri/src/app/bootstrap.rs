use crate::linux;

pub(crate) fn should_force_x11_for_runtime(is_flatpak: bool, gdk_backend_is_set: bool) -> bool {
    linux::env::should_force_x11_backend(is_flatpak, gdk_backend_is_set)
}

pub(crate) fn apply_linux_backend_config_with(
    is_flatpak: bool,
    gdk_backend_is_set: bool,
    set_env: &mut dyn FnMut(&str, &str),
    log: &mut dyn FnMut(&str),
) {
    if should_force_x11_for_runtime(is_flatpak, gdk_backend_is_set) {
        set_env("GDK_BACKEND", "x11");
        log("[INFO] Forced GDK_BACKEND=x11 for WebKitGTK compatibility");
    } else if is_flatpak {
        log("[INFO] Running in Flatpak - using native display backend");
    }
}

pub(crate) fn configure_linux_environment_with(
    is_steam_deck: bool,
    is_flatpak: bool,
    webkit_dmabuf_is_set: bool,
    libgl_software_is_set: bool,
    webkit_compositing_is_set: bool,
    egl_platform_is_set: bool,
    gdk_backend_is_set: bool,
    set_env: &mut dyn FnMut(&str, &str),
    log: &mut dyn FnMut(&str),
) {
    for (key, value) in linux::env::linux_webkit_env_updates(webkit_dmabuf_is_set) {
        set_env(key, value);
    }
    log("[INFO] Linux WebKit network compatibility configured");

    if is_steam_deck {
        log("[INFO] Steam Deck detected, applying compatibility settings...");
        for (key, value) in linux::env::steam_deck_env_updates(
            is_flatpak,
            libgl_software_is_set,
            webkit_compositing_is_set,
            egl_platform_is_set,
            gdk_backend_is_set,
        ) {
            set_env(key, value);
        }
        log("[INFO] Steam Deck compatibility environment configured");
    }
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
