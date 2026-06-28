use super::*;

pub(crate) fn register_lifecycle_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    core_window_slot: Rc<RefCell<Option<StoryLockCoreApp>>>,
    on_closed: Rc<dyn Fn()>,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    let weak = core.as_weak();
    let close_slot = Rc::clone(&core_window_slot);
    let on_button_closed = Rc::clone(&on_closed);
    core.on_close_requested(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core)) {
                eprintln!("failed to save StoryLock UI settings: {error}");
            }
            let _ = core.hide();
        }
        *close_slot.borrow_mut() = None;
        on_button_closed();
    });

    let weak = core.as_weak();
    let window_close_slot = Rc::clone(&core_window_slot);
    let on_window_closed = Rc::clone(&on_closed);
    core.window().on_close_requested(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core)) {
                eprintln!("failed to save StoryLock UI settings: {error}");
            }
            let _ = core.hide();
        }
        *window_close_slot.borrow_mut() = None;
        on_window_closed();
        slint::CloseRequestResponse::HideWindow
    });

    let weak = core.as_weak();
    let settings_dir = package_dir.to_path_buf();
    let settings_dialog_for_open = Rc::clone(&settings_dialog);
    core.on_open_core_settings(move || {
        if let Some(core) = weak.upgrade() {
            open_storylock_core_settings_dialog(
                &core,
                &settings_dir,
                Rc::clone(&settings_dialog_for_open),
            );
        }
    });

    let weak = core.as_weak();
    let browse_fallback_dir = package_dir.to_path_buf();
    core.on_browse_core_data_dir(move || {
        if let Some(core) = weak.upgrade() {
            let current_dir = storylock_core_package_dir_from_window(&core, &browse_fallback_dir);
            let mut dialog = rfd::FileDialog::new();
            if current_dir.exists() {
                dialog = dialog.set_directory(&current_dir);
            }
            if let Some(selected_dir) = dialog.pick_folder() {
                match ensure_storylock_core_package(&selected_dir) {
                    Ok(()) => {
                        initialize_storylock_core_window(&core, &selected_dir);
                        if let Err(error) =
                            save_storylock_ui_settings(&settings_from_storylock_core(&core))
                        {
                            core.set_config_status(SharedString::from(format!(
                                "Settings save failed: {error}"
                            )));
                        }
                        core.set_config_status(SharedString::from(
                            "StoryLock Core workspace loaded from selected directory.",
                        ));
                    }
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!(
                            "Workspace load failed: {error}"
                        )));
                    }
                }
            }
        }
    });

    let weak = core.as_weak();
    let export_browse_fallback_dir = package_dir.to_path_buf();
    core.on_browse_export_package_dir(move || {
        if let Some(core) = weak.upgrade() {
            let current = core.get_export_package_dir();
            let current_trimmed = current.as_str().trim();
            let mut dialog = rfd::FileDialog::new();
            if !current_trimmed.is_empty() {
                let current_path = std::path::PathBuf::from(current_trimmed);
                if current_path.exists() {
                    dialog = dialog.set_directory(current_path);
                }
            } else {
                dialog =
                    dialog.set_directory(default_storylock_export_dir(&export_browse_fallback_dir));
            }
            if let Some(selected_dir) = dialog.pick_folder() {
                core.set_export_package_dir(SharedString::from(selected_dir.display().to_string()));
                if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core))
                {
                    core.set_config_status(SharedString::from(format!(
                        "Settings save failed: {error}"
                    )));
                    return;
                }
                core.set_config_status(SharedString::from(
                    "Export directory selected for the next package export.",
                ));
            }
        }
    });
}
