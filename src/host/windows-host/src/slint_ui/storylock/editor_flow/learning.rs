use super::*;

pub(crate) fn load_learning_node_into_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    requested_index: i32,
    learning_passed: Option<&Rc<RefCell<LearningProgress>>>,
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
    let options = node_answer_options(&node);
    core.set_learning_index(node_index as i32);
    core.set_selected_question(SharedString::from((node_index + 1).to_string()));
    core.set_learning_total_questions(24);
    core.set_learning_current_question((node_index + 1) as i32);
    update_learning_prompt_position(core);
    update_learning_progress_headline(core);
    core.set_learning_question(json_string(&node, &["question"]));
    set_learning_answers_into_window(core, &options);
    if let Some(learning_passed) = learning_passed {
        restore_cached_learning_answers_into_window(core, learning_passed, node_index);
    }
    core.set_learning_result(SharedString::from(format!(
        "Question {} loaded. Mark each answer as correct or wrong from memory, then use Next.",
        node_index + 1
    )));
}

pub(crate) fn set_learning_answers_into_window(core: &StoryLockCoreApp, options: &[Value]) {
    let (text, _) = option_text_and_state(options, 0);
    core.set_learning_answer_1(text);
    core.set_learning_answer_1_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 1);
    core.set_learning_answer_2(text);
    core.set_learning_answer_2_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 2);
    core.set_learning_answer_3(text);
    core.set_learning_answer_3_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 3);
    core.set_learning_answer_4(text);
    core.set_learning_answer_4_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 4);
    core.set_learning_answer_5(text);
    core.set_learning_answer_5_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 5);
    core.set_learning_answer_6(text);
    core.set_learning_answer_6_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 6);
    core.set_learning_answer_7(text);
    core.set_learning_answer_7_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 7);
    core.set_learning_answer_8(text);
    core.set_learning_answer_8_state(binary_state_text(false));
    let (text, _) = option_text_and_state(options, 8);
    core.set_learning_answer_9(text);
    core.set_learning_answer_9_state(binary_state_text(false));
}

pub(crate) fn set_learning_answer_states_into_window(core: &StoryLockCoreApp, states: &[bool]) {
    core.set_learning_answer_1_state(binary_state_text(states.get(0).copied().unwrap_or(false)));
    core.set_learning_answer_2_state(binary_state_text(states.get(1).copied().unwrap_or(false)));
    core.set_learning_answer_3_state(binary_state_text(states.get(2).copied().unwrap_or(false)));
    core.set_learning_answer_4_state(binary_state_text(states.get(3).copied().unwrap_or(false)));
    core.set_learning_answer_5_state(binary_state_text(states.get(4).copied().unwrap_or(false)));
    core.set_learning_answer_6_state(binary_state_text(states.get(5).copied().unwrap_or(false)));
    core.set_learning_answer_7_state(binary_state_text(states.get(6).copied().unwrap_or(false)));
    core.set_learning_answer_8_state(binary_state_text(states.get(7).copied().unwrap_or(false)));
    core.set_learning_answer_9_state(binary_state_text(states.get(8).copied().unwrap_or(false)));
}

pub(crate) fn toggle_learning_answer_state(core: &StoryLockCoreApp, answer_index: usize) {
    let mut states = learning_answer_states_from_window(core);
    if let Some(state) = states.get_mut(answer_index) {
        *state = !*state;
        set_learning_answer_states_into_window(core, &states);
    }
}

pub(crate) fn learning_answer_states_from_window(core: &StoryLockCoreApp) -> Vec<bool> {
    [
        core.get_learning_answer_1_state(),
        core.get_learning_answer_2_state(),
        core.get_learning_answer_3_state(),
        core.get_learning_answer_4_state(),
        core.get_learning_answer_5_state(),
        core.get_learning_answer_6_state(),
        core.get_learning_answer_7_state(),
        core.get_learning_answer_8_state(),
        core.get_learning_answer_9_state(),
    ]
    .into_iter()
    .map(|state| state.as_str().eq_ignore_ascii_case("correct"))
    .collect()
}

pub(crate) fn cache_current_learning_answers(
    core: &StoryLockCoreApp,
    learning_passed: &Rc<RefCell<LearningProgress>>,
) {
    let node_index = normalize_node_index(core.get_learning_index());
    let states = learning_answer_states_from_window(core);
    learning_passed
        .borrow_mut()
        .cache_answers_for_question(node_index, &states);
}

