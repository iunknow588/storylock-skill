use super::*;

pub(crate) fn set_core_status(core: &StoryLockCoreApp, result: Result<()>, success_message: &str) {
    match result {
        Ok(()) => {
            core.set_config_status(SharedString::from(success_message));
            core.set_export_preview(SharedString::from(build_export_preview(Path::new(
                core.get_core_data_dir().as_str(),
            ))));
        }
        Err(error) => core.set_config_status(SharedString::from(format!("Save failed: {error}"))),
    }
}

pub(crate) fn open_answer_editor_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>>,
) {
    if answer_editor.borrow().is_none() {
        match AnswerEditorDialog::new() {
            Ok(dialog) => {
                wire_answer_editor_callbacks(&dialog, core.as_weak(), package_dir.to_path_buf());
                *answer_editor.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Answer editor failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = answer_editor.borrow().as_ref() {
        copy_core_question_to_answer_editor(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Answer editor failed to show: {error}"
            )));
        }
    }
}

pub(crate) fn open_storylock_core_settings_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    if settings_dialog.borrow().is_none() {
        match StoryLockCoreSettingsDialog::new() {
            Ok(dialog) => {
                wire_storylock_core_settings_callbacks(
                    &dialog,
                    core.as_weak(),
                    package_dir.to_path_buf(),
                    Rc::clone(&settings_dialog),
                );
                *settings_dialog.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Settings failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = settings_dialog.borrow().as_ref() {
        copy_core_settings_to_dialog(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Settings failed to show: {error}"
            )));
        }
    }
}

pub(crate) fn open_object_editor_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    object_editor: Rc<RefCell<Option<ObjectEditorDialog>>>,
) {
    if object_editor.borrow().is_none() {
        match ObjectEditorDialog::new() {
            Ok(dialog) => {
                wire_object_editor_callbacks(
                    &dialog,
                    core.as_weak(),
                    package_dir.to_path_buf(),
                    Rc::clone(&object_editor),
                );
                *object_editor.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Object editor failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = object_editor.borrow().as_ref() {
        copy_core_object_to_dialog(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Object editor failed to show: {error}"
            )));
        }
    }
}

pub(crate) fn open_learning_test_dialog(
    core: &StoryLockCoreApp,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
) {
    if learning_dialog.borrow().is_none() {
        match LearningTestDialog::new() {
            Ok(dialog) => {
                wire_learning_test_dialog_callbacks(
                    &dialog,
                    core.as_weak(),
                    Rc::clone(&learning_dialog),
                );
                *learning_dialog.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Learning dialog failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = learning_dialog.borrow().as_ref() {
        copy_core_learning_to_dialog(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Learning dialog failed to show: {error}"
            )));
        }
    }
}

pub(crate) fn wire_object_editor_callbacks(
    dialog: &ObjectEditorDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
    object_editor: Rc<RefCell<Option<ObjectEditorDialog>>>,
) {
    let close_slot = Rc::clone(&object_editor);
    let core_for_save = core_weak.clone();
    let save_dir = package_dir.clone();
    let weak = dialog.as_weak();
    dialog.on_save_requested(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_save.upgrade()) {
            copy_dialog_object_to_core(&dialog, &core);
            match save_object_editor_resource_from_window(&core, &save_dir) {
                Ok(()) => {
                    core.set_config_status(SharedString::from("Managed object saved."));
                    copy_core_object_to_dialog(&core, &dialog);
                }
                Err(error) => {
                    let message = SharedString::from(format!("Object save failed: {error}"));
                    core.set_config_status(message.clone());
                }
            }
        }
    });

    let weak = dialog.as_weak();
    let close_slot_for_button = Rc::clone(&close_slot);
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        *close_slot_for_button.borrow_mut() = None;
    });

    let weak = dialog.as_weak();
    let close_slot_for_window = close_slot;
    dialog.window().on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        *close_slot_for_window.borrow_mut() = None;
        slint::CloseRequestResponse::HideWindow
    });
}

