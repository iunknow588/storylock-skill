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
    register_export_callback(core, package_dir, learning_passed, learning_dialog);
}

fn begin_learning_gate(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<LearningProgress>>,
    learning_dialog: &Rc<RefCell<Option<LearningTestDialog>>>,
) -> Result<String> {
    save_temp_draft_from_window(core, package_dir)?;
    save_learning_policy_from_window(core, package_dir)?;
    let report = validate_learning_test_inputs(package_dir)?;
    let prompts_per_question = learning_prompts_per_question_from_policy(package_dir);
    let plan = LearningProgress::from_prompts_per_question(prompts_per_question);
    let total_prompts = plan.total_prompts();
    let first_index = plan.current_node_index();
    *learning_passed.borrow_mut() = plan;
    core.set_active_page(4);
    core.set_export_ready(false);
    load_learning_node_into_window(core, package_dir, first_index as i32);
    set_learning_progress_into_window(core, 0, total_prompts, 0);
    core.set_learning_action_hint(SharedString::from(
        "Use the 3x3 grid to mark each answer as correct or wrong. Final pass/fail is checked after all prompts are finished.",
    ));
    core.set_learning_result(SharedString::from(format!(
        "Training started from the current draft. Complete {total_prompts} prompts: 24 questions, {prompts_per_question} prompt(s) each, then final pass/fail will be checked once."
    )));
    core.set_learning_status(SharedString::from(format!(
        "Training active: 0 / {total_prompts} prompts checked. Encrypted save remains blocked."
    )));
    open_learning_test_dialog(core, Rc::clone(learning_dialog));
    Ok(report)
}

fn sync_learning_dialog_if_open(
    core: &StoryLockCoreApp,
    learning_dialog: &Rc<RefCell<Option<LearningTestDialog>>>,
) {
    if let Some(dialog) = learning_dialog.borrow().as_ref() {
        copy_core_learning_to_dialog(core, dialog);
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
            match begin_learning_gate(
                &core,
                &package_dir,
                &run_learning_passed,
                &run_learning_dialog,
            ) {
                Ok(report) => {
                    core.set_config_status(SharedString::from(report));
                    sync_learning_dialog_if_open(&core, &run_learning_dialog);
                }
                Err(error) => {
                    *run_learning_passed.borrow_mut() = LearningProgress::new();
                    core.set_active_page(4);
                    core.set_export_ready(false);
                    core.set_learning_progress_summary(SharedString::from(
                        "0 / 0 prompts completed, errors recorded: 0",
                    ));
                    core.set_learning_action_hint(SharedString::from(
                        "Fix the current draft first, then start training again.",
                    ));
                    core.set_learning_result(SharedString::from(
                        "Training could not start. Review the current question set and continue learning the correct answers.",
                    ));
                    core.set_learning_status(SharedString::from(format!(
                        "Training setup failed: {error}"
                    )));
                    core.set_config_status(SharedString::from(
                        "Encrypted save is blocked until the nine-grid training passes.",
                    ));
                    sync_learning_dialog_if_open(&core, &run_learning_dialog);
                }
            }
        }
    });

    let weak = core.as_weak();
    let previous_learning_dir = package_dir.to_path_buf();
    let previous_learning_dialog = Rc::clone(&learning_dialog);
    core.on_learning_previous(move || {
        if let Some(core) = weak.upgrade() {
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
            let next_index = core.get_learning_index().saturating_sub(1);
            load_learning_node_into_window(&core, &package_dir, next_index);
            sync_learning_dialog_if_open(&core, &previous_learning_dialog);
        }
    });

    let weak = core.as_weak();
    let next_learning_dir = package_dir.to_path_buf();
    let next_learning_dialog = Rc::clone(&learning_dialog);
    core.on_learning_next(move || {
        if let Some(core) = weak.upgrade() {
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
            let next_index = (core.get_learning_index() + 1).min(23);
            load_learning_node_into_window(&core, &package_dir, next_index);
            sync_learning_dialog_if_open(&core, &next_learning_dialog);
        }
    });

    let weak = core.as_weak();
    let check_learning_dir = package_dir.to_path_buf();
    let check_learning_passed = Rc::clone(&learning_passed);
    let check_learning_dialog = Rc::clone(&learning_dialog);
    core.on_check_learning_current(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &check_learning_dir) {
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
                    sync_learning_dialog_if_open(&core, &check_learning_dialog);
                }
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    core.set_learning_result(SharedString::from(
                        "Current answer-state match failed. Review memory and try again.",
                    ));
                    sync_learning_dialog_if_open(&core, &check_learning_dialog);
                }
            }
        }
    });

    let weak = core.as_weak();
    let reveal_learning_dir = package_dir.to_path_buf();
    let reveal_learning_dialog = Rc::clone(&learning_dialog);
    core.on_reveal_learning_answer(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &reveal_learning_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_learning_status(SharedString::from(format!(
                            "Learning answer load failed: {error}"
                        )));
                        return;
                    }
                };
            match reveal_learning_answer_for_current(&core, &package_dir) {
                Ok(report) => {
                    core.set_learning_action_hint(SharedString::from(
                        "Correct answers are visible now. Review them, then restart training when ready.",
                    ));
                    core.set_learning_result(SharedString::from(report));
                    sync_learning_dialog_if_open(&core, &reveal_learning_dialog);
                }
                Err(error) => {
                    core.set_learning_result(SharedString::from(format!(
                        "Could not reveal the correct answers: {error}"
                    )));
                    sync_learning_dialog_if_open(&core, &reveal_learning_dialog);
                }
            }
        }
    });
}

