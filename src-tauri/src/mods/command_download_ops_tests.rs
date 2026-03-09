use super::{
    download_archive_to_path_with, ensure_success_status, maybe_emit_download_progress_with,
    progress_payload,
};
use crate::models::InstallProgressPayload;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::async_runtime;
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

async fn serve_once(status_line: &str, body: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind test listener");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let status = status_line.to_string();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut req = [0u8; 1024];
        let _ = stream.read(&mut req);
        let header = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            status,
            body.len()
        );
        stream
            .write_all(header.as_bytes())
            .expect("write header should succeed");
        if !body.is_empty() {
            stream.write_all(body).expect("write body should succeed");
        }
    });
    format!("http://{}", addr)
}

async fn serve_truncated_once(
    status_line: &str,
    declared_len: usize,
    body: &'static [u8],
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind test listener");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let status = status_line.to_string();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut req = [0u8; 1024];
        let _ = stream.read(&mut req);
        let header = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            status, declared_len
        );
        stream
            .write_all(header.as_bytes())
            .expect("write header should succeed");
        if !body.is_empty() {
            stream.write_all(body).expect("write body should succeed");
        }
    });
    format!("http://{}", addr)
}

#[test]
fn ensure_success_status_accepts_success_and_formats_error() {
    ensure_success_status(reqwest::StatusCode::OK).expect("ok should pass");
    let err = ensure_success_status(reqwest::StatusCode::BAD_REQUEST).expect_err("expected err");
    assert!(err.contains("400 Bad Request"));
}

#[test]
fn progress_payload_formats_expected_fields() {
    let payload = progress_payload("abc", 42);
    assert_eq!(payload.id, "abc");
    assert_eq!(payload.step, "Downloading: 42%");
    assert_eq!(payload.progress, Some(42));
}

#[test]
fn maybe_emit_download_progress_with_emits_only_when_percent_available() {
    let emitted = Mutex::new(Vec::<InstallProgressPayload>::new());

    maybe_emit_download_progress_with(Some("abc"), 5, 10, &mut |payload| {
        emitted.lock().expect("emitted lock").push(payload)
    });
    maybe_emit_download_progress_with(Some("abc"), 1, 0, &mut |payload| {
        emitted.lock().expect("emitted lock").push(payload)
    });
    maybe_emit_download_progress_with(None, 5, 10, &mut |payload| {
        emitted.lock().expect("emitted lock").push(payload)
    });

    let events = emitted.into_inner().expect("emitted into_inner");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "abc");
    assert_eq!(events[0].progress, Some(50));
}

#[test]
fn download_archive_to_path_with_downloads_file_and_reports_progress() {
    async_runtime::block_on(async {
        let base = temp_test_dir("download_archive_ok");
        let out_path = base.join("mod.zip");
        let url = serve_once("200 OK", b"abcdef").await;
        let progress = Mutex::new(Vec::<u64>::new());

        let result = download_archive_to_path_with(&url, &out_path, &mut |downloaded, total| {
            if total > 0 {
                progress
                    .lock()
                    .expect("progress lock")
                    .push((downloaded * 100) / total);
            }
        })
        .await
        .expect("download should succeed");

        assert_eq!(result.size, 6);
        let content = fs::read(&out_path).expect("downloaded file should exist");
        assert_eq!(content, b"abcdef");
        assert!(
            progress
                .lock()
                .expect("progress lock")
                .iter()
                .any(|pct| *pct == 100),
            "expected a 100% progress event"
        );

        fs::remove_dir_all(base).expect("cleanup should succeed");
    });
}

#[test]
fn download_archive_to_path_with_reports_http_error_status() {
    async_runtime::block_on(async {
        let base = temp_test_dir("download_archive_status_err");
        let out_path = base.join("mod.zip");
        let url = serve_once("404 Not Found", b"").await;

        let err =
            match download_archive_to_path_with(&url, &out_path, &mut |_downloaded, _total| {})
                .await
            {
                Ok(_) => panic!("expected status error"),
                Err(err) => err,
            };
        assert!(err.contains("404 Not Found"));

        fs::remove_dir_all(base).expect("cleanup should succeed");
    });
}

#[test]
fn download_archive_to_path_with_reports_create_file_errors() {
    async_runtime::block_on(async {
        let base = temp_test_dir("download_archive_create_err");
        let url = serve_once("200 OK", b"abcdef").await;
        let err =
            match download_archive_to_path_with(&url, &base, &mut |_downloaded, _total| {}).await {
                Ok(_) => panic!("expected create file error"),
                Err(err) => err,
            };
        assert!(err.contains("Failed to create file"));

        fs::remove_dir_all(base).expect("cleanup should succeed");
    });
}

#[test]
fn download_archive_to_path_with_reports_request_start_errors() {
    async_runtime::block_on(async {
        let base = temp_test_dir("download_archive_request_err");
        let out_path = base.join("mod.zip");

        let err = match download_archive_to_path_with(
            "://bad-url",
            &out_path,
            &mut |_downloaded, _total| {},
        )
        .await
        {
            Ok(_) => panic!("invalid URL should fail request startup"),
            Err(err) => err,
        };
        assert!(err.contains("Failed to initiate HTTP request"));

        fs::remove_dir_all(base).expect("cleanup should succeed");
    });
}

#[test]
fn download_archive_to_path_with_reports_metadata_errors_after_write() {
    async_runtime::block_on(async {
        let base = temp_test_dir("download_archive_metadata_err");
        let out_path = base.join("mod.zip");
        let out_path_for_callback = out_path.clone();
        let removed = Mutex::new(false);
        let url = serve_once("200 OK", b"abcdef").await;

        let err = download_archive_to_path_with(&url, &out_path, &mut |_downloaded, _total| {
            let mut removed_guard = removed.lock().expect("removed lock");
            if !*removed_guard {
                fs::remove_file(&out_path_for_callback)
                    .expect("callback should be able to remove output path");
                *removed_guard = true;
            }
        })
        .await;
        let err = match err {
            Ok(_) => panic!("missing final path should fail metadata lookup"),
            Err(err) => err,
        };
        assert!(!err.is_empty());

        fs::remove_dir_all(base).expect("cleanup should succeed");
    });
}

#[test]
fn download_archive_to_path_with_reports_chunk_read_errors() {
    async_runtime::block_on(async {
        let base = temp_test_dir("download_archive_chunk_err");
        let out_path = base.join("mod.zip");
        let url = serve_truncated_once("200 OK", 10, b"abc").await;

        let err =
            match download_archive_to_path_with(&url, &out_path, &mut |_downloaded, _total| {})
                .await
            {
                Ok(_) => panic!("truncated response should fail while reading chunks"),
                Err(err) => err,
            };
        assert!(!err.is_empty(), "expected a non-empty chunk read error");

        fs::remove_dir_all(base).expect("cleanup should succeed");
    });
}

#[cfg(unix)]
#[test]
fn download_archive_to_path_with_reports_write_errors() {
    async_runtime::block_on(async {
        let sink = Path::new("/dev/full");
        if !sink.exists() {
            return;
        }

        let url = serve_once("200 OK", b"abcdef").await;
        let err =
            match download_archive_to_path_with(&url, sink, &mut |_downloaded, _total| {}).await {
                Ok(_) => panic!("writing to /dev/full should fail"),
                Err(err) => err,
            };
        assert!(!err.is_empty());
    });
}