pub(crate) fn wire_learning_test_dialog_callbacks(
    dialog: &LearningTestDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
) {
    let weak = dialog.as_weak();
    let core_for_restart = core_weak.clone();
    dialog.on_restart_learning(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_restart.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_run_learning();
        }
    });

    let weak = dialog.as_weak();
    let core_for_reveal = core_weak.clone();
    dialog.on_reveal_learning_answer(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_reveal.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_reveal_learning_answer();
        }
    });

    let weak = dialog.as_weak();
    let core_for_previous = core_weak.clone();
    dialog.on_learning_previous(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_previous.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_learning_previous();
        }
    });

    let weak = dialog.as_weak();
    let core_for_next = core_weak.clone();
    dialog.on_learning_next(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_next.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_learning_next();
        }
    });

    let weak = dialog.as_weak();
    let core_for_check = core_weak;
    dialog.on_check_learning_current(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_check.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_check_learning_current();
        }
    });

    let weak = dialog.as_weak();
    let close_slot_for_button = Rc::clone(&learning_dialog);
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        *close_slot_for_button.borrow_mut() = None;
    });

    let weak = dialog.as_weak();
    let close_slot_for_window = learning_dialog;
    dialog.window().on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        *close_slot_for_window.borrow_mut() = None;
        slint::CloseRequestResponse::HideWindow
    });
}

