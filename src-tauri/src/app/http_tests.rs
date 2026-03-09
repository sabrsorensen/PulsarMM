use super::{
    is_image_request, is_image_response, lowercased_response_headers, map_image_body_result,
    map_text_body_result, normalized_http_method, perform_http_request, HttpVerb,
};
use base64::Engine as _;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

fn serve_once(status_line: &str, headers: &[(&str, &str)], body: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local addr should resolve");
    let status = status_line.to_string();
    let response_headers = headers
        .iter()
        .map(|(k, v)| format!("{k}: {v}\r\n"))
        .collect::<String>();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut request = [0u8; 1024];
        let _ = stream.read(&mut request);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{response_headers}Connection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write header should succeed");
        if !body.is_empty() {
            stream.write_all(body).expect("write body should succeed");
        }
    });

    format!("http://{addr}")
}

fn serve_truncated_once(
    status_line: &str,
    headers: &[(&str, &str)],
    declared_len: usize,
    body: &'static [u8],
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local addr should resolve");
    let status = status_line.to_string();
    let response_headers = headers
        .iter()
        .map(|(k, v)| format!("{k}: {v}\r\n"))
        .collect::<String>();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut request = [0u8; 1024];
        let _ = stream.read(&mut request);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {declared_len}\r\n{response_headers}Connection: close\r\n\r\n",
        );
        stream
            .write_all(response.as_bytes())
            .expect("write header should succeed");
        if !body.is_empty() {
            stream.write_all(body).expect("write body should succeed");
        }
    });

    format!("http://{addr}")
}

fn serve_once_capturing_request(
    status_line: &str,
    headers: &[(&str, &str)],
    body: &'static [u8],
) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local addr should resolve");
    let status = status_line.to_string();
    let response_headers = headers
        .iter()
        .map(|(k, v)| format!("{k}: {v}\r\n"))
        .collect::<String>();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut request = [0u8; 2048];
        let read = stream.read(&mut request).expect("read should succeed");
        tx.send(String::from_utf8_lossy(&request[..read]).into_owned())
            .expect("send captured request should succeed");

        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{response_headers}Connection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write header should succeed");
        if !body.is_empty() {
            stream.write_all(body).expect("write body should succeed");
        }
    });

    (format!("http://{addr}"), rx)
}

#[test]
fn http_verb_from_input_maps_supported_methods() {
    assert_eq!(HttpVerb::from_input(None).unwrap(), HttpVerb::Get);
    assert_eq!(
        HttpVerb::from_input(Some("post".to_string())).unwrap(),
        HttpVerb::Post
    );
    assert_eq!(
        HttpVerb::from_input(Some("PUT".to_string())).unwrap(),
        HttpVerb::Put
    );
    assert_eq!(
        HttpVerb::from_input(Some("delete".to_string())).unwrap(),
        HttpVerb::Delete
    );
    assert_eq!(
        HttpVerb::from_input(Some("head".to_string())).unwrap(),
        HttpVerb::Head
    );
}

#[test]
fn is_image_request_checks_content_type_and_url_suffixes() {
    assert!(is_image_request("https://x/y", "image/png"));
    assert!(is_image_request("https://x/y/file.jpg", "text/plain"));
    assert!(!is_image_request("https://x/y/file.txt", "text/plain"));
}

#[test]
fn normalized_http_method_defaults_to_get_and_uppercases() {
    assert_eq!(normalized_http_method(None), "GET");
    assert_eq!(normalized_http_method(Some("post".to_string())), "POST");
}

#[test]
fn is_image_response_uses_content_type_header_when_present() {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/webp"));
    let lowered = lowercased_response_headers(&headers);
    assert!(is_image_response("https://x/y/file.txt", &lowered));
}

#[test]
fn http_verb_from_input_rejects_unknown_method() {
    let err = HttpVerb::from_input(Some("patch".to_string())).unwrap_err();
    assert_eq!(err, "Unsupported HTTP method: PATCH");
}

#[test]
fn http_verb_build_request_uses_expected_methods() {
    let client = reqwest::Client::new();
    for (verb, expected) in [
        (HttpVerb::Get, "GET"),
        (HttpVerb::Post, "POST"),
        (HttpVerb::Put, "PUT"),
        (HttpVerb::Delete, "DELETE"),
        (HttpVerb::Head, "HEAD"),
    ] {
        let req = verb
            .build_request(&client, "http://127.0.0.1:1")
            .build()
            .expect("request should build");
        assert_eq!(req.method().as_str(), expected);
    }
}

#[test]
fn lowercased_response_headers_collects_ascii_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    headers.insert(USER_AGENT, HeaderValue::from_static("PulsarMM"));

    let out = lowercased_response_headers(&headers);
    assert_eq!(
        out.get("content-type").map(String::as_str),
        Some("image/png")
    );
    assert_eq!(out.get("user-agent").map(String::as_str), Some("PulsarMM"));
}

#[test]
fn lowercased_response_headers_skips_non_utf8_values() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-binary",
        HeaderValue::from_bytes(&[0xFF, 0xFE]).expect("header bytes should be accepted"),
    );

    let out = lowercased_response_headers(&headers);
    assert!(!out.contains_key("x-binary"));
}

