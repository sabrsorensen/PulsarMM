use std::path::PathBuf;

pub fn parse_steam_library_folders(content: &str) -> Vec<PathBuf> {
    let mut libraries = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.contains("\"path\"") {
            if let Some(path_str) = trimmed.split('"').nth(3) {
                libraries.push(PathBuf::from(path_str.replace("\\\\", "/")));
            }
            continue;
        }

        let parts: Vec<&str> = trimmed.split('"').collect();
        if parts.len() >= 4 {
            let key = parts[1];
            if key.chars().all(|c| c.is_ascii_digit()) {
                libraries.push(PathBuf::from(parts[3].replace("\\\\", "/")));
            }
        }
    }

    libraries
}

pub fn extract_installdir_from_manifest(content: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.contains("\"installdir\""))
        .and_then(|l| l.split('"').nth(3))
        .map(|s| s.to_string())
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
