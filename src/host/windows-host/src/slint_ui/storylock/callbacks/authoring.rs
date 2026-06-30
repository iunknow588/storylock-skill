use super::*;
use std::time::Instant;

pub(crate) fn register_authoring_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
    answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>>,
    object_editor: Rc<RefCell<Option<ObjectEditorDialog>>>,
) {
    register_temp_draft_callback(core, package_dir, Rc::clone(&learning_passed));
    register_node_navigation_callbacks(
        core,
        package_dir,
        Rc::clone(&learning_passed),
        Rc::clone(&answer_editor),
    );
    register_resource_and_template_callbacks(core, package_dir, learning_passed, object_editor);
}

fn register_temp_draft_callback(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
) {
    let weak = core.as_weak();
    let temp_draft_dir = package_dir.to_path_buf();
    core.on_save_temp_draft(move || {
        if let Some(core) = weak.upgrade() {
            if core.get_temp_draft_cooling() {
                return;
            }
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Temporary draft save") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            core.set_temp_draft_cooling(true);
            core.set_temp_draft_label(SharedString::from(
                if core.get_language().as_str() == "zh" {
                    "\u{5df2}\u{4fdd}\u{5b58}"
                } else {
                    "Saved"
                },
            ));
            let result = ensure_storylock_core_package_dir_from_window(&core, &temp_draft_dir)
                .and_then(|package_dir| {
                    let result = save_temp_draft_from_window(&core, &package_dir);
                    reset_learning_gate(
                        &core,
                        &package_dir,
                        &learning_passed,
                        "Temporary draft saved. Run learning test again before export.",
                    );
                    result
                });
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
                            "\u{4fdd}\u{5b58}\u{8349}\u{7a3f}"
                        } else {
                            "Save Draft"
                        },
                    ));
                }
            });
        }
    });
}

fn register_node_navigation_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
    answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>>,
) {
    let weak = core.as_weak();
    let previous_node_dir = package_dir.to_path_buf();
    let previous_learning_passed = Rc::clone(&learning_passed);
    core.on_previous_node(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Question navigation") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
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
                &package_dir,
                &previous_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = core.get_node_index().saturating_sub(1);
            load_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_node_dir = package_dir.to_path_buf();
    let next_learning_passed = Rc::clone(&learning_passed);
    core.on_next_node(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Question navigation") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &next_node_dir) {
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
                &package_dir,
                &next_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = (core.get_node_index() + 1).min(23);
            load_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let select_node_dir = package_dir.to_path_buf();
    let select_learning_passed = Rc::clone(&learning_passed);
    let answer_editor_for_select = Rc::clone(&answer_editor);
    core.on_select_node(move |value| {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Question selection") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
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
                &package_dir,
                &select_learning_passed,
                "Question selection saved a draft. Run learning test again before export.",
            );
            let selected_index = value
                .parse::<i32>()
                .ok()
                .map(|number| number - 1)
                .unwrap_or_else(|| core.get_node_index());
            core.set_overview_selection_enabled(true);
            load_node_into_window(&core, &package_dir, selected_index);
            open_answer_editor_dialog(&core, &package_dir, Rc::clone(&answer_editor_for_select));
        }
    });
}

fn register_resource_and_template_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    _learning_passed: Rc<RefCell<LearningProgress>>,
    object_editor: Rc<RefCell<Option<ObjectEditorDialog>>>,
) {
    let object_select_suppressed_until = Rc::new(RefCell::new(None::<Instant>));
    let weak = core.as_weak();
    let group_dir = package_dir.to_path_buf();
    core.on_select_resource_group(move |value| {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Protected object load") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &group_dir)
            {
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
            let catalog = read_protected_resources(&package_dir);
            if let Some(resource) = first_resource_for_group(&catalog, &group) {
                load_resource_into_window(&core, resource);
            }
            core.set_protected_object_list(SharedString::from(format_protected_object_list(
                &catalog, &group,
            )));
            set_protected_object_rows_into_window(&core, &catalog, &group);
            core.set_active_page(2);
        }
    });

    let weak = core.as_weak();
    let select_object_dir = package_dir.to_path_buf();
    let select_object_suppressed_until = Rc::clone(&object_select_suppressed_until);
    core.on_select_object(move |resource_id| {
        if let Some(core) = weak.upgrade() {
            if let Err(error) =
                ensure_storylock_package_unlocked(&core, "Managed object editing")
            {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            if select_object_suppressed_until
                .borrow()
                .is_some_and(|until| Instant::now() < until)
            {
                return;
            }
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &select_object_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!(
                            "Object load failed: {error}"
                        )));
                        return;
                    }
                };
            let catalog = read_protected_resources(&package_dir);
            if resource_id.trim().is_empty() {
                prepare_new_resource_in_window(&core, &catalog);
                open_object_editor_dialog(&core, &package_dir, Rc::clone(&object_editor));
                *select_object_suppressed_until.borrow_mut() =
                    Some(Instant::now() + Duration::from_millis(700));
            } else if let Some(resource) = resource_by_id(&catalog, resource_id.as_str()) {
                load_resource_into_window(&core, resource);
                open_object_editor_dialog(&core, &package_dir, Rc::clone(&object_editor));
                *select_object_suppressed_until.borrow_mut() =
                    Some(Instant::now() + Duration::from_millis(700));
            }
        }
    });

    let weak = core.as_weak();
    let resource_dir = package_dir.to_path_buf();
    core.on_save_resource(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Resource save") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            let result = ensure_storylock_core_package_dir_from_window(&core, &resource_dir)
                .and_then(|package_dir| {
                    let result = save_resource_from_window(&core, &package_dir);
                    core.set_export_ready(has_current_learning_completed_state(&package_dir));
                    result
                });
            set_core_status(&core, result, "Resource catalog saved locally.");
        }
    });

    let weak = core.as_weak();
    let template_dir = package_dir.to_path_buf();
    core.on_save_template(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Template save") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            let result = ensure_storylock_core_package_dir_from_window(&core, &template_dir)
                .and_then(|package_dir| {
                    let result = save_template_from_window(&core, &package_dir);
                    core.set_export_ready(has_current_learning_completed_state(&package_dir));
                    result
                });
            set_core_status(&core, result, "Story draft template saved locally.");
        }
    });

    let weak = core.as_weak();
    let apply_template_dir = package_dir.to_path_buf();
    core.on_apply_template(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = ensure_storylock_package_unlocked(&core, "Template apply") {
                core.set_config_status(SharedString::from(error.to_string()));
                return;
            }
            let result = ensure_storylock_core_package_dir_from_window(&core, &apply_template_dir)
                .and_then(|package_dir| {
                    let result = apply_story_draft_template_to_window(&core, &package_dir);
                    core.set_export_ready(has_current_learning_completed_state(&package_dir));
                    result
                });
            set_core_status(
                &core,
                result,
                "Story draft template loaded into current UI.",
            );
        }
    });
}