#[test]
fn map_image_body_result_encodes_bytes_and_formats_errors() {
    assert_eq!(
        map_image_body_result(Ok(vec![1, 2, 3])).expect("image bytes should encode"),
        base64::engine::general_purpose::STANDARD.encode([1u8, 2, 3])
    );
    assert_eq!(
        map_image_body_result(Err("boom".to_string())).unwrap_err(),
        "Failed to read response bytes: boom"
    );
}

#[test]
fn map_text_body_result_returns_body_and_formats_errors() {
    assert_eq!(
        map_text_body_result(Ok("hello".to_string())).expect("text body should pass through"),
        "hello"
    );
    assert_eq!(
        map_text_body_result(Err("boom".to_string())).unwrap_err(),
        "Failed to read response body: boom"
    );
}

#[test]
fn perform_http_request_rejects_unsupported_method() {
    let out = tauri::async_runtime::block_on(perform_http_request(
        "https://example.com".to_string(),
        Some("patch".to_string()),
        None,
    ));
    match out {
        Ok(_) => panic!("unsupported method should fail"),
        Err(err) => assert_eq!(err, "Unsupported HTTP method: PATCH"),
    }
}

#[test]
fn perform_http_request_reports_invalid_url_errors() {
    let out = tauri::async_runtime::block_on(perform_http_request(
        "://not-a-url".to_string(),
        None,
        None,
    ));
    match out {
        Ok(_) => panic!("invalid url should fail"),
        Err(err) => assert!(err.contains("HTTP request failed:")),
    }
}

#[test]
fn perform_http_request_reads_text_body_and_headers() {
    let url = serve_once(
        "200 OK",
        &[("Content-Type", "text/plain"), ("X-Test", "yes")],
        b"hello world",
    );
    let out = tauri::async_runtime::block_on(perform_http_request(url, None, None))
        .expect("request should succeed");
    assert_eq!(out.status, 200);
    assert_eq!(out.status_text, "OK");
    assert_eq!(out.body, "hello world");
    assert_eq!(
        out.headers.get("content-type").map(String::as_str),
        Some("text/plain")
    );
    assert_eq!(out.headers.get("x-test").map(String::as_str), Some("yes"));
}

#[test]
fn perform_http_request_uses_empty_status_text_when_reason_is_unknown() {
    let url = serve_once(
        "299 Custom",
        &[("Content-Type", "text/plain")],
        b"custom body",
    );
    let out = tauri::async_runtime::block_on(perform_http_request(url, None, None))
        .expect("request should succeed");
    assert_eq!(out.status, 299);
    assert_eq!(out.status_text, "");
    assert_eq!(out.body, "custom body");
}

#[test]
fn perform_http_request_base64_encodes_image_response() {
    let bytes: &'static [u8] = &[1, 2, 3, 4];
    let url = serve_once("200 OK", &[("Content-Type", "image/png")], bytes);
    let out = tauri::async_runtime::block_on(perform_http_request(url, None, None))
        .expect("image request should succeed");
    assert_eq!(
        out.body,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    );
}

#[test]
fn perform_http_request_sends_custom_headers() {
    let (url, request_rx) =
        serve_once_capturing_request("200 OK", &[("Content-Type", "text/plain")], b"ok");
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Test".to_string(), "yes".to_string());
    headers.insert("Accept".to_string(), "text/plain".to_string());

    let out = tauri::async_runtime::block_on(perform_http_request(url, None, Some(headers)))
        .expect("request with headers should succeed");
    assert_eq!(out.status, 200);

    let request = request_rx.recv().expect("captured request should arrive");
    let lower = request.to_ascii_lowercase();
    assert!(lower.contains("x-test: yes"));
    assert!(lower.contains("accept: text/plain"));
}

#[test]
fn perform_http_request_supports_put_delete_and_head_methods() {
    for (method, expected) in [("PUT", "put"), ("DELETE", "delete"), ("HEAD", "head")] {
        let (url, request_rx) =
            serve_once_capturing_request("200 OK", &[("Content-Type", "text/plain")], b"ok");

        let out = tauri::async_runtime::block_on(perform_http_request(
            url,
            Some(method.to_string()),
            None,
        ))
        .expect("request should succeed");
        assert_eq!(out.status, 200, "{expected} request should keep status");

        let request = request_rx.recv().expect("captured request should arrive");
        assert!(
            request.starts_with(&format!("{method} / HTTP/1.1\r\n")),
            "{expected} request should use the expected verb"
        );
    }
}

#[test]
fn perform_http_request_reports_truncated_text_body_errors() {
    let url = serve_truncated_once("200 OK", &[("Content-Type", "text/plain")], 10, b"short");
    let err = match tauri::async_runtime::block_on(perform_http_request(url, None, None)) {
        Ok(_) => panic!("truncated text body should fail"),
        Err(err) => err,
    };
    assert!(err.contains("Failed to read response body:"));
}

#[test]
fn perform_http_request_reports_truncated_image_body_errors() {
    let url = serve_truncated_once("200 OK", &[("Content-Type", "image/png")], 10, b"img");
    let err = match tauri::async_runtime::block_on(perform_http_request(url, None, None)) {
        Ok(_) => panic!("truncated image body should fail"),
        Err(err) => err,
    };
    assert!(err.contains("Failed to read response bytes:"));
}
