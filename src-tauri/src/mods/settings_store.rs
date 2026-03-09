use crate::models::SettingsData;
use crate::xml_format;
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use std::fs;
use std::path::Path;

pub fn parse_settings(xml_content: &str) -> Result<SettingsData, String> {
    match from_str(xml_content) {
        Ok(settings) => Ok(settings),
        Err(error) => Err(format!("Failed to parse GCMODSETTINGS.MXML: {}", error)),
    }
}

pub fn load_settings_file(path: &Path) -> Result<SettingsData, String> {
    let xml_content = match fs::read_to_string(path) {
        Ok(xml_content) => xml_content,
        Err(error) => return Err(format!("Failed to read GCMODSETTINGS.MXML: {}", error)),
    };

    parse_settings(&xml_content)
}

pub fn to_formatted_xml(root: &SettingsData) -> Result<String, String> {
    let unformatted_xml = to_string(root).expect("serializing SettingsData to XML should not fail");

    xml_format::format_pulsar_xml(&unformatted_xml)
}

pub fn save_settings_file(path: &Path, root: &SettingsData) -> Result<(), String> {
    let final_content = to_formatted_xml(root)?;
    match fs::write(path, final_content) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!(
            "Failed to save updated GCMODSETTINGS.MXML: {}",
            error
        )),
    }
}

#[cfg(test)]
#[path = "settings_store_tests.rs"]
mod tests;
