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
    candidate_paths: &[&str],
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
    candidate_paths: &[&str],
    read: impl FnMut(&str) -> std::io::Result<Vec<u8>>,
    decode: impl FnMut(&[u8]) -> Result<(Vec<u8>, u32, u32), String>,
) -> Option<tauri::image::Image<'static>> {
    let (rgba, width, height) = first_decoded_icon_rgba_with(candidate_paths, read, decode)?;
    Some(build_window_icon(rgba, width, height))
}

pub(crate) fn runtime_icon_candidate_paths() -> [&'static str; 3] {
    [
        "/app/share/icons/hicolor/128x128/apps/com.sabrsorensen.Pulsar.png",
        "src-tauri/icons/128x128.png",
        "icons/128x128.png",
    ]
}

#[cfg(test)]
#[path = "icon_tests.rs"]
mod tests;
