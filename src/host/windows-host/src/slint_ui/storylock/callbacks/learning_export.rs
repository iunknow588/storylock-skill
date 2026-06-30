use super::*;

pub(crate) fn register_learning_export_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
    host_port: u16,
) {
    register_candidate_and_preview_callbacks(core, package_dir, host_port);
    register_learning_callbacks(
        core,
        package_dir,
        Rc::clone(&learning_passed),
        Rc::clone(&learning_dialog),
    );
    register_export_callback(core, package_dir, learning_dialog);
}

fn begin_learning_gate(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<LearningProgress>>,
    learning_dialog: &Rc<RefCell<Option<LearningTestDialog>>>,
) -> String {
    let mut status_notes = Vec::new();
    if let Err(error) = save_temp_draft_from_window(core, package_dir) {
        status_notes.push(format!("draft save warning: {error}"));
    }
    if let Err(error) = save_learning_policy_from_window(core, package_dir) {
        status_notes.push(format!("learning policy warning: {error}"));
    }
    if let Err(error) = validate_learning_test_inputs(package_dir) {
        status_notes.push(format!("learning validation warning: {error}"));
        core.set_learning_status(SharedString::from(format!(
            "Training started, but the current draft still has issues: {error}"
        )));
    }
    let prompts_per_question = learning_prompts_per_question_from_policy(package_dir);
    let plan = LearningProgress::from_prompts_per_question(prompts_per_question);
    let total_prompts = plan.total_prompts();
    let first_index = plan.current_node_index();
    *learning_passed.borrow_mut() = plan;
    core.set_active_page(4);
    core.set_export_ready(false);
    load_learning_node_into_window(core, package_dir, first_index as i32, Some(learning_passed));
    set_learning_progress_into_window(core, 0, total_prompts, 0);
    core.set_learning_action_hint(SharedString::from(
        "Use the 3x3 grid to mark each answer as correct or wrong, then use Next.",
    ));
    core.set_learning_result(SharedString::from(format!(
        "Training started from the current draft. Complete {total_prompts} prompts: 24 questions, {prompts_per_question} prompt(s) each. Final pass/fail is checked once at the end."
    )));
    core.set_learning_status(SharedString::from(format!(
        "Training active: 0/{total_prompts} prompts checked. Export remains blocked."
    )));
    open_learning_test_dialog(core, Rc::clone(learning_dialog));
    if !status_notes.is_empty() {
        core.set_learning_result(SharedString::from(format!(
            "Training started with warnings: {}",
            status_notes.join(" ")
        )));
    }
    if status_notes.is_empty() {
        "Learning test started from the current draft.".to_string()
    } else {
        format!(
            "Learning test started from the current draft. {}",
            status_notes.join(" ")
        )
    }
}

fn sync_learning_dialog_if_open(
    core: &StoryLockCoreApp,
    learning_dialog: &Rc<RefCell<Option<LearningTestDialog>>>,
) {
    if let Some(dialog) = learning_dialog.borrow().as_ref() {
        copy_core_learning_to_dialog(core, dialog);
    }
}

fn localized_export_complete_message(core: &StoryLockCoreApp, path: &Path) -> String {
    if core.get_language().as_str() == "zh" {
        format!(
            "\u{5bfc}\u{51fa}\u{6210}\u{529f}\u{ff1a}\u{52a0}\u{5bc6}\u{6587}\u{4ef6}\u{5df2}\u{751f}\u{6210}\u{5230} {}\u{3002}\u{672c}\u{5730}\u{8349}\u{7a3f}\u{5de5}\u{4f5c}\u{533a}\u{4ecd}\u{4fdd}\u{7559}\u{5728}\u{672c}\u{673a}\u{ff0c}\u{4e0d}\u{518d}\u{9700}\u{8981}\u{65f6}\u{8bf7}\u{5220}\u{9664}\u{3002}",
            path.display()
        )
    } else {
        format!(
            "Export complete. Encrypted package generated at {}. The local draft workspace still remains on this machine; delete it if it is no longer needed.",
            path.display()
        )
    }
}

fn localized_export_failed_message(core: &StoryLockCoreApp, error: &anyhow::Error) -> String {
    if core.get_language().as_str() == "zh" {
        format!("\u{5bfc}\u{51fa}\u{5931}\u{8d25}\u{ff1a}{error}")
    } else {
        format!("Export failed: {error}")
    }
}

fn run_encrypted_export_after_learning(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_dialog: &Rc<RefCell<Option<LearningTestDialog>>>,
) {
    if !has_current_learning_completed_state(package_dir) {
        core.set_export_ready(false);
        core.set_config_status(SharedString::from(
            "Encrypted save requires a passed nine-grid confirmation first.",
        ));
        sync_learning_dialog_if_open(core, learning_dialog);
        return;
    }
    core.set_export_ready(true);
    if let Err(error) = save_learning_policy_from_window(core, package_dir) {
        let message = localized_export_failed_message(core, &error);
        core.set_config_status(SharedString::from(message));
        sync_learning_dialog_if_open(core, learning_dialog);
        return;
    }
    let export_dir = storylock_export_dir_from_window(core, package_dir);
    match export_storylock_package_to(package_dir, &export_dir) {
        Ok(path) => {
            let message = localized_export_complete_message(core, &path);
            core.set_export_ready(has_current_learning_completed_state(package_dir));
            core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
            core.set_config_status(SharedString::from(message));
            sync_learning_dialog_if_open(core, learning_dialog);
        }
        Err(error) => {
            core.set_export_ready(has_current_learning_completed_state(package_dir));
            let message = localized_export_failed_message(core, &error);
            core.set_config_status(SharedString::from(message));
            sync_learning_dialog_if_open(core, learning_dialog);
        }
    }
}

