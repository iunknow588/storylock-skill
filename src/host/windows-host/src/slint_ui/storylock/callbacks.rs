use super::*;

pub(crate) fn wire_storylock_core_callbacks(
    core: &StoryLockCoreApp,
    package_dir: std::path::PathBuf,
    core_window_slot: Rc<RefCell<Option<StoryLockCoreApp>>>,
    on_closed: Rc<dyn Fn()>,
    host_port: u16,
) {
    let learning_passed = Rc::new(RefCell::new(LearningProgress::new()));
    let answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>> = Rc::new(RefCell::new(None));
    let settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>> =
        Rc::new(RefCell::new(None));
    let weak = core.as_weak();
    let close_slot = Rc::clone(&core_window_slot);
    let on_button_closed = Rc::clone(&on_closed);
    core.on_close_requested(move || {
        if let Some(core) = weak.upgrade() {
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
            let _ = core.hide();
        }
        *window_close_slot.borrow_mut() = None;
        on_window_closed();
        slint::CloseRequestResponse::HideWindow
    });

    let weak = core.as_weak();
    let settings_dir = package_dir.clone();
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
    let browse_fallback_dir = package_dir.clone();
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
    let export_browse_fallback_dir = package_dir.clone();
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
                dialog = dialog.set_directory(default_storylock_export_dir(&export_browse_fallback_dir));
            }
            if let Some(selected_dir) = dialog.pick_folder() {
                core.set_export_package_dir(SharedString::from(selected_dir.display().to_string()));
                core.set_config_status(SharedString::from(
                    "Export directory selected for the next package export.",
                ));
            }
        }
    });

    let weak = core.as_weak();
    let temp_draft_dir = package_dir.clone();
    let temp_draft_learning_passed = Rc::clone(&learning_passed);
    core.on_save_temp_draft(move || {
        if let Some(core) = weak.upgrade() {
            if core.get_temp_draft_cooling() {
                return;
            }
            core.set_temp_draft_cooling(true);
            core.set_temp_draft_label(SharedString::from(if core.get_language().as_str() == "zh" {
                "已暂存"
            } else {
                "Saved"
            }));
            let result = ensure_storylock_core_package_dir_from_window(&core, &temp_draft_dir)
                .and_then(|package_dir| save_temp_draft_from_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &temp_draft_learning_passed,
                "Temporary draft saved. Run learning test again before export.",
            );
            set_core_status(
                &core,
                result,
                "Current StoryLock Core memory saved as temporary draft.",
            );
            let weak_for_timer = core.as_weak();
            slint::Timer::single_shot(Duration::from_millis(900), move || {
                if let Some(core) = weak_for_timer.upgrade() {
                    core.set_temp_draft_cooling(false);
                    core.set_temp_draft_label(SharedString::from(
                        if core.get_language().as_str() == "zh" {
                            "暂存草稿"
                        } else {
                            "Save Draft"
                        },
                    ));
                }
            });
        }
    });

    let weak = core.as_weak();
    let previous_node_dir = package_dir.clone();
    let previous_learning_passed = Rc::clone(&learning_passed);
    core.on_previous_node(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &previous_node_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                        return;
                    }
                };
            if let Err(error) = save_current_node_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &previous_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = core.get_node_index().saturating_sub(1);
            load_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_node_dir = package_dir.clone();
    let next_learning_passed = Rc::clone(&learning_passed);
    core.on_next_node(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &next_node_dir)
            {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                    return;
                }
            };
            if let Err(error) = save_current_node_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &next_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = (core.get_node_index() + 1).min(23);
            load_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let select_node_dir = package_dir.clone();
    let select_learning_passed = Rc::clone(&learning_passed);
    let answer_editor_for_select = Rc::clone(&answer_editor);
    core.on_select_node(move |value| {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &select_node_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                        return;
                    }
                };
            if let Err(error) = save_current_node_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &select_learning_passed,
                "Question selection saved a draft. Run learning test again before export.",
            );
            let selected_index = value
                .parse::<i32>()
                .ok()
                .map(|number| number - 1)
                .unwrap_or_else(|| core.get_node_index());
            load_node_into_window(&core, &package_dir, selected_index);
            open_answer_editor_dialog(&core, &package_dir, Rc::clone(&answer_editor_for_select));
        }
    });

    let weak = core.as_weak();
    let group_dir = package_dir.clone();
    core.on_select_resource_group(move |value| {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &group_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Workspace load failed: {error}"
                    )));
                    return;
                }
            };
            let group = normalize_resource_group(value.as_str());
            core.set_resource_group(SharedString::from(group.clone()));
            let catalog = read_json_or_default(
                &storylock_core_catalog_path(&package_dir),
                default_resource_catalog_json(),
            );
            if let Some(resource) = first_resource_for_group(&catalog, &group) {
                core.set_resource_id(json_string(resource, &["resourceId"]));
                core.set_resource_kind(json_string(resource, &["resourceKind"]));
                core.set_provider_id(json_string(resource, &["providerId"]));
                core.set_display_name(json_string(resource, &["displayName"]));
                core.set_resource_bindings(SharedString::from(format_bindings(resource)));
                core.set_object_meta(SharedString::from(format_object_meta(resource)));
            }
            core.set_protected_object_list(SharedString::from(format_protected_object_list(
                &catalog,
                &group,
            )));
            core.set_active_page(2);
        }
    });

    let weak = core.as_weak();
    let resource_dir = package_dir.clone();
    let resource_learning_passed = Rc::clone(&learning_passed);
    core.on_save_resource(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &resource_dir)
                .and_then(|package_dir| save_resource_from_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &resource_learning_passed,
                "Managed object changed. Run learning test again before export.",
            );
            set_core_status(&core, result, "Resource catalog saved locally.");
        }
    });

    let weak = core.as_weak();
    let template_dir = package_dir.clone();
    let template_learning_passed = Rc::clone(&learning_passed);
    core.on_save_template(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &template_dir)
                .and_then(|package_dir| save_template_from_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &template_learning_passed,
                "Template changed. Run learning test again before export.",
            );
            set_core_status(&core, result, "Story draft template saved locally.");
        }
    });

    let weak = core.as_weak();
    let apply_template_dir = package_dir.clone();
    let apply_template_learning_passed = Rc::clone(&learning_passed);
    core.on_apply_template(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &apply_template_dir)
                .and_then(|package_dir| apply_story_draft_template_to_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &apply_template_learning_passed,
                "Story template loaded. Run learning test again before export.",
            );
            set_core_status(&core, result, "Story draft template loaded into current UI.");
        }
    });

    let weak = core.as_weak();
    let candidate_dir = package_dir.clone();
    core.on_pull_template_candidates(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &candidate_dir)
                .and_then(|package_dir| pull_story_template_candidates_into_vault(&core, &package_dir, host_port));
            match result {
                Ok(message) => {
                    core.set_candidate_template_status(SharedString::from(message));
                    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(
                        &storylock_core_package_dir_from_window(&core, &candidate_dir),
                    )));
                    core.set_config_status(SharedString::from(
                        "Story template candidates pulled into local StoryLock templates.",
                    ));
                }
                Err(error) => {
                    core.set_candidate_template_status(SharedString::from(format!(
                        "Candidate pull failed: {error}"
                    )));
                    core.set_config_status(SharedString::from(format!(
                        "Candidate pull failed: {error}"
                    )));
                }
            }
        }
    });

    let weak = core.as_weak();
    let refresh_dir = package_dir.clone();
    core.on_refresh_export(move || {
        if let Some(core) = weak.upgrade() {
            match ensure_storylock_core_package_dir_from_window(&core, &refresh_dir) {
                Ok(package_dir) => {
                    core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
                    core.set_config_status(SharedString::from(
                        "Export preview refreshed from local StoryLock Core package.",
                    ));
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Export preview failed: {error}"
                    )));
                }
            }
        }
    });

    let weak = core.as_weak();
    let learning_policy_dir = package_dir.clone();
    core.on_save_learning_policy(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &learning_policy_dir)
                .and_then(|package_dir| save_learning_policy_from_window(&core, &package_dir));
            match result {
                Ok(()) => {
                    core.set_config_status(SharedString::from(
                        "Learning policy saved to learning-policy.json for Host execution.",
                    ));
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Learning policy save failed: {error}"
                    )));
                }
            }
        }
    });

    let weak = core.as_weak();
    let learning_dir = package_dir.clone();
    let run_learning_passed = Rc::clone(&learning_passed);
    core.on_run_learning(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &learning_dir)
            {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_result(SharedString::from(
                        "Pre-export test blocked because the workspace is invalid.",
                    ));
                    core.set_config_status(SharedString::from(format!(
                        "Workspace load failed: {error}"
                    )));
                    return;
                }
            };
            if let Err(error) = save_learning_policy_from_window(&core, &package_dir) {
                core.set_export_ready(false);
                core.set_learning_result(SharedString::from(
                    "Pre-export test blocked because the learning policy is invalid.",
                ));
                core.set_config_status(SharedString::from(format!(
                    "Learning policy save failed: {error}"
                )));
                return;
            }
            match validate_learning_test_inputs(&package_dir) {
                Ok(report) => {
                    let plan = LearningProgress::new();
                    let first_index = plan.current_node_index();
                    *run_learning_passed.borrow_mut() = plan;
                    core.set_export_ready(false);
                    load_learning_node_into_window(&core, &package_dir, first_index as i32);
                    core.set_learning_result(SharedString::from(
                        "Pre-learning started. Complete 48 prompts: each of the 24 questions appears twice.",
                    ));
                    core.set_learning_status(SharedString::from(
                        "Pre-learning active: 0 / 48 prompts checked. Export remains blocked.",
                    ));
                    core.set_config_status(SharedString::from(report));
                }
                Err(error) => {
                    *run_learning_passed.borrow_mut() = LearningProgress::new();
                    core.set_export_ready(false);
                    core.set_learning_result(SharedString::from(
                        "Pre-export test failed. Fix the local StoryLock data and run the test again.",
                    ));
                    core.set_learning_status(SharedString::from(format!(
                        "Pre-export test failed: {error}"
                    )));
                    core.set_config_status(SharedString::from(
                        "Export is blocked until the pre-export test passes.",
                    ));
                }
            }
        }
    });

    let weak = core.as_weak();
    let previous_learning_dir = package_dir.clone();
    core.on_learning_previous(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &previous_learning_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_learning_status(SharedString::from(format!(
                        "Learning load failed: {error}"
                    )));
                    return;
                }
            };
            let next_index = core.get_learning_index().saturating_sub(1);
            load_learning_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_learning_dir = package_dir.clone();
    core.on_learning_next(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &next_learning_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_learning_status(SharedString::from(format!(
                        "Learning load failed: {error}"
                    )));
                    return;
                }
            };
            let next_index = (core.get_learning_index() + 1).min(23);
            load_learning_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let check_learning_dir = package_dir.clone();
    let check_learning_passed = Rc::clone(&learning_passed);
    core.on_check_learning_current(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &check_learning_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    return;
                }
            };
            match check_learning_current(&core, &package_dir, &check_learning_passed) {
                Ok(report) => {
                    core.set_learning_status(SharedString::from(report.clone()));
                    core.set_learning_result(SharedString::from(report));
                }
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    core.set_learning_result(SharedString::from(
                        "Current answer-state match failed. Review memory and try again.",
                    ));
                }
            }
        }
    });

    let weak = core.as_weak();
    let export_dir = package_dir.clone();
    core.on_export_package(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &export_dir)
            {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Export blocked. Workspace is invalid: {error}"
                    )));
                    return;
                }
            };
            if let Err(error) = save_learning_policy_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!(
                    "Export blocked. Learning policy is invalid: {error}"
                )));
                return;
            }
            if !core.get_export_ready() {
                core.set_config_status(SharedString::from(
                    "Export blocked. Run the pre-export test successfully first.",
                ));
                return;
            }
            let export_dir = storylock_export_dir_from_window(&core, &package_dir);
            match export_storylock_package_to(&package_dir, &export_dir) {
                Ok(path) => {
                    core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
                    core.set_config_status(SharedString::from(format!(
                        "Export complete. Managed key package replaced at {}",
                        path.display()
                    )));
                    core.set_learning_status(SharedString::from(
                        "Pre-export test passed. Encrypted export completed.",
                    ));
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Export failed: {error}"
                    )));
                }
            }
        }
    });
}
