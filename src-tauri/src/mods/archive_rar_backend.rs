use std::path::Path;
use unrar;

pub(super) fn process_rar_archive_entries(
    abs_archive_path: &Path,
    on_progress: &mut dyn FnMut(u64),
) -> Result<(), String> {
    let mut archive = Some(
        unrar::Archive::new(abs_archive_path)
            .open_for_processing()
            .map_err(|e| format!("{:?}", e))?,
    );

    let mut step = || {
        let current = archive
            .take()
            .expect("rar archive state should exist while processing");
        match current.read_header() {
            Ok(Some(header)) => {
                archive = Some(header.extract().map_err(|e| format!("{:?}", e))?);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => Err(format!("{:?}", e)),
        }
    };

    super::run_rar_processing_loop(&mut step, on_progress)
}
