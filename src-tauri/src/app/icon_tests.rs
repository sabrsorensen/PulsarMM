use super::{
    build_window_icon, decode_icon_rgba, first_decoded_icon_rgba_with, load_runtime_window_icon,
    load_runtime_window_icon_with, read_icon_bytes, runtime_icon_candidate_paths,
};
use std::io;

#[test]
fn runtime_icon_candidate_paths_are_stable_and_ordered() {
    let paths = runtime_icon_candidate_paths();
    assert_eq!(paths.len(), 3);
    assert_eq!(
        paths[0],
        "/app/share/icons/hicolor/128x128/apps/com.sabrsorensen.Pulsar.png"
    );
    assert_eq!(paths[1], "src-tauri/icons/128x128.png");
    assert_eq!(paths[2], "icons/128x128.png");
}

#[test]
fn first_decoded_icon_rgba_with_skips_errors_and_returns_first_success() {
    let mut reads = Vec::new();
    let mut decodes = 0usize;

    let result = first_decoded_icon_rgba_with(
        &["/missing.png", "/bad.png", "/good.png"],
        |path| {
            reads.push(path.to_string());
            match path {
                "/missing.png" => Err(io::Error::new(io::ErrorKind::NotFound, "missing")),
                "/bad.png" => Ok(vec![0, 1, 2]),
                "/good.png" => Ok(vec![3, 4, 5, 6]),
                _ => unreachable!("unexpected path"),
            }
        },
        |bytes| {
            decodes += 1;
            if bytes == [0, 1, 2] {
                return Err("decode failed".to_string());
            }
            Ok((bytes.to_vec(), 2, 3))
        },
    );

    assert_eq!(
        reads,
        vec![
            "/missing.png".to_string(),
            "/bad.png".to_string(),
            "/good.png".to_string()
        ]
    );
    assert_eq!(decodes, 2);
    let (rgba, width, height) = result.expect("expected first successful decode");
    assert_eq!(rgba, vec![3, 4, 5, 6]);
    assert_eq!(width, 2);
    assert_eq!(height, 3);
}

#[test]
fn first_decoded_icon_rgba_with_returns_none_when_all_candidates_fail() {
    let result = first_decoded_icon_rgba_with(
        &["/a.png", "/b.png"],
        |_| Err(io::Error::other("no file")),
        |_| Ok((vec![255], 1, 1)),
    );
    assert!(result.is_none());
}

#[test]
fn load_runtime_window_icon_with_builds_image_and_handles_none() {
    let icon = load_runtime_window_icon_with(
        &["/good.png"],
        |_path| Ok(vec![1, 2, 3, 4]),
        |bytes| Ok((bytes.to_vec(), 2, 2)),
    )
    .expect("expected icon");
    assert_eq!(icon.width(), 2);
    assert_eq!(icon.height(), 2);
    assert_eq!(icon.rgba(), &[1, 2, 3, 4]);

    let none = load_runtime_window_icon_with(
        &["/missing.png"],
        |_path| Err(io::Error::other("missing")),
        |_bytes| Ok((vec![255, 0, 0, 255], 1, 1)),
    );
    assert!(none.is_none());
}

#[test]
fn build_window_icon_constructs_expected_image() {
    let icon = build_window_icon(vec![1, 2, 3, 4], 2, 2);
    assert_eq!(icon.width(), 2);
    assert_eq!(icon.height(), 2);
    assert_eq!(icon.rgba(), &[1, 2, 3, 4]);
}

#[test]
fn load_runtime_window_icon_smoke_test() {
    let icon = load_runtime_window_icon();
    if let Some(icon) = icon {
        assert!(icon.width() > 0);
        assert!(icon.height() > 0);
        assert!(!icon.rgba().is_empty());
    }
}

#[test]
fn decode_icon_rgba_handles_valid_and_invalid_bytes() {
    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/32x32.png");
    let bytes =
        read_icon_bytes(icon_path.to_string_lossy().as_ref()).expect("icon fixture should exist");
    let (rgba, width, height) = decode_icon_rgba(&bytes).expect("fixture should decode");
    assert!(width > 0);
    assert!(height > 0);
    assert!(!rgba.is_empty());

    let err = decode_icon_rgba(b"not-a-real-image").expect_err("invalid bytes should fail");
    assert!(!err.is_empty());
}

#[test]
fn read_icon_bytes_reads_fixture_and_reports_missing_file() {
    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/32x32.png");
    let bytes =
        read_icon_bytes(icon_path.to_string_lossy().as_ref()).expect("icon fixture should exist");
    assert!(!bytes.is_empty());

    let missing = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/missing.png");
    let err = read_icon_bytes(missing.to_string_lossy().as_ref())
        .expect_err("missing icon should report read error");
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}
