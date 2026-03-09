use crate::models::ModInfo;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn invalid_json_object_error() -> String {
    "mod_info.json is not a valid JSON object.".to_string()
}

fn read_mod_info_content(path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) => Err(format!("Failed to read mod_info.json: {}", error)),
    }
}

fn parse_mod_info_json(content: &str) -> Result<Value, String> {
    match serde_json::from_str(content) {
        Ok(json_value) => Ok(json_value),
        Err(error) => Err(format!("Failed to parse mod_info.json: {}", error)),
    }
}

fn serialize_mod_info_json(json_value: &Value) -> String {
    serde_json::to_string_pretty(json_value)
        .expect("serializing mod_info.json updates should not fail")
}

fn write_mod_info_content(path: &Path, content: String) -> Result<(), String> {
    match fs::write(path, content) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("Failed to write updated mod_info.json: {}", error)),
    }
}

fn load_existing_mod_info_json(mod_info_path: &Path) -> Result<Value, String> {
    let content = match fs::read_to_string(mod_info_path) {
        Ok(content) => content,
        Err(error) => return Err(error.to_string()),
    };

    match serde_json::from_str(&content) {
        Ok(json_value) => Ok(json_value),
        Err(error) => Err(error.to_string()),
    }
}

fn serialize_plain_json(json_value: &Value) -> String {
    serde_json::to_string_pretty(json_value)
        .expect("serializing mod_info.json content should not fail")
}

pub struct EnsureModInfoInput {
    pub mod_id: String,
    pub file_id: String,
    pub version: String,
    pub install_source: String,
}

pub(crate) fn set_mod_id_field(json_value: &mut Value, new_mod_id: &str) -> Result<(), String> {
    if let Some(obj) = json_value.as_object_mut() {
        obj.insert("id".to_string(), Value::String(new_mod_id.to_string()));
        Ok(())
    } else {
        Err(invalid_json_object_error())
    }
}

pub(crate) fn apply_mod_info_input(
    json_value: &mut Value,
    input: &EnsureModInfoInput,
) -> Result<(), String> {
    if let Some(obj) = json_value.as_object_mut() {
        if !input.mod_id.is_empty() {
            obj.insert("modId".to_string(), Value::String(input.mod_id.clone()));
        }
        if !input.file_id.is_empty() {
            obj.insert("fileId".to_string(), Value::String(input.file_id.clone()));
        }
        if !input.version.is_empty() {
            obj.insert("version".to_string(), Value::String(input.version.clone()));
        }
        obj.insert(
            "installSource".to_string(),
            Value::String(input.install_source.clone()),
        );
        Ok(())
    } else {
        Err(invalid_json_object_error())
    }
}

pub fn update_mod_id_in_json_file(mod_info_path: &Path, new_mod_id: &str) -> Result<(), String> {
    if !mod_info_path.exists() {
        return Err(format!(
            "mod_info.json not found for path '{}'.",
            mod_info_path.display()
        ));
    }

    let content = read_mod_info_content(mod_info_path)?;
    let mut json_value = parse_mod_info_json(&content)?;

    set_mod_id_field(&mut json_value, new_mod_id)?;

    let new_content = serialize_mod_info_json(&json_value);
    write_mod_info_content(mod_info_path, new_content)?;
    Ok(())
}

pub fn ensure_mod_info_file(
    mod_info_path: &Path,
    input: &EnsureModInfoInput,
) -> Result<(), String> {
    let mut json_value = if mod_info_path.exists() {
        load_existing_mod_info_json(mod_info_path)?
    } else {
        json!({})
    };

    apply_mod_info_input(&mut json_value, input)?;

    let new_content = serialize_plain_json(&json_value);
    if let Err(error) = fs::write(mod_info_path, new_content) {
        return Err(error.to_string());
    }

    Ok(())
}

pub fn read_mod_info_file(mod_path: &Path) -> Option<ModInfo> {
    let info_path = mod_path.join("mod_info.json");
    if !info_path.exists() {
        return None;
    }

    fs::read_to_string(info_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

#[cfg(test)]
#[path = "info_ops_tests.rs"]
mod tests;
