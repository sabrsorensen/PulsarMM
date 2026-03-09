use super::{
    await_api_key_from_messages_with, ensure_auth_path_for_app_data, get_auth_file_path_with,
    get_nexus_api_key_command_with, get_nexus_api_key_with, handle_login_text_with,
    handle_login_ws_message_with, is_protocol_handler_registered_command_linux_with,
    is_protocol_handler_registered_linux_with, linux_home_from_env, logout_nexus_command_with,
    logout_nexus_with, nexus_ws_url, open_auth_url_with, parse_login_message_for_api_key,
    parse_nexus_ws_url_with, persist_api_key_for_login_with,
    register_nxm_protocol_command_linux_with, register_nxm_protocol_linux_default_with,
    register_nxm_protocol_linux_with, send_handshake_with,
    unregister_nxm_protocol_command_linux_with, unregister_nxm_protocol_linux_default_with,
    unregister_nxm_protocol_linux_with,
};
use futures_util::stream;
use std::path::{Path, PathBuf};
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

fn auth_json_path(app_data_dir: &Path) -> Result<PathBuf, String> {
    Ok(app_data_dir.join("auth.json"))
}

fn load_abc_key(_path: &Path) -> Result<String, String> {
    Ok("abc".to_string())
}

fn remove_auth_ok(_path: &Path) -> Result<bool, String> {
    Ok(true)
}

fn linux_home_ok() -> Result<String, String> {
    Ok("/home/deck".to_string())
}

fn remove_linux_protocol_ok(home: &str) -> Result<(), String> {
    assert_eq!(home, "/home/deck");
    Ok(())
}

