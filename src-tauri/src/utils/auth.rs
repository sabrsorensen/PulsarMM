use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn extract_api_key_from_content(content: &str) -> Result<String, String> {
    let json: Value = serde_json::from_str(content).map_err(|_| "Invalid auth file".to_string())?;

    json.get("apikey")
        .and_then(|k| k.as_str())
        .map(str::to_string)
        .ok_or_else(|| "No API Key found. Please log in.".to_string())
}

pub(crate) fn load_api_key_from_file(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err("No API Key found. Please log in.".to_string());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    extract_api_key_from_content(&content)
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
