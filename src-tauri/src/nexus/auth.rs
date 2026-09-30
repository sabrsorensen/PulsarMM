use serde_json::{json, Value};

// The app id NexusMods issued for Pulsar on registration. The SSO websocket handshake and the
// browser-facing authorize URL must use the same value (https://github.com/Nexus-Mods/node-nexus-api,
// "appid you got on registration") -- omitting it from the handshake gets "Application ID was invalid".
const NEXUS_SSO_APP_ID: &str = "sabrsorensen-pulsar";

pub fn handshake_payload(uuid: &str) -> Value {
    json!({
        "id": uuid,
        "appid": NEXUS_SSO_APP_ID,
        "token": null,
        "protocol": 2
    })
}

pub fn auth_url(uuid: &str) -> String {
    format!(
        "https://www.nexusmods.com/sso?id={}&application={}",
        uuid, NEXUS_SSO_APP_ID
    )
}

pub fn parse_api_key_message(text: &str) -> Result<Option<String>, String> {
    let response: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;

    if let Some(data) = response.get("data") {
        if let Some(api_key) = data.get("api_key").and_then(|k| k.as_str()) {
            return Ok(Some(api_key.to_string()));
        }
    }

    if let Some(success) = response.get("success").and_then(|s| s.as_bool()) {
        if !success {
            return Err("Nexus refused the connection.".to_string());
        }
    }

    Ok(None)
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
