use super::bootstrap;
use crate::linux;

pub(crate) fn is_running_on_steam_deck_with(
    get_env: impl Fn(&str) -> Option<String>,
    product_name: Option<&str>,
) -> bool {
    linux::env::is_steam_deck(get_env, product_name)
}

pub(crate) fn is_running_on_steam_deck() -> bool {
    let product_name = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").ok();
    is_running_on_steam_deck_with(|k| std::env::var(k).ok(), product_name.as_deref())
}

pub(crate) fn configure_linux_environment_with(
    is_steam_deck: bool,
    is_flatpak: bool,
    has_webkit_disable_dmabuf_renderer: bool,
    has_libgl_always_software: bool,
    has_webkit_disable_compositing_mode: bool,
    has_egl_platform: bool,
    has_gdk_backend: bool,
    set_env: &mut dyn FnMut(&str, &str),
    log: &mut dyn FnMut(&str),
) {
    bootstrap::configure_linux_environment_with(
        is_steam_deck,
        is_flatpak,
        has_webkit_disable_dmabuf_renderer,
        has_libgl_always_software,
        has_webkit_disable_compositing_mode,
        has_egl_platform,
        has_gdk_backend,
        set_env,
        log,
    );
}

pub(crate) fn apply_linux_backend_config_with(
    is_flatpak: bool,
    gdk_backend_is_set: bool,
    set_env: &mut dyn FnMut(&str, &str),
    log: &mut dyn FnMut(&str),
) {
    bootstrap::apply_linux_backend_config_with(is_flatpak, gdk_backend_is_set, set_env, log);
}

#[cfg(test)]
#[path = "linux_tests.rs"]
mod tests;