fn register_export_callback(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
) {
    let weak = core.as_weak();
    let export_dir = package_dir.to_path_buf();
    let export_learning_passed = Rc::clone(&learning_passed);
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
                match begin_learning_gate(
                    &core,
                    &package_dir,
                    &export_learning_passed,
                    &learning_dialog,
                ) {
                    Ok(report) => {
                        core.set_config_status(SharedString::from(report));
                        sync_learning_dialog_if_open(&core, &learning_dialog);
                    }
                    Err(error) => {
                        core.set_learning_result(SharedString::from(
                            "Nine-grid confirmation has not passed. Continue learning the correct answers before encrypted save.",
                        ));
                        core.set_learning_status(SharedString::from(format!(
                            "Encrypted save blocked: {error}"
                        )));
                        core.set_config_status(SharedString::from(
                            "Encrypted save requires a passed nine-grid confirmation first.",
                        ));
                        sync_learning_dialog_if_open(&core, &learning_dialog);
                    }
                }
                core.set_active_page(4);
                return;
            }
            if let Err(error) = save_learning_policy_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(
                    format!("Export blocked. Learning policy is invalid: {error}")
                ));
                return;
            }
            let export_dir = storylock_export_dir_from_window(&core, &package_dir);
            match export_storylock_package_to(&package_dir, &export_dir) {
                Ok(path) => {
                    core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
                    core.set_config_status(SharedString::from(format!(
                        "Export complete. Managed key package replaced at {}. The local draft workspace still remains on this machine. Delete it if it is no longer needed; keeping it is your own risk.",
                        path.display()
                    )));
                    core.set_learning_status(SharedString::from(
                        "Nine-grid confirmation passed. Encrypted export completed. Reminder: delete the local draft workspace if you no longer need it.",
                    ));
                    core.set_learning_action_hint(SharedString::from(
                        "Export completed. Delete the local draft workspace if you no longer need it.",
                    ));
                    core.set_learning_result(SharedString::from(
                        "Encrypted save finished. The export used the current draft that passed training. If you keep the local draft workspace, you accept the remaining local disclosure risk yourself.",
                    ));
                    sync_learning_dialog_if_open(&core, &learning_dialog);
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!("Export failed: {error}")));
                    sync_learning_dialog_if_open(&core, &learning_dialog);
                }
            }
        }
    });
}
