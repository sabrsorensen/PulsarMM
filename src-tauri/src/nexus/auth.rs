use serde_json::{json, Value};

pub fn handshake_payload(uuid: &str) -> Value {
    json!({
        "id": uuid,
        "token": null,
        "protocol": 2
    })
}

pub fn auth_url(uuid: &str) -> String {
    format!(
        "https://www.nexusmods.com/sso?id={}&application=sabrsorensen-pulsar",
        uuid
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
