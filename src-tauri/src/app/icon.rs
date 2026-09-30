use std::fs;

fn read_icon_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

fn decode_icon_rgba(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let decoded = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((rgba.into_raw(), width, height))
}

fn build_window_icon(rgba: Vec<u8>, width: u32, height: u32) -> tauri::image::Image<'static> {
    tauri::image::Image::new_owned(rgba, width, height)
}

fn first_decoded_icon_rgba_with(
    candidate_paths: &[String],
    mut read: impl FnMut(&str) -> std::io::Result<Vec<u8>>,
    mut decode: impl FnMut(&[u8]) -> Result<(Vec<u8>, u32, u32), String>,
) -> Option<(Vec<u8>, u32, u32)> {
    for candidate in candidate_paths {
        let Ok(bytes) = read(candidate) else {
            continue;
        };
        let Ok((rgba, width, height)) = decode(&bytes) else {
            continue;
        };
        return Some((rgba, width, height));
    }

    None
}

pub(crate) fn load_runtime_window_icon() -> Option<tauri::image::Image<'static>> {
    load_runtime_window_icon_with(
        &runtime_icon_candidate_paths(),
        read_icon_bytes,
        decode_icon_rgba,
    )
}

fn load_runtime_window_icon_with(
    candidate_paths: &[String],
    read: impl FnMut(&str) -> std::io::Result<Vec<u8>>,
    decode: impl FnMut(&[u8]) -> Result<(Vec<u8>, u32, u32), String>,
) -> Option<tauri::image::Image<'static>> {
    let (rgba, width, height) = first_decoded_icon_rgba_with(candidate_paths, read, decode)?;
    Some(build_window_icon(rgba, width, height))
}

// Installed prefixes (Flatpak /app, the Nix store, AppImage's $APPDIR) all follow the Unix
// convention of `<prefix>/bin/<exe>` next to `<prefix>/share/...`, so deriving the icon path
// from the running executable's own location works across every packaging format without
// hardcoding any one of them. Flatpak's icon is named after the bundle identifier; the Nix/deb
// bundler (tauri-bundler) names it after `productName` instead, so both basenames are tried.
fn exe_relative_icon_candidates(exe_path: Option<std::path::PathBuf>) -> Vec<String> {
    let Some(prefix) = exe_path
        .as_deref()
        .and_then(|exe| exe.parent())
        .and_then(|bin_dir| bin_dir.parent())
    else {
        return Vec::new();
    };

    ["com.sabrsorensen.Pulsar.png", "Pulsar.png"]
        .into_iter()
        .filter_map(|name| {
            prefix
                .join("share/icons/hicolor/128x128/apps")
                .join(name)
                .to_str()
                .map(str::to_string)
        })
        .collect()
}

pub(crate) fn runtime_icon_candidate_paths() -> Vec<String> {
    let mut candidates = exe_relative_icon_candidates(std::env::current_exe().ok());
    candidates.push("src-tauri/icons/128x128.png".to_string());
    candidates.push("icons/128x128.png".to_string());
    candidates
}

#[cfg(test)]
#[path = "icon_tests.rs"]
mod tests;
