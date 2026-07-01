use super::*;

pub(crate) fn reset_learning_gate(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<LearningProgress>>,
    message: &str,
) {
    let changed =
        clear_learning_completed_state_if_answer_config_changed(package_dir).unwrap_or(false);
    core.set_export_ready(has_current_learning_completed_state(package_dir));
    *learning_passed.borrow_mut() = LearningProgress::new();
    if changed {
        core.set_learning_status(SharedString::from(message));
        core.set_learning_result(SharedString::from(
            "Training progress reset because the answer configuration changed.",
        ));
    }
}

pub(crate) fn save_story_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let mut draft = read_effective_author_draft(package_dir);
    draft["storyTitle"] = json!(core.get_story_title().to_string());
    draft["summary"] = json!(core.get_story_summary().to_string());
    draft["storyPlot"] = json!(core.get_story_plot().to_string());
    draft["memoryAnchors"] = json!(split_list(core.get_memory_anchors().as_str(), "/"));
    draft["elementGroups"] = json!(split_list(core.get_element_group().as_str(), ","));
    write_current_node_to_draft(core, &mut draft);
    write_pending_author_draft(package_dir, &draft)?;
    Ok(())
}

pub(crate) fn save_temp_draft_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    save_story_from_window(core, package_dir)?;
    save_resource_from_window(core, package_dir)?;
    save_template_from_window(core, package_dir)?;
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    Ok(())
}

pub(crate) fn save_current_node_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    let mut draft = read_effective_author_draft(package_dir);
    write_current_node_to_draft(core, &mut draft);
    write_pending_author_draft(package_dir, &draft)?;
    core.set_node_overview(SharedString::from(format_node_overview(&draft)));
    set_question_overview_titles(core, &draft);
    core.set_node_output(SharedString::from(format!(
        "temporary draft saved for node {}\nnodeId={}\ntitle={}\nelementId={}\nquestion={}\n\nSaved to .tmp/author-draft.pending.json. Export promotes it only after learning test passes.",
        core.get_node_position(),
        core.get_node_id(),
        core.get_node_title(),
        core.get_element_id(),
        core.get_question_text()
    )));
    Ok(())
}

pub(crate) fn write_current_node_to_draft(core: &StoryLockCoreApp, draft: &mut Value) {
    let node_index = normalize_node_index(core.get_node_index());
    ensure_draft_nodes(draft);
    if let Some(node) = draft
        .get_mut("nodes")
        .and_then(Value::as_array_mut)
        .and_then(|nodes| nodes.get_mut(node_index))
    {
        node["nodeId"] = json!(core.get_node_id().to_string());
        node["title"] = json!(core.get_node_title().to_string());
        node["elementId"] = json!(core.get_element_id().to_string());
        node["question"] = json!(core.get_question_text().to_string());
        node["recommendedSelectionMode"] = json!(core.get_selection_mode().to_string());
        let answer_options = answer_options_from_window(core);
        let correct_count = answer_options
            .iter()
            .filter(|option| {
                option
                    .get("isCorrect")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .count() as u32;
        node["recommendedCorrectCount"] = json!(correct_count);
        node["candidatePoolSize"] = json!(answer_options.len() as u32);
        node["recallPriority"] = json!(core.get_recall_priority().to_string());
        node["verifyPolicy"] = json!(core.get_verify_policy().to_string());
        node["editorNotes"] = json!(core.get_editor_notes().to_string());
        node["canonicalAnswerLocalOnly"] = json!(core.get_canonical_answer().to_string());
        node["acceptedAnswersLocalOnly"] =
            json!(split_list(core.get_accepted_answers().as_str(), ";"));
        node["answerOptionsLocalOnly"] = json!(answer_options);
    }
}

pub(crate) fn load_node_into_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    requested_index: i32,
) {
    let node_index = normalize_node_index(requested_index);
    let mut draft = read_effective_author_draft(package_dir);
    ensure_draft_nodes(&mut draft);
    let node = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.get(node_index))
        .cloned()
        .unwrap_or_else(|| default_author_draft_json()["nodes"][node_index].clone());
    core.set_node_index(node_index as i32);
    core.set_node_position(SharedString::from(format!("{} / 24", node_index + 1)));
    core.set_selected_question(SharedString::from((node_index + 1).to_string()));
    core.set_node_id(json_string(&node, &["nodeId"]));
    core.set_node_title(json_string(&node, &["title"]));
    core.set_element_id(json_string(&node, &["elementId"]));
    core.set_question_text(json_string(&node, &["question"]));
    core.set_selection_mode(json_string(&node, &["recommendedSelectionMode"]));
    core.set_correct_count(SharedString::from(
        node.get("recommendedCorrectCount")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| {
                node.get("answerOptionsLocalOnly")
                    .and_then(Value::as_array)
                    .map(|options| {
                        options
                            .iter()
                            .filter(|option| {
                                option
                                    .get("isCorrect")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false)
                            })
                            .count()
                            .to_string()
                    })
                    .unwrap_or_else(|| "3".to_string())
            }),
    ));
    core.set_candidate_pool_size(SharedString::from(
        node.get("candidatePoolSize")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "9".to_string()),
    ));
    core.set_recall_priority(json_string(&node, &["recallPriority"]));
    core.set_verify_policy(json_string(&node, &["verifyPolicy"]));
    core.set_editor_notes(json_string(&node, &["editorNotes"]));
    core.set_canonical_answer(json_string(&node, &["canonicalAnswerLocalOnly"]));
    core.set_accepted_answers(SharedString::from(join_json_string_array(
        node.get("acceptedAnswersLocalOnly"),
        "; ",
    )));
    let answer_options = node_answer_options(&node);
    core.set_answer_options(SharedString::from(format_answer_options(&answer_options)));
    core.set_correct_options(SharedString::from(format_correct_option_indexes(
        &answer_options,
    )));
    set_answer_options_into_window(core, &answer_options);
    core.set_node_output(SharedString::from(format!(
        "loaded node {}\nnodeId={}\ntitle={}\n\nUse Save before closing. Answers and editor notes are local-core only.",
        node_index + 1,
        core.get_node_id(),
        core.get_node_title()
    )));
}

