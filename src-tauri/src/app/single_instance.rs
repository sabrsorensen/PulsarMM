pub(crate) fn handle_single_instance_with(
    argv: &[String],
    find_nxm: &dyn Fn(&[String]) -> Option<String>,
    emit_nxm_link: &mut dyn FnMut(String),
    restore_focus: &mut dyn FnMut() -> Result<(), String>,
    log_warn: &mut dyn FnMut(&str),
) {
    if let Some(nxm_link) = find_nxm(argv) {
        emit_nxm_link(nxm_link);
    }
    if let Err(e) = restore_focus() {
        log_warn(&e);
    }
}

pub(crate) fn run_single_instance_flow_with(
    argv: &[String],
    log_info: &mut dyn FnMut(&str),
    find_nxm: &dyn Fn(&[String]) -> Option<String>,
    emit_nxm_link: &mut dyn FnMut(String),
    restore_focus: &mut dyn FnMut() -> Result<(), String>,
    log_warn: &mut dyn FnMut(&str),
) {
    log_info(&format!("New instance detected, args: {:?}", argv));
    let mut wrapped_log_warn = |message: &str| {
        log_warn(&format!(
            "single-instance window activation failed: {}",
            message
        ))
    };
    handle_single_instance_with(
        argv,
        find_nxm,
        emit_nxm_link,
        restore_focus,
        &mut wrapped_log_warn,
    );
}

pub(crate) fn restore_focus_for_main_window_with(
    unminimize: &mut dyn FnMut() -> Result<(), String>,
    set_focus: &mut dyn FnMut() -> Result<(), String>,
) -> Result<(), String> {
    unminimize()?;
    set_focus()?;
    Ok(())
}

#[cfg(test)]
#[path = "single_instance_tests.rs"]
mod tests;
