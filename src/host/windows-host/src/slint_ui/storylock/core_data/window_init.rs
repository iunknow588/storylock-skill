use super::*;

pub(crate) fn initialize_storylock_core_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let vault = read_storylock_vault_payload(package_dir);
    let draft = storylock_author_draft_from_vault(&vault);
    let catalog = read_protected_resources(package_dir);
    core.set_package_unlocked(true);
    core.set_core_data_dir(SharedString::from(package_dir.display().to_string()));
    core.set_draft_file_path(SharedString::from("vault.stlk"));
    core.set_manifest_file_path(SharedString::from("package-manifest.json"));
    core.set_encrypted_vault_path(SharedString::from("vault.stlk"));
    core.set_resource_catalog_path(SharedString::from("resource-catalog.json"));
    core.set_learning_policy_path(SharedString::from("learning-policy.json"));
    core.set_export_package_dir(SharedString::from(default_storylock_export_dir(package_dir).display().to_string()));
    core.set_story_title(json_string(&draft, &["storyTitle"]));
    core.set_story_summary(json_string(&draft, &["summary"]));
    core.set_story_plot(json_string(&draft, &["storyPlot"]));
    core.set_template_id(json_string(&draft, &["templateId"]));
    core.set_memory_anchors(SharedString::from(
        draft
            .get("memoryAnchors")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .unwrap_or_default(),
    ));
    core.set_element_group(SharedString::from(
        draft
            .get("elementGroups")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default(),
    ));
    core.set_node_overview(SharedString::from(format_node_overview(&draft)));
    set_question_overview_titles(core, &draft);
    load_node_into_window(core, package_dir, core.get_node_index());
    if let Some(resource) = catalog
        .get("resources")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
    {
        load_resource_into_window(core, resource);
    }
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        core.get_resource_group().as_str(),
    )));
    set_protected_object_rows_into_window(&core, &catalog, core.get_resource_group().as_str());
    core.set_template_display_name(json_string(&draft, &["storyTitle"]));
    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(
        package_dir,
    )));
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    core.set_candidate_template_status(SharedString::from(
        "Host can queue candidates; StoryLock must pull them explicitly.",
    ));
    if has_current_learning_completed_state(package_dir) {
        core.set_learning_result(SharedString::from(
            "Learning has already passed for the current story answers. You can export directly.",
        ));
        core.set_learning_status(SharedString::from(
            "Learning completed for current content. Export is enabled.",
        ));
        core.set_export_ready(true);
    } else {
        core.set_learning_result(SharedString::from(
            "Run the pre-export test after finishing edits and policy changes.",
        ));
        core.set_learning_status(SharedString::from(
            "Pre-export test idle. Export stays blocked until the test passes.",
        ));
        core.set_export_ready(false);
    }
    load_learning_policy_into_window(core, package_dir);
    core.set_config_status(SharedString::from(format!(
        "StoryLock Core package ready at {} | vault {} | export {}",
        package_dir.display(),
        storylock_core_vault_path(package_dir).display(),
        default_storylock_export_dir(package_dir).display()
    )));
    set_storylock_start_page_to_questions(core);
    schedule_storylock_start_page_to_questions(core);
}