fn register_candidate_and_preview_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    host_port: u16,
) {
    let weak = core.as_weak();
    let candidate_dir = package_dir.to_path_buf();
    core.on_pull_template_candidates(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &candidate_dir)
                .and_then(|package_dir| {
                    pull_story_template_candidates_into_vault(&core, &package_dir, host_port)
                });
            match result {
                Ok(message) => {
                    core.set_candidate_template_status(SharedString::from(message));
                    core.set_template_bindings(SharedString::from(
                        format_story_draft_template_summary(
                            &storylock_core_package_dir_from_window(&core, &candidate_dir),
                        ),
                    ));
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
    let refresh_dir = package_dir.to_path_buf();
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
    let learning_policy_dir = package_dir.to_path_buf();
    core.on_save_learning_policy(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &learning_policy_dir)
                .and_then(|package_dir| save_learning_policy_from_window(&core, &package_dir));
            match result {
                Ok(()) => {
                    let package_dir =
                        storylock_core_package_dir_from_window(&core, &learning_policy_dir);
                    core.set_export_ready(has_current_learning_completed_state(&package_dir));
                    core.set_config_status(SharedString::from(
                        "Learning policy saved. Existing learning pass state is unchanged.",
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
}

fn register_learning_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
) {
    let weak = core.as_weak();
    let learning_dir = package_dir.to_path_buf();
    let run_learning_passed = Rc::clone(&learning_passed);
    let run_learning_dialog = Rc::clone(&learning_dialog);
    core.on_run_learning(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &learning_dir) {
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
            let report = begin_learning_gate(
                &core,
                &package_dir,
                &run_learning_passed,
                &run_learning_dialog,
            );
            core.set_config_status(SharedString::from(report));
            sync_learning_dialog_if_open(&core, &run_learning_dialog);
        }
    });

    let weak = core.as_weak();
    let previous_learning_dir = package_dir.to_path_buf();
    let previous_learning_passed = Rc::clone(&learning_passed);
    let previous_learning_dialog = Rc::clone(&learning_dialog);
    core.on_learning_previous(move || {
        if let Some(core) = weak.upgrade() {
            if previous_learning_passed.borrow().checked() == 0 {
                core.set_learning_result(SharedString::from(
                    "Already at the first prompt; Previous is disabled.",
                ));
                sync_learning_dialog_if_open(&core, &previous_learning_dialog);
                return;
            }
            cache_current_learning_answers(&core, &previous_learning_passed);
            let package_dir = match ensure_storylock_core_package_dir_from_window(
                &core,
                &previous_learning_dir,
            ) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_learning_status(SharedString::from(format!(
                        "Learning load failed: {error}"
                    )));
                    return;
                }
            };
            retreat_learning_cursor(&core, &package_dir, &previous_learning_passed);
            sync_learning_dialog_if_open(&core, &previous_learning_dialog);
        }
    });

    let weak = core.as_weak();
    let next_learning_dir = package_dir.to_path_buf();
    let next_learning_passed = Rc::clone(&learning_passed);
    let next_learning_dialog = Rc::clone(&learning_dialog);
    core.on_learning_next(move || {
        if let Some(core) = weak.upgrade() {
            {
                let progress = next_learning_passed.borrow();
                if progress.checked() >= progress.total_prompts() {
                    core.set_learning_result(SharedString::from(
                        "Training is already complete; Next is disabled.",
                    ));
                    core.set_learning_action_hint(SharedString::from(
                        "Training completed. You can export now.",
                    ));
                    core.set_export_ready(true);
                    sync_learning_dialog_if_open(&core, &next_learning_dialog);
                    return;
                }
            }
            cache_current_learning_answers(&core, &next_learning_passed);
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &next_learning_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_learning_status(SharedString::from(format!(
                            "Learning load failed: {error}"
                        )));
                        return;
                    }
                };
            match check_learning_current(&core, &package_dir, &next_learning_passed) {
                Ok(report) => {
                    core.set_learning_status(SharedString::from(report.clone()));
                    core.set_learning_result(SharedString::from(report));
                    sync_learning_dialog_if_open(&core, &next_learning_dialog);
                }
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    core.set_learning_result(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    sync_learning_dialog_if_open(&core, &next_learning_dialog);
                }
            }
        }
    });
}

fn register_export_callback(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
) {
    let weak = core.as_weak();
    let export_dir = package_dir.to_path_buf();
    core.on_export_package(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &export_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!(
                            "Export blocked. Workspace is invalid: {error}"
                        )));
                        return;
                    }
            };
            if !core.get_export_ready() {
                core.set_config_status(SharedString::from(
                    "Export blocked. Run and pass the nine-grid test first.",
                ));
                core.set_learning_status(SharedString::from(
                    "Export is locked until the current story answers pass the nine-grid test.",
                ));
                core.set_learning_result(SharedString::from(
                    "Please click Start Test first. After the test passes, Export can be clicked repeatedly.",
                ));
                sync_learning_dialog_if_open(&core, &learning_dialog);
                core.set_active_page(4);
                return;
            }
            core.set_config_status(SharedString::from(if core.get_language().as_str() == "zh" {
                "\u{6b63}\u{5728}\u{5bfc}\u{51fa}\u{52a0}\u{5bc6}\u{6587}\u{4ef6}\u{2026}"
            } else {
                "Exporting encrypted package..."
            }));
            run_encrypted_export_after_learning(&core, &package_dir, &learning_dialog);
        }
    });
}