pub(crate) fn load_learning_cursor_into_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<LearningProgress>>,
) {
    let node_index = learning_passed.borrow().current_node_index();
    load_learning_node_into_window(core, package_dir, node_index as i32, Some(learning_passed));
}

pub(crate) fn retreat_learning_cursor(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<LearningProgress>>,
) {
    let (checked, total, errors) = {
        let mut progress = learning_passed.borrow_mut();
        progress.retreat();
        (progress.checked, progress.total_prompts(), progress.errors)
    };
    load_learning_cursor_into_window(core, package_dir, learning_passed);
    set_learning_progress_into_window(core, checked, total, errors);
}

pub(crate) fn restore_cached_learning_answers_into_window(
    core: &StoryLockCoreApp,
    learning_passed: &Rc<RefCell<LearningProgress>>,
    question_index: usize,
) {
    let states = learning_passed
        .borrow()
        .cached_answers_for_question(question_index);
    set_learning_answer_states_into_window(core, &states);
}

pub(crate) fn set_learning_progress_into_window(
    core: &StoryLockCoreApp,
    checked: usize,
    total: usize,
    errors: usize,
) {
    let checked = checked.min(total);
    let percent = if total == 0 {
        0
    } else {
        ((checked * 100) / total).min(100)
    };
    core.set_learning_checked_prompts(checked as i32);
    core.set_learning_total_prompts(total as i32);
    core.set_learning_error_count(errors as i32);
    core.set_learning_progress_percent(percent as i32);
    update_learning_prompt_position(core);
    update_learning_progress_headline(core);
    core.set_learning_progress_summary(SharedString::from(format!(
        "{checked}/{total} prompts completed, errors: {errors}"
    )));
}

fn update_learning_prompt_position(core: &StoryLockCoreApp) {
    let total_prompts = core.get_learning_total_prompts().max(1);
    let checked_prompts = core.get_learning_checked_prompts().max(0);
    let current_prompt = (checked_prompts + 1).min(total_prompts);
    core.set_learning_position(SharedString::from(format!(
        "{current_prompt}/{total_prompts}"
    )));
}

fn update_learning_progress_headline(core: &StoryLockCoreApp) {
    let total_questions = core.get_learning_total_questions().max(24);
    let current_question = core
        .get_learning_current_question()
        .clamp(1, total_questions);
    let checked_prompts = core.get_learning_checked_prompts().max(0);
    let total_prompts = core.get_learning_total_prompts().max(1);
    let current_prompt = (checked_prompts + 1).min(total_prompts);
    let errors = core.get_learning_error_count().max(0);
    let headline = if core.get_language().as_str() == "zh" {
        format!(
            "\u{603b}\u{9898}\u{6570} {total_questions}  |  \u{5f53}\u{524d}\u{8fdb}\u{5ea6} {current_prompt}/{total_prompts}  |  \u{5f53}\u{524d}\u{9898}\u{76ee} {current_question}/{total_questions}  |  \u{9519}\u{8bef} {errors}"
        )
    } else {
        format!(
            "Questions {total_questions}  |  Progress {current_prompt}/{total_prompts}  |  Current Question {current_question}/{total_questions}  |  Errors {errors}"
        )
    };
    core.set_learning_progress_headline(SharedString::from(headline));
}

#[derive(Clone)]
pub(crate) struct LearningProgress {
    pub(crate) plan: Vec<usize>,
    cursor: usize,
    checked: usize,
    errors: usize,
    failed_by_question: Vec<usize>,
    answer_state_cache: Vec<Vec<bool>>,
}

impl LearningProgress {
    pub(crate) fn new() -> Self {
        Self::from_prompts_per_question(2)
    }

    pub(crate) fn from_prompts_per_question(prompts_per_question: usize) -> Self {
        let repeats = prompts_per_question.max(1);
        let mut plan = Vec::with_capacity(24 * repeats);
        for _round in 0..repeats {
            for offset in 0..24 {
                plan.push(offset);
            }
        }
        Self {
            plan,
            cursor: 0,
            checked: 0,
            errors: 0,
            failed_by_question: vec![0; 24],
            answer_state_cache: vec![vec![false; 9]; 24],
        }
    }

