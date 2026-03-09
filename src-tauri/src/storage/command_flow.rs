use super::logic::{clear_dirs_in_dir, clear_files_in_dir, select_special_folder_path};
use super::mutations::{apply_downloads_path_change, apply_library_path_change};
use std::path::{Path, PathBuf};

fn clear_downloads_and_library_with_impl(
    downloads_path: &Path,
    library_path: &Path,
    clear_files: &mut dyn FnMut(&Path) -> Result<usize, String>,
    clear_dirs: &mut dyn FnMut(&Path) -> Result<usize, String>,
) -> Result<(), String> {
    let _ = clear_files(downloads_path)?;
    let _ = clear_dirs(library_path)?;
    Ok(())
}

pub fn clear_downloads_and_library_with(
    downloads_path: &Path,
    library_path: &Path,
    clear_files: impl FnOnce(&Path) -> Result<usize, String>,
    clear_dirs: impl FnOnce(&Path) -> Result<usize, String>,
) -> Result<(), String> {
    let mut clear_files = Some(clear_files);
    let mut clear_dirs = Some(clear_dirs);
    clear_downloads_and_library_with_impl(
        downloads_path,
        library_path,
        &mut |path| clear_files.take().expect("clear_files called once")(path),
        &mut |path| clear_dirs.take().expect("clear_dirs called once")(path),
    )
}

fn set_downloads_path_with_impl(
    old_path: &Path,
    new_path: &str,
    config_path: &Path,
    log: &mut dyn FnMut(&str, &str),
) -> Result<(), String> {
    log(
        "INFO",
        &format!(
            "Changing Downloads Path. Old: {:?}, New: {}",
            old_path, new_path
        ),
    );
    let user_selected_root = PathBuf::from(new_path);
    if let Err(err) = apply_downloads_path_change(old_path, &user_selected_root, config_path) {
        log("WARN", &err);
        return Err(err);
    }
    log("INFO", "Downloads path updated successfully.");
    Ok(())
}

pub fn set_downloads_path_with(
    old_path: &Path,
    new_path: &str,
    config_path: &Path,
    mut log: impl FnMut(&str, &str),
) -> Result<(), String> {
    set_downloads_path_with_impl(old_path, new_path, config_path, &mut log)
}

fn set_library_path_with_impl(
    old_path: &Path,
    new_path: &str,
    config_path: &Path,
) -> Result<(), String> {
    let user_selected_root = PathBuf::from(new_path);
    let _ = apply_library_path_change(old_path, &user_selected_root, config_path)?;
    Ok(())
}

pub fn set_library_path_with(
    old_path: &Path,
    new_path: &str,
    config_path: &Path,
) -> Result<(), String> {
    set_library_path_with_impl(old_path, new_path, config_path)
}

fn open_special_folder_with_impl(
    folder_type: &str,
    downloads_dir: PathBuf,
    profiles_dir: PathBuf,
    library_dir: PathBuf,
    open_path: &mut dyn FnMut(PathBuf) -> Result<(), String>,
) -> Result<(), String> {
    let path = select_special_folder_path(folder_type, downloads_dir, profiles_dir, library_dir)?;
    open_path(path)
}

pub fn open_special_folder_with(
    folder_type: &str,
    downloads_dir: PathBuf,
    profiles_dir: PathBuf,
    library_dir: PathBuf,
    open_path: impl FnOnce(PathBuf) -> Result<(), String>,
) -> Result<(), String> {
    let mut open_path = Some(open_path);
    open_special_folder_with_impl(
        folder_type,
        downloads_dir,
        profiles_dir,
        library_dir,
        &mut |path| open_path.take().expect("open_path called once")(path),
    )
}

pub fn clear_downloads_and_library(
    downloads_path: &Path,
    library_path: &Path,
) -> Result<(), String> {
    clear_downloads_and_library_with(
        downloads_path,
        library_path,
        clear_files_in_dir,
        clear_dirs_in_dir,
    )
}

#[cfg(test)]
#[path = "command_flow_tests.rs"]
mod tests;