pub(crate) fn wire_storylock_core_settings_callbacks(
    dialog: &StoryLockCoreSettingsDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    let close_slot = Rc::clone(&settings_dialog);
    let core_for_close = core_weak.clone();
    let close_settings = Rc::new(move |dialog: &StoryLockCoreSettingsDialog| {
        if let Some(core) = core_for_close.upgrade() {
            copy_dialog_settings_to_core(dialog, &core);
            if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core)) {
                core.set_config_status(SharedString::from(format!(
                    "Settings save failed: {error}"
                )));
            }
        }
        let _ = dialog.hide();
        *close_slot.borrow_mut() = None;
    });

    let weak = dialog.as_weak();
    let close_settings_for_button = Rc::clone(&close_settings);
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            close_settings_for_button(&dialog);
        }
    });

    let weak = dialog.as_weak();
    let close_settings_for_window = Rc::clone(&close_settings);
    dialog.window().on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            close_settings_for_window(&dialog);
        }
        slint::CloseRequestResponse::HideWindow
    });

    let weak = dialog.as_weak();
    let core_for_language = core_weak.clone();
    dialog.on_language_changed(move |language| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_language.upgrade()) {
            core.set_language(language);
            copy_core_settings_to_dialog(&core, &dialog);
            if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core)) {
                core.set_config_status(SharedString::from(format!(
                    "Settings save failed: {error}"
                )));
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_browse = core_weak.clone();
    let browse_fallback_dir = package_dir.clone();
    dialog.on_browse_core_data_dir(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_browse.upgrade()) {
            copy_dialog_settings_to_core(&dialog, &core);
            let current_dir = storylock_core_package_dir_from_window(&core, &browse_fallback_dir);
            let mut file_dialog = rfd::FileDialog::new();
            if current_dir.exists() {
                file_dialog = file_dialog.set_directory(&current_dir);
            }
            if let Some(selected_dir) = file_dialog.pick_folder() {
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
                        copy_core_settings_to_dialog(&core, &dialog);
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
}

pub(crate) fn wire_answer_editor_callbacks(
    dialog: &AnswerEditorDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
) {
    let weak = dialog.as_weak();
    let core_for_window_close = core_weak.clone();
    let close_dir = package_dir.clone();
    dialog.window().on_close_requested(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_window_close.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            match save_current_node_from_window(&core, &close_dir) {
                Ok(()) => core
                    .set_config_status(SharedString::from("Answer editor saved current question.")),
                Err(error) => core.set_config_status(SharedString::from(format!(
                    "Answer editor save failed: {error}"
                ))),
            }
            let _ = dialog.hide();
        }
        slint::CloseRequestResponse::HideWindow
    });

    let weak = dialog.as_weak();
    let core_for_previous = core_weak.clone();
    let previous_dir = package_dir.clone();
    dialog.on_previous_node(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_previous.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &previous_dir).is_ok() {
                let next_index = core.get_node_index().saturating_sub(1);
                load_node_into_window(&core, &previous_dir, next_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_next = core_weak.clone();
    let next_dir = package_dir.clone();
    dialog.on_next_node(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_next.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &next_dir).is_ok() {
                let next_index = (core.get_node_index() + 1).min(23);
                load_node_into_window(&core, &next_dir, next_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_select = core_weak;
    let select_dir = package_dir;
    dialog.on_select_node(move |value| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_select.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &select_dir).is_ok() {
                let selected_index = value
                    .parse::<i32>()
                    .ok()
                    .map(|number| number - 1)
                    .unwrap_or_else(|| core.get_node_index());
                load_node_into_window(&core, &select_dir, selected_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });
}

pub(crate) fn copy_core_settings_to_dialog(
    core: &StoryLockCoreApp,
    dialog: &StoryLockCoreSettingsDialog,
) {
    dialog.set_language(core.get_language());
    dialog.set_core_data_dir(core.get_core_data_dir());
}

pub(crate) fn copy_dialog_settings_to_core(
    dialog: &StoryLockCoreSettingsDialog,
    core: &StoryLockCoreApp,
) {
    core.set_language(dialog.get_language());
    core.set_core_data_dir(dialog.get_core_data_dir());
}

pub(crate) fn copy_core_question_to_answer_editor(
    core: &StoryLockCoreApp,
    dialog: &AnswerEditorDialog,
) {
    dialog.set_language(core.get_language());
    dialog.set_selected_question(core.get_selected_question());
    dialog.set_question_text(core.get_question_text());
    dialog.set_answer_1(core.get_answer_1());
    dialog.set_answer_1_state(core.get_answer_1_state());
    dialog.set_answer_2(core.get_answer_2());
    dialog.set_answer_2_state(core.get_answer_2_state());
    dialog.set_answer_3(core.get_answer_3());
    dialog.set_answer_3_state(core.get_answer_3_state());
    dialog.set_answer_4(core.get_answer_4());
    dialog.set_answer_4_state(core.get_answer_4_state());
    dialog.set_answer_5(core.get_answer_5());
    dialog.set_answer_5_state(core.get_answer_5_state());
    dialog.set_answer_6(core.get_answer_6());
    dialog.set_answer_6_state(core.get_answer_6_state());
    dialog.set_answer_7(core.get_answer_7());
    dialog.set_answer_7_state(core.get_answer_7_state());
    dialog.set_answer_8(core.get_answer_8());
    dialog.set_answer_8_state(core.get_answer_8_state());
    dialog.set_answer_9(core.get_answer_9());
    dialog.set_answer_9_state(core.get_answer_9_state());
}

pub(crate) fn copy_answer_editor_to_core(dialog: &AnswerEditorDialog, core: &StoryLockCoreApp) {
    core.set_selected_question(dialog.get_selected_question());
    core.set_question_text(dialog.get_question_text());
    core.set_answer_1(dialog.get_answer_1());
    core.set_answer_1_state(dialog.get_answer_1_state());
    core.set_answer_2(dialog.get_answer_2());
    core.set_answer_2_state(dialog.get_answer_2_state());
    core.set_answer_3(dialog.get_answer_3());
    core.set_answer_3_state(dialog.get_answer_3_state());
    core.set_answer_4(dialog.get_answer_4());
    core.set_answer_4_state(dialog.get_answer_4_state());
    core.set_answer_5(dialog.get_answer_5());
    core.set_answer_5_state(dialog.get_answer_5_state());
    core.set_answer_6(dialog.get_answer_6());
    core.set_answer_6_state(dialog.get_answer_6_state());
    core.set_answer_7(dialog.get_answer_7());
    core.set_answer_7_state(dialog.get_answer_7_state());
    core.set_answer_8(dialog.get_answer_8());
    core.set_answer_8_state(dialog.get_answer_8_state());
    core.set_answer_9(dialog.get_answer_9());
    core.set_answer_9_state(dialog.get_answer_9_state());
}

pub(crate) fn copy_core_object_to_dialog(core: &StoryLockCoreApp, dialog: &ObjectEditorDialog) {
    dialog.set_language(core.get_language());
    dialog.set_uri(core.get_display_name());
    dialog.set_username(core.get_provider_id());
    dialog.set_password(core.get_secret_reference());
    dialog.set_show_password(false);
}

pub(crate) fn copy_dialog_object_to_core(dialog: &ObjectEditorDialog, core: &StoryLockCoreApp) {
    core.set_display_name(dialog.get_uri());
    core.set_provider_id(dialog.get_username());
    core.set_secret_reference(dialog.get_password());
    core.set_object_kind(SharedString::from("password_fill"));
}

pub(crate) fn copy_core_learning_to_dialog(core: &StoryLockCoreApp, dialog: &LearningTestDialog) {
    dialog.set_language(core.get_language());
    dialog.set_learning_plan_summary(core.get_learning_plan_summary());
    dialog.set_learning_status(core.get_learning_status());
    dialog.set_learning_progress_summary(core.get_learning_progress_summary());
    dialog.set_learning_action_hint(core.get_learning_action_hint());
    dialog.set_learning_position(core.get_learning_position());
    dialog.set_learning_question(core.get_learning_question());
    dialog.set_learning_answer_1(core.get_learning_answer_1());
    dialog.set_learning_answer_1_state(core.get_learning_answer_1_state());
    dialog.set_learning_answer_2(core.get_learning_answer_2());
    dialog.set_learning_answer_2_state(core.get_learning_answer_2_state());
    dialog.set_learning_answer_3(core.get_learning_answer_3());
    dialog.set_learning_answer_3_state(core.get_learning_answer_3_state());
    dialog.set_learning_answer_4(core.get_learning_answer_4());
    dialog.set_learning_answer_4_state(core.get_learning_answer_4_state());
    dialog.set_learning_answer_5(core.get_learning_answer_5());
    dialog.set_learning_answer_5_state(core.get_learning_answer_5_state());
    dialog.set_learning_answer_6(core.get_learning_answer_6());
    dialog.set_learning_answer_6_state(core.get_learning_answer_6_state());
    dialog.set_learning_answer_7(core.get_learning_answer_7());
    dialog.set_learning_answer_7_state(core.get_learning_answer_7_state());
    dialog.set_learning_answer_8(core.get_learning_answer_8());
    dialog.set_learning_answer_8_state(core.get_learning_answer_8_state());
    dialog.set_learning_answer_9(core.get_learning_answer_9());
    dialog.set_learning_answer_9_state(core.get_learning_answer_9_state());
    dialog.set_learning_result(core.get_learning_result());
}

pub(crate) fn copy_dialog_learning_to_core(dialog: &LearningTestDialog, core: &StoryLockCoreApp) {
    core.set_learning_answer_1(dialog.get_learning_answer_1());
    core.set_learning_answer_1_state(dialog.get_learning_answer_1_state());
    core.set_learning_answer_2(dialog.get_learning_answer_2());
    core.set_learning_answer_2_state(dialog.get_learning_answer_2_state());
    core.set_learning_answer_3(dialog.get_learning_answer_3());
    core.set_learning_answer_3_state(dialog.get_learning_answer_3_state());
    core.set_learning_answer_4(dialog.get_learning_answer_4());
    core.set_learning_answer_4_state(dialog.get_learning_answer_4_state());
    core.set_learning_answer_5(dialog.get_learning_answer_5());
    core.set_learning_answer_5_state(dialog.get_learning_answer_5_state());
    core.set_learning_answer_6(dialog.get_learning_answer_6());
    core.set_learning_answer_6_state(dialog.get_learning_answer_6_state());
    core.set_learning_answer_7(dialog.get_learning_answer_7());
    core.set_learning_answer_7_state(dialog.get_learning_answer_7_state());
    core.set_learning_answer_8(dialog.get_learning_answer_8());
    core.set_learning_answer_8_state(dialog.get_learning_answer_8_state());
    core.set_learning_answer_9(dialog.get_learning_answer_9());
    core.set_learning_answer_9_state(dialog.get_learning_answer_9_state());
}
