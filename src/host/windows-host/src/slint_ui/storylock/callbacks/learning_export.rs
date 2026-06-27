use super::*;

pub(crate) fn register_learning_export_callbacks(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: Rc<RefCell<LearningProgress>>,
    host_port: u16,
) {
    register_candidate_and_preview_callbacks(core, package_dir, host_port);
    register_learning_callbacks(core, package_dir, Rc::clone(&learning_passed));
    register_export_callback(core, package_dir);
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
) {
    let weak = core.as_weak();
    let learning_dir = package_dir.to_path_buf();
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
    let previous_learning_dir = package_dir.to_path_buf();
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
        }
    });

    let weak = core.as_weak();
    let next_learning_dir = package_dir.to_path_buf();
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
        }
    });

    let weak = core.as_weak();
    let check_learning_dir = package_dir.to_path_buf();
    let check_learning_passed = Rc::clone(&learning_passed);
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
}

fn register_export_callback(core: &StoryLockCoreApp, package_dir: &Path) {
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
                    core.set_config_status(SharedString::from(format!("Export failed: {error}")));
                }
            }
        }
    });
}