pub(crate) fn set_question_overview_titles(core: &StoryLockCoreApp, draft: &Value) {
    let nodes = draft.get("nodes").and_then(Value::as_array);
    let title = |index: usize| -> SharedString {
        let label = nodes
            .and_then(|items| items.get(index))
            .map(|node| {
                let question = node
                    .get("question")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if question.is_empty() {
                    json_string(node, &["title"]).to_string()
                } else {
                    question.to_string()
                }
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("Q{}", index + 1));
        SharedString::from(format!("{}. {}", index + 1, label))
    };

    core.set_question_1(title(0));
    core.set_question_2(title(1));
    core.set_question_3(title(2));
    core.set_question_4(title(3));
    core.set_question_5(title(4));
    core.set_question_6(title(5));
    core.set_question_7(title(6));
    core.set_question_8(title(7));
    core.set_question_9(title(8));
    core.set_question_10(title(9));
    core.set_question_11(title(10));
    core.set_question_12(title(11));
    core.set_question_13(title(12));
    core.set_question_14(title(13));
    core.set_question_15(title(14));
    core.set_question_16(title(15));
    core.set_question_17(title(16));
    core.set_question_18(title(17));
    core.set_question_19(title(18));
    core.set_question_20(title(19));
    core.set_question_21(title(20));
    core.set_question_22(title(21));
    core.set_question_23(title(22));
    core.set_question_24(title(23));
}

pub(crate) fn normalize_node_index(index: i32) -> usize {
    index.clamp(0, 23) as usize
}

pub(crate) fn ensure_draft_nodes(draft: &mut Value) {
    let needs_reset = draft
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| nodes.len() != 24)
        .unwrap_or(true);
    if needs_reset {
        draft["nodes"] = default_author_draft_json()["nodes"].clone();
    }
}

pub(crate) fn join_json_string_array(value: Option<&Value>, delimiter: &str) -> String {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(delimiter)
        })
        .unwrap_or_default()
}

pub(crate) fn format_node_overview(draft: &Value) -> String {
    draft
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    let title = node
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or("Question");
                    let question = node.get("question").and_then(Value::as_str).unwrap_or("");
                    let answer_count = node
                        .get("answerOptionsLocalOnly")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0);
                    format!(
                        "{:02}. {} | {} | {} answers",
                        index + 1,
                        title,
                        question,
                        answer_count
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|| "No question overview is available.".to_string())
}