fn create_dir_ok(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn write_file_ok(_path: &Path, _content: &str) -> Result<(), String> {
    Ok(())
}

fn run_xdg_mime_ok() -> Result<(), String> {
    Ok(())
}

fn ignore_api_key(_api_key: &str) -> Result<(), String> {
    Ok(())
}

fn ignore_ws_message(_message: Message) -> Result<Option<String>, String> {
    Ok(None)
}

async fn accept_hello_message(message: Message) -> Result<(), String> {
    let Message::Text(text) = message else {
        return Err("unexpected".to_string());
    };
    assert_eq!(text.as_str(), "hello");
    Ok(())
}

#[test]
fn nexus_wrappers_route_dependencies() {
    let auth = get_nexus_api_key_with(Path::new("/tmp/auth.json"), &|_p| Ok("k".to_string()))
        .expect("expected key");
    assert_eq!(auth, "k");

    let ensured = ensure_auth_path_for_app_data(Path::new("/tmp/app"), &|_dir| Ok(()));
    assert!(ensured.is_ok());
}

#[test]
fn nexus_ws_url_is_stable() {
    assert_eq!(nexus_ws_url(), "wss://sso.nexusmods.com");
}

#[test]
fn linux_home_from_env_validates_presence() {
    let home = linux_home_from_env(&|_| Some("/home/deck".to_string())).expect("home");
    assert_eq!(home, "/home/deck");
    let err = linux_home_from_env(&|_| None).expect_err("missing home should error");
    assert_eq!(err, "Could not find HOME");
}

#[test]
fn parse_login_message_for_api_key_uses_auth_parser() {
    let parsed = parse_login_message_for_api_key(r#"{"success":true,"data":{"api_key":"abc123"}}"#)
        .expect("parse should succeed");
    assert_eq!(parsed.as_deref(), Some("abc123"));
}

#[test]
fn ensure_auth_path_for_app_data_returns_path_from_provider() {
    let out = ensure_auth_path_for_app_data(Path::new("/tmp/app"), &|_dir| Ok(()))
        .expect("path should be returned");
    assert_eq!(out, PathBuf::from("/tmp/app/auth.json"));
}

#[test]
fn get_auth_file_path_with_covers_provider_and_ensure_errors() {
    let out = get_auth_file_path_with(&|| Ok(PathBuf::from("/tmp/app")), &auth_json_path)
        .expect("expected auth path");
    assert_eq!(out, PathBuf::from("/tmp/app/auth.json"));

    let err = get_auth_file_path_with(&|| Err("no-app-data".to_string()), &auth_json_path)
        .expect_err("provider error should bubble");
    assert_eq!(err, "no-app-data");

    let err = get_auth_file_path_with(&|| Ok(PathBuf::from("/tmp/app")), &|_app_data_dir| {
        Err("ensure-failed".to_string())
    })
    .expect_err("ensure error should bubble");
    assert_eq!(err, "ensure-failed");
}

#[test]
fn logout_nexus_with_forwards_to_remove_dependency() {
    let removed = logout_nexus_with(Path::new("/tmp/auth.json"), &|_path| Ok(true))
        .expect("logout should run");
    assert!(removed);

    let key =
        get_nexus_api_key_command_with(&|| Ok(PathBuf::from("/tmp/auth.json")), &load_abc_key)
            .expect("get key command wrapper");
    assert_eq!(key, "abc");

    let err = get_nexus_api_key_command_with(&|| Err("no-auth-path".to_string()), &load_abc_key)
        .expect_err("auth path failure should bubble");
    assert_eq!(err, "no-auth-path");

    let removed =
        logout_nexus_command_with(&|| Ok(PathBuf::from("/tmp/auth.json")), &remove_auth_ok)
            .expect("logout command wrapper");
    assert!(removed);

    let err = logout_nexus_command_with(&|| Err("no-auth-path".to_string()), &remove_auth_ok)
        .expect_err("logout auth path error should bubble");
    assert_eq!(err, "no-auth-path");
}

#[test]
fn nexus_command_helpers_propagate_dependency_errors() {
    let err = get_nexus_api_key_with(Path::new("/tmp/auth.json"), &|_path| {
        Err("load-failed".to_string())
    })
    .expect_err("load failure should bubble");
    assert_eq!(err, "load-failed");

    let err = get_nexus_api_key_command_with(&|| Ok(PathBuf::from("/tmp/auth.json")), &|_path| {
        Err("load-failed".to_string())
    })
    .expect_err("load failure should bubble from command wrapper");
    assert_eq!(err, "load-failed");

    let err = logout_nexus_with(Path::new("/tmp/auth.json"), &|_path| {
        Err("remove-failed".to_string())
    })
    .expect_err("remove failure should bubble");
    assert_eq!(err, "remove-failed");

    let err = logout_nexus_command_with(&|| Ok(PathBuf::from("/tmp/auth.json")), &|_path| {
        Err("remove-failed".to_string())
    })
    .expect_err("remove failure should bubble from command wrapper");
    assert_eq!(err, "remove-failed");
}

#[test]
fn linux_protocol_command_helpers_cover_success_and_error_paths() {
    let unregister_err =
        unregister_nxm_protocol_linux_with(&|| Err("no-home".to_string()), &|_home| Ok(()))
            .expect_err("missing home should fail");
    assert_eq!(unregister_err, "no-home");

    unregister_nxm_protocol_linux_with(&|| Ok("/home/deck".to_string()), &|home| {
        assert_eq!(home, "/home/deck");
        Ok(())
    })
    .expect("unregister should succeed");

    let register_err = register_nxm_protocol_linux_with(
        &|| Err("no-exe".to_string()),
        &linux_home_ok,
        &create_dir_ok,
        &write_file_ok,
        &run_xdg_mime_ok,
    )
    .expect_err("exe error should fail");
    assert_eq!(register_err, "no-exe");

    register_nxm_protocol_linux_with(
        &|| Ok(PathBuf::from("/opt/Pulsar")),
        &linux_home_ok,
        &create_dir_ok,
        &write_file_ok,
        &run_xdg_mime_ok,
    )
    .expect("register should succeed");

    unregister_nxm_protocol_linux_default_with(&linux_home_ok, &remove_linux_protocol_ok)
        .expect("default unregister should succeed");

    register_nxm_protocol_linux_default_with(
        &|| Ok(PathBuf::from("/opt/Pulsar")),
        &linux_home_ok,
        &create_dir_ok,
        &|_path, content| {
            assert!(content.contains("x-scheme-handler/nxm"));
            Ok(())
        },
        &run_xdg_mime_ok,
    )
    .expect("default register should succeed");

    unregister_nxm_protocol_command_linux_with(&linux_home_ok, &remove_linux_protocol_ok)
        .expect("unregister command wrapper should succeed");

    assert!(is_protocol_handler_registered_command_linux_with(
        &|| Some("/home/deck".to_string()),
        &|home| home == "/home/deck"
    ));

    register_nxm_protocol_command_linux_with(
        &|| Ok(PathBuf::from("/opt/Pulsar")),
        &linux_home_ok,
        &create_dir_ok,
        &write_file_ok,
        &run_xdg_mime_ok,
    )
    .expect("register command wrapper should succeed");
}

#[test]
fn linux_protocol_command_helpers_cover_more_error_paths() {
    let err = unregister_nxm_protocol_command_linux_with(
        &|| Err("no-home".to_string()),
        &remove_linux_protocol_ok,
    )
    .expect_err("command wrapper should propagate home error");
    assert_eq!(err, "no-home");

    assert!(!is_protocol_handler_registered_command_linux_with(
        &|| None,
        &|home| !home.is_empty()
    ));

    let err = register_nxm_protocol_command_linux_with(
        &|| Ok(PathBuf::from("/opt/Pulsar")),
        &|| Err("no-home".to_string()),
        &create_dir_ok,
        &write_file_ok,
        &run_xdg_mime_ok,
    )
    .expect_err("register command wrapper should propagate home error");
    assert_eq!(err, "no-home");
}

#[test]
fn handle_login_text_with_covers_parse_no_key_and_key_paths() {
    let none = handle_login_text_with("{}", &|_| Ok(None), &mut |_api_key| Ok(()))
        .expect("no key should succeed");
    assert!(none.is_none());

    let mut saved = String::new();
    let key = handle_login_text_with("k", &|_| Ok(Some("abc123".to_string())), &mut |api_key| {
        saved = api_key.to_string();
        Ok(())
    })
    .expect("key path should succeed");
    assert_eq!(key.as_deref(), Some("abc123"));
    assert_eq!(saved, "abc123");

    let ignored = handle_login_text_with(
        "k",
        &|_| Ok(Some("ignored".to_string())),
        &mut ignore_api_key,
    )
    .expect("helper callback should succeed");
    assert_eq!(ignored.as_deref(), Some("ignored"));

    let err = handle_login_text_with(
        "bad",
        &|_| Err("parse-failed".to_string()),
        &mut ignore_api_key,
    )
    .expect_err("parse failure should bubble");
    assert_eq!(err, "parse-failed");
}

#[test]
fn login_text_and_message_helpers_propagate_callback_failures() {
    let err = handle_login_text_with("k", &|_| Ok(Some("abc123".to_string())), &mut |_api_key| {
        Err("save-failed".to_string())
    })
    .expect_err("api-key callback failure should bubble");
    assert_eq!(err, "save-failed");

    let err = handle_login_ws_message_with(Message::Text("payload".into()), &|_text| {
        Err("text-failed".to_string())
    })
    .expect_err("text handler failure should bubble");
    assert_eq!(err, "text-failed");
}

#[test]
fn protocol_and_url_helpers_cover_expected_paths() {
    assert!(is_protocol_handler_registered_linux_with(
        &|| Some("/home/deck".to_string()),
        &|home| home == "/home/deck"
    ));
    assert!(!is_protocol_handler_registered_linux_with(
        &|| None,
        &|home| !home.is_empty()
    ));

    let parsed = parse_nexus_ws_url_with("wss://sso.nexusmods.com", &|url| {
        Url::parse(url).map_err(|e| e.to_string())
    })
    .expect("valid ws url");
    assert_eq!(parsed.as_str(), "wss://sso.nexusmods.com/");

    let err = parse_nexus_ws_url_with("://bad", &|url| Url::parse(url).map_err(|e| e.to_string()))
        .expect_err("invalid ws url should fail");
    assert!(!err.is_empty());
}

#[test]
fn login_message_helpers_cover_text_binary_and_persist_failures() {
    let ignored =
        ignore_ws_message(Message::Text("ignored".into())).expect("helper should return no key");
    assert!(ignored.is_none());

    let text_key = handle_login_ws_message_with(
        Message::Text(r#"{"success":true,"data":{"api_key":"k1"}}"#.into()),
        &parse_login_message_for_api_key,
    )
    .expect("text should parse");
    assert_eq!(text_key.as_deref(), Some("k1"));

    let non_text = handle_login_ws_message_with(
        Message::Binary(vec![1, 2].into()),
        &parse_login_message_for_api_key,
    )
    .expect("binary should be ignored");
    assert!(non_text.is_none());

    persist_api_key_for_login_with(
        "k2",
        &|| Ok(PathBuf::from("/tmp/auth.json")),
        &write_file_ok,
    )
    .expect("persist should succeed");

    let err =
        persist_api_key_for_login_with("k2", &|| Err("no-auth-path".to_string()), &write_file_ok)
            .expect_err("auth path error should bubble");
    assert_eq!(err, "no-auth-path");

    let err = persist_api_key_for_login_with(
        "k2",
        &|| Ok(PathBuf::from("/tmp/auth.json")),
        &|_path, _key| Err("save-failed".to_string()),
    )
    .expect_err("save error should bubble");
    assert_eq!(err, "save-failed");
}

#[test]
fn await_api_key_from_messages_with_covers_success_error_and_closed_paths() {
    tauri::async_runtime::block_on(async {
        let success_messages = stream::iter(vec![
            Ok(Message::Binary(vec![1, 2].into())),
            Ok(Message::Text(
                r#"{"success":true,"data":{"api_key":"k-ok"}}"#.into(),
            )),
        ]);
        let success = await_api_key_from_messages_with(success_messages, |message| {
            handle_login_ws_message_with(message, &parse_login_message_for_api_key)
        })
        .await
        .expect("should extract key");
        assert_eq!(success, "k-ok");

        let stream_errors = stream::iter(vec![Err("ws-failed".to_string())]);
        let stream_error = await_api_key_from_messages_with(stream_errors, ignore_ws_message)
            .await
            .expect_err("stream error should bubble");
        assert_eq!(stream_error, "ws-failed");

        let parse_errors = stream::iter(vec![Ok(Message::Text("not-json".into()))]);
        let parse_error = await_api_key_from_messages_with(parse_errors, |message| {
            handle_login_ws_message_with(message, &parse_login_message_for_api_key)
        })
        .await
        .expect_err("parse error should bubble");
        assert!(!parse_error.is_empty());

        let closed_messages = stream::iter(Vec::<Result<Message, String>>::new());
        let closed = await_api_key_from_messages_with(closed_messages, ignore_ws_message)
            .await
            .expect_err("closed stream should return terminal error");
        assert_eq!(closed, "Connection closed before authentication finished.");
    });
}

#[test]
fn handshake_and_auth_url_helpers_cover_success_and_error_paths() {
    tauri::async_runtime::block_on(async {
        let mut logs = Vec::<String>::new();
        send_handshake_with(
            "hello".to_string(),
            |level, message| logs.push(format!("{level}:{message}")),
            accept_hello_message,
        )
        .await
        .expect("handshake should succeed");
        assert!(logs.iter().any(|m| m.contains("Sending handshake: hello")));

        let err = send_handshake_with(
            "boom".to_string(),
            |_level, _message| {},
            |_message| async { Err("send-failed".to_string()) },
        )
        .await
        .expect_err("send failure should bubble");
        assert_eq!(err, "Failed to send handshake: send-failed");

        let unexpected = accept_hello_message(Message::Binary(vec![1, 2].into()))
            .await
            .expect_err("non-text handshake message should fail");
        assert_eq!(unexpected, "unexpected");
    });

    open_auth_url_with("abc", &mut |_url| Ok(())).expect("auth URL open should succeed");
    let err = open_auth_url_with("abc", &mut |_url| Err("browser-failed".to_string()))
        .expect_err("open failure should be mapped");
    assert_eq!(err, "Failed to open Nexus auth URL: browser-failed");
}

#[test]
fn open_auth_url_with_builds_expected_url() {
    let mut opened = None::<String>;
    open_auth_url_with("abc", &mut |url| {
        opened = Some(url);
        Ok(())
    })
    .expect("auth URL should be passed through");
    let opened = opened.expect("open callback should capture URL");
    assert!(opened.contains("id=abc"));
    assert!(opened.contains("application=sabrsorensen-pulsar"));
}