    pub(crate) fn current_node_index(&self) -> usize {
        self.plan.get(self.cursor).copied().unwrap_or(0)
    }

    pub(crate) fn sync_cursor_to_question(&mut self, question_index: usize) -> bool {
        let clamped = question_index.min(23);
        if self.current_node_index() == clamped {
            return false;
        }
        if let Some(position) = self.plan.iter().position(|index| *index == clamped) {
            self.cursor = position;
            self.checked = self.checked.min(position);
            return true;
        }
        false
    }

    fn advance(&mut self) {
        if self.cursor + 1 < self.plan.len() {
            self.cursor += 1;
        }
    }

    fn retreat(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        if self.checked > 0 {
            self.checked -= 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn checked(&self) -> usize {
        self.checked
    }

    pub(crate) fn total_prompts(&self) -> usize {
        self.plan.len()
    }

    fn weak_item_count(&self) -> usize {
        self.failed_by_question
            .iter()
            .filter(|failures| **failures >= 2)
            .count()
    }

    pub(crate) fn cache_answers_for_question(&mut self, question_index: usize, states: &[bool]) {
        if let Some(slot) = self.answer_state_cache.get_mut(question_index) {
            slot.clear();
            slot.extend(states.iter().copied().take(9));
            if slot.len() < 9 {
                slot.resize(9, false);
            }
        }
    }

    pub(crate) fn cached_answers_for_question(&self, question_index: usize) -> Vec<bool> {
        self.answer_state_cache
            .get(question_index)
            .cloned()
            .unwrap_or_else(|| vec![false; 9])
    }
}

pub(crate) fn learning_prompts_per_question_from_policy(package_dir: &Path) -> usize {
    let policy = read_learning_policy(package_dir);
    policy_number_i64(&policy, &["preLearning", "promptsPerQuestion"], 2).max(1) as usize
}

pub(crate) fn check_learning_current(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<LearningProgress>>,
) -> Result<String> {
    let node_index = normalize_node_index(core.get_learning_index());
    let mut draft = read_effective_author_draft(package_dir);
    ensure_draft_nodes(&mut draft);
    let node = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.get(node_index))
        .ok_or_else(|| anyhow::anyhow!("question {} is missing", node_index + 1))?;
    let expected = node_answer_options(node)
        .iter()
        .map(|option| {
            option
                .get("isCorrect")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let actual = learning_answer_states_from_window(core);
    if expected.len() != actual.len() {
        anyhow::bail!("question {} answer count mismatch", node_index + 1);
    }
    let matched = expected
        .iter()
        .zip(actual.iter())
        .all(|(expected, actual)| expected == actual);
    let mut passed = learning_passed.borrow_mut();
    let expected_node = passed.current_node_index();
    if node_index != expected_node {
        passed.sync_cursor_to_question(node_index);
    }
    passed.checked += 1;
    if !matched {
        passed.errors += 1;
        if let Some(failures) = passed.failed_by_question.get_mut(node_index) {
            *failures += 1;
        }
    }
    let weak_items = passed.weak_item_count();
    let checked = passed.checked;
    let total = passed.total_prompts();
    let errors = passed.errors;
    set_learning_progress_into_window(core, checked, total, errors);
    if checked >= total {
        write_learning_completed_state(package_dir)?;
        core.set_export_ready(true);
        core.set_learning_action_hint(SharedString::from(
            "Training completed. You can export now.",
        ));
        return Ok(format!(
            "Training complete: {checked}/{total} prompts checked, errors recorded={errors}, weakItems={weak_items}. Export is enabled."
        ));
    }
    core.set_export_ready(false);
    core.set_learning_action_hint(SharedString::from(
        "Mark the current 3x3 answers from memory, then use Next.",
    ));
    passed.advance();
    let next_index = passed.current_node_index();
    drop(passed);
    load_learning_node_into_window(core, package_dir, next_index as i32, Some(learning_passed));
    if matched {
        Ok(format!(
            "Prompt passed. Progress: {checked}/{total}. errors={errors}. Continue with question {}.",
            next_index + 1
        ))
    } else {
        Ok(format!(
            "Prompt recorded. Progress: {checked}/{total}. errors={errors}. Continue with question {}.",
            next_index + 1
        ))
    }
}
