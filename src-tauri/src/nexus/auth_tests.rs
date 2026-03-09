use super::*;

#[test]
fn handshake_payload_uses_expected_contract() {
    let payload = handshake_payload("abc-123");
    assert_eq!(payload["id"], "abc-123");
    assert_eq!(payload["protocol"], 2);
    assert!(payload["token"].is_null());
}

#[test]
fn auth_url_contains_uuid_and_app_id() {
    let url = auth_url("abc-123");
    assert!(url.contains("id=abc-123"));
    assert!(url.contains("application=sabrsorensen-pulsar"));
}

#[test]
fn parse_api_key_message_extracts_key() {
    let msg = r#"{"data":{"api_key":"secret-key"}}"#;
    let key = parse_api_key_message(msg).expect("api key message should parse");
    assert_eq!(key.as_deref(), Some("secret-key"));
}

#[test]
fn parse_api_key_message_errors_on_refusal() {
    let msg = r#"{"success":false}"#;
    assert!(parse_api_key_message(msg).is_err());
}

#[test]
fn parse_api_key_message_ignores_irrelevant_success_messages() {
    let msg = r#"{"success":true}"#;
    assert_eq!(
        parse_api_key_message(msg).expect("success message should parse"),
        None
    );
}

#[test]
fn parse_api_key_message_rejects_invalid_json() {
    assert!(parse_api_key_message("not-json").is_err());
}

#[test]
fn parse_api_key_message_returns_none_for_missing_or_non_string_key() {
    assert_eq!(
        parse_api_key_message(r#"{"data":{}}"#).expect("missing key should parse"),
        None
    );
    assert_eq!(
        parse_api_key_message(r#"{"data":{"api_key":123}}"#).expect("non-string key should parse"),
        None
    );
}