pub(crate) fn initialize_storylock_core_empty_window(core: &StoryLockCoreApp, package_dir: &Path) {
    core.set_package_unlocked(false);
    core.set_core_data_dir(SharedString::from(package_dir.display().to_string()));
    core.set_draft_file_path(SharedString::from("vault.stlk"));
    core.set_manifest_file_path(SharedString::from("package-manifest.json"));
    core.set_encrypted_vault_path(SharedString::from("vault.stlk"));
    core.set_resource_catalog_path(SharedString::from("resource-catalog.json"));
    core.set_learning_policy_path(SharedString::from("learning-policy.json"));
    core.set_export_package_dir(SharedString::from(
        default_storylock_export_dir(package_dir)
            .display()
            .to_string(),
    ));

    core.set_story_title(SharedString::from(""));
    core.set_story_summary(SharedString::from(""));
    core.set_story_plot(SharedString::from(""));
    core.set_template_id(SharedString::from(""));
    core.set_memory_anchors(SharedString::from(""));
    core.set_element_group(SharedString::from(""));
    core.set_node_overview(SharedString::from(
        "Package locked. Unlock the current package to load story questions.",
    ));
    set_locked_question_overview_titles(core);
    set_locked_question_editor_state(core);
    set_locked_resource_state(core);
    set_locked_learning_state(core);
    core.set_template_display_name(SharedString::from(""));
    core.set_template_bindings(SharedString::from(
        "Package locked. Unlock the current package to load template details.",
    ));
    core.set_export_preview(SharedString::from(
        "Package locked. Unlock the current package to view export preview.",
    ));
    core.set_candidate_template_status(SharedString::from(
        "Package locked. Unlock the current package before pulling template candidates.",
    ));
    core.set_config_status(SharedString::from(format!(
        "StoryLock Core opened in empty mode. Current package target is {}. Unlock the current package before loading package content.",
        package_dir.display()
    )));
    core.set_active_page(1);
    core.set_overview_selection_enabled(false);
}

pub(crate) fn ensure_storylock_package_unlocked(
    core: &StoryLockCoreApp,
    action: &str,
) -> Result<()> {
    if core.get_package_unlocked() {
        Ok(())
    } else {
        anyhow::bail!(
            "{action} blocked. Unlock the current package before loading or editing current package content."
        )
    }
}

fn set_locked_question_overview_titles(core: &StoryLockCoreApp) {
    let label = |index: usize| SharedString::from(format!("{}. Locked", index + 1));
    core.set_question_1(label(0));
    core.set_question_2(label(1));
    core.set_question_3(label(2));
    core.set_question_4(label(3));
    core.set_question_5(label(4));
    core.set_question_6(label(5));
    core.set_question_7(label(6));
    core.set_question_8(label(7));
    core.set_question_9(label(8));
    core.set_question_10(label(9));
    core.set_question_11(label(10));
    core.set_question_12(label(11));
    core.set_question_13(label(12));
    core.set_question_14(label(13));
    core.set_question_15(label(14));
    core.set_question_16(label(15));
    core.set_question_17(label(16));
    core.set_question_18(label(17));
    core.set_question_19(label(18));
    core.set_question_20(label(19));
    core.set_question_21(label(20));
    core.set_question_22(label(21));
    core.set_question_23(label(22));
    core.set_question_24(label(23));
}

fn set_locked_question_editor_state(core: &StoryLockCoreApp) {
    core.set_node_index(0);
    core.set_node_position(SharedString::from("0 / 24"));
    core.set_selected_question(SharedString::from("1"));
    core.set_node_id(SharedString::from(""));
    core.set_node_title(SharedString::from(""));
    core.set_element_id(SharedString::from(""));
    core.set_question_text(SharedString::from(
        "Package locked. Unlock the current package to load question text.",
    ));
    core.set_selection_mode(SharedString::from(""));
    core.set_correct_count(SharedString::from(""));
    core.set_candidate_pool_size(SharedString::from(""));
    core.set_recall_priority(SharedString::from(""));
    core.set_verify_policy(SharedString::from(""));
    core.set_editor_notes(SharedString::from(""));
    core.set_canonical_answer(SharedString::from(""));
    core.set_accepted_answers(SharedString::from(""));
    core.set_answer_options(SharedString::from(""));
    core.set_correct_options(SharedString::from(""));
    core.set_answer_1(SharedString::from(""));
    core.set_answer_1_state(SharedString::from("wrong"));
    core.set_answer_2(SharedString::from(""));
    core.set_answer_2_state(SharedString::from("wrong"));
    core.set_answer_3(SharedString::from(""));
    core.set_answer_3_state(SharedString::from("wrong"));
    core.set_answer_4(SharedString::from(""));
    core.set_answer_4_state(SharedString::from("wrong"));
    core.set_answer_5(SharedString::from(""));
    core.set_answer_5_state(SharedString::from("wrong"));
    core.set_answer_6(SharedString::from(""));
    core.set_answer_6_state(SharedString::from("wrong"));
    core.set_answer_7(SharedString::from(""));
    core.set_answer_7_state(SharedString::from("wrong"));
    core.set_answer_8(SharedString::from(""));
    core.set_answer_8_state(SharedString::from("wrong"));
    core.set_answer_9(SharedString::from(""));
    core.set_answer_9_state(SharedString::from("wrong"));
    core.set_node_output(SharedString::from(
        "Package locked. Unlock the current package before editing question content.",
    ));
}

