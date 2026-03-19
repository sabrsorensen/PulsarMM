use crate::mods::install_scan::scan_for_installable_mods;
use std::fs;
use std::path::{Path, PathBuf};

pub fn copy_archive_to_downloads(
    archive_path: &Path,
    downloads_dir: &Path,
) -> Result<(PathBuf, bool), String> {
    if !downloads_dir.exists() {
        fs::create_dir_all(downloads_dir).map_err(|e| {
            format!(
                "Failed to create downloads directory '{}': {}",
                downloads_dir.display(),
                e
            )
        })?;
    }

    let in_downloads = if let (Ok(archive_canon), Ok(downloads_canon)) =
        (archive_path.canonicalize(), downloads_dir.canonicalize())
    {
        archive_canon.starts_with(downloads_canon)
    } else {
        false
    };

    if in_downloads {
        return Ok((archive_path.to_path_buf(), true));
    }

    let file_name = archive_path
        .file_name()
        .ok_or("Invalid filename".to_string())?;
    let target_path = downloads_dir.join(file_name);
    fs::copy(archive_path, &target_path).map_err(|e| {
        format!(
            "Failed to copy archive from '{}' to '{}': {}",
            archive_path.display(),
            target_path.display(),
            e
        )
    })?;
    Ok((target_path, false))
}

pub fn library_folder_name_for_archive(archive_path: &Path) -> Result<String, String> {
    let zip_name = archive_path
        .file_name()
        .ok_or("Invalid archive filename".to_string())?
        .to_string_lossy()
        .into_owned();
    Ok(format!("{}_unpacked", zip_name))
}

pub fn scan_library_mod_path(
    library_mod_path: &Path,
) -> Result<(Vec<String>, Vec<String>), String> {
    let installable_paths = scan_for_installable_mods(library_mod_path, library_mod_path);
    let folder_names = fs::read_dir(library_mod_path)
        .map_err(|e| {
            format!(
                "Failed to read extracted archive directory '{}': {}",
                library_mod_path.display(),
                e
            )
        })?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    Ok((folder_names, installable_paths))
}

#[cfg(test)]
#[path = "install_archive_flow_tests.rs"]
mod tests;