fn set_locked_resource_state(core: &StoryLockCoreApp) {
    let catalog = default_resource_catalog_json();
    core.set_resource_group(SharedString::from("normal"));
    core.set_editing_resource_group(SharedString::from("normal"));
    core.set_resource_id(SharedString::from(""));
    core.set_resource_kind(SharedString::from(""));
    core.set_provider_id(SharedString::from(""));
    core.set_display_name(SharedString::from(""));
    core.set_object_id(SharedString::from(""));
    core.set_object_kind(SharedString::from(""));
    core.set_required_correct_count(SharedString::from(""));
    core.set_authorization_frequency(SharedString::from(""));
    core.set_secret_reference(SharedString::from(""));
    core.set_resource_bindings(SharedString::from(""));
    core.set_object_meta(SharedString::from(""));
    core.set_protected_object_list(SharedString::from(
        "Package locked. Unlock the current package to load protected objects.",
    ));
    set_protected_object_rows_into_window(core, &catalog, "normal");
}

fn set_locked_learning_state(core: &StoryLockCoreApp) {
    core.set_learning_plan_summary(SharedString::from(
        "Package locked. Unlock the current package to load learning policy.",
    ));
    core.set_learning_status(SharedString::from(
        "Package locked. Learning is unavailable until the current package is unlocked.",
    ));
    core.set_learning_progress_summary(SharedString::from(""));
    core.set_learning_total_questions(24);
    core.set_learning_current_question(1);
    core.set_learning_checked_prompts(0);
    core.set_learning_total_prompts(0);
    core.set_learning_error_count(0);
    core.set_learning_progress_percent(0);
    core.set_learning_progress_headline(SharedString::from(""));
    core.set_learning_action_hint(SharedString::from(
        "Unlock the current package before starting learning.",
    ));
    core.set_export_ready(false);
    core.set_learning_index(0);
    core.set_learning_position(SharedString::from("0 / 0"));
    core.set_learning_question(SharedString::from(""));
    core.set_learning_result(SharedString::from(
        "Package locked. Export and learning remain unavailable.",
    ));
    core.set_learning_answer_1(SharedString::from(""));
    core.set_learning_answer_1_state(SharedString::from("wrong"));
    core.set_learning_answer_2(SharedString::from(""));
    core.set_learning_answer_2_state(SharedString::from("wrong"));
    core.set_learning_answer_3(SharedString::from(""));
    core.set_learning_answer_3_state(SharedString::from("wrong"));
    core.set_learning_answer_4(SharedString::from(""));
    core.set_learning_answer_4_state(SharedString::from("wrong"));
    core.set_learning_answer_5(SharedString::from(""));
    core.set_learning_answer_5_state(SharedString::from("wrong"));
    core.set_learning_answer_6(SharedString::from(""));
    core.set_learning_answer_6_state(SharedString::from("wrong"));
    core.set_learning_answer_7(SharedString::from(""));
    core.set_learning_answer_7_state(SharedString::from("wrong"));
    core.set_learning_answer_8(SharedString::from(""));
    core.set_learning_answer_8_state(SharedString::from("wrong"));
    core.set_learning_answer_9(SharedString::from(""));
    core.set_learning_answer_9_state(SharedString::from("wrong"));
}

pub(crate) fn set_storylock_start_page_to_questions(core: &StoryLockCoreApp) {
    core.set_overview_selection_enabled(false);
    core.set_active_page(1);
}

pub(crate) fn schedule_storylock_start_page_to_questions(core: &StoryLockCoreApp) {
    let weak = core.as_weak();
    slint::Timer::single_shot(Duration::from_millis(0), move || {
        if let Some(core) = weak.upgrade() {
            set_storylock_start_page_to_questions(&core);
        }
    });
}
