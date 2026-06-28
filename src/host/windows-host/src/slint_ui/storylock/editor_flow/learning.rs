use super::*;

pub(crate) fn load_learning_node_into_window(
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
    let options = node_answer_options(&node);
    core.set_learning_index(node_index as i32);
    core.set_learning_position(SharedString::from(format!("{} / 24", node_index + 1)));
    core.set_learning_question(json_string(&node, &["question"]));
    set_learning_answers_into_window(core, &options);
    core.set_learning_result(SharedString::from(format!(
        "Question {} loaded. Mark each visible answer as correct or wrong from memory, then check current.",
        node_index + 1
    )));
}

pub(crate) fn set_learning_answers_into_window(core: &StoryLockCoreApp, options: &[Value]) {
    let answer_text = |index: usize| -> SharedString {
        SharedString::from(
            options
                .get(index)
                .and_then(|option| option.get("text"))
                .and_then(Value::as_str)
                .unwrap_or(""),
        )
    };
    core.set_learning_answer_1(answer_text(0));
    core.set_learning_answer_1_state(SharedString::from("wrong"));
    core.set_learning_answer_2(answer_text(1));
    core.set_learning_answer_2_state(SharedString::from("wrong"));
    core.set_learning_answer_3(answer_text(2));
    core.set_learning_answer_3_state(SharedString::from("wrong"));
    core.set_learning_answer_4(answer_text(3));
    core.set_learning_answer_4_state(SharedString::from("wrong"));
    core.set_learning_answer_5(answer_text(4));
    core.set_learning_answer_5_state(SharedString::from("wrong"));
    core.set_learning_answer_6(answer_text(5));
    core.set_learning_answer_6_state(SharedString::from("wrong"));
    core.set_learning_answer_7(answer_text(6));
    core.set_learning_answer_7_state(SharedString::from("wrong"));
    core.set_learning_answer_8(answer_text(7));
    core.set_learning_answer_8_state(SharedString::from("wrong"));
    core.set_learning_answer_9(answer_text(8));
    core.set_learning_answer_9_state(SharedString::from("wrong"));
}

pub(crate) fn set_learning_answer_states_into_window(core: &StoryLockCoreApp, states: &[bool]) {
    let state_text = |index: usize| {
        SharedString::from(if states.get(index).copied().unwrap_or(false) {
            "correct"
        } else {
            "wrong"
        })
    };
    core.set_learning_answer_1_state(state_text(0));
    core.set_learning_answer_2_state(state_text(1));
    core.set_learning_answer_3_state(state_text(2));
    core.set_learning_answer_4_state(state_text(3));
    core.set_learning_answer_5_state(state_text(4));
    core.set_learning_answer_6_state(state_text(5));
    core.set_learning_answer_7_state(state_text(6));
    core.set_learning_answer_8_state(state_text(7));
    core.set_learning_answer_9_state(state_text(8));
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

pub(crate) fn set_learning_progress_into_window(
    core: &StoryLockCoreApp,
    checked: usize,
    total: usize,
    errors: usize,
) {
    core.set_learning_progress_summary(SharedString::from(format!(
        "{checked} / {total} prompts completed, errors recorded: {errors}"
    )));
}

pub(crate) fn reveal_learning_answer_for_current(
    core: &StoryLockCoreApp,
    package_dir: &Path,
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
    set_learning_answer_states_into_window(core, &expected);
    Ok(format!(
        "Correct answers for question {} are now shown. Memorize them, then restart training when ready.",
        node_index + 1
    ))
}

#[derive(Clone)]
pub(crate) struct LearningProgress {
    pub(crate) plan: Vec<usize>,
    cursor: usize,
    checked: usize,
    errors: usize,
    failed_by_question: Vec<usize>,
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
        }
    }

    pub(crate) fn current_node_index(&self) -> usize {
        self.plan.get(self.cursor).copied().unwrap_or(0)
    }

    fn advance(&mut self) {
        if self.cursor + 1 < self.plan.len() {
            self.cursor += 1;
        }
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

    fn first_failed_question(&self) -> Option<usize> {
        self.failed_by_question
            .iter()
            .position(|failures| *failures > 0)
    }
}

pub(crate) fn learning_prompts_per_question_from_policy(package_dir: &Path) -> usize {
    let policy = read_learning_policy(package_dir);
    policy_number_i64(&policy, &["preLearning", "promptsPerQuestion"], 2)
        .max(1) as usize
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
        load_learning_node_into_window(core, package_dir, expected_node as i32);
        anyhow::bail!(
            "learning prompt order mismatch: expected question {}, loaded question {}",
            expected_node + 1,
            node_index + 1
        );
    }
    passed.checked += 1;
    if !matched {
        passed.errors += 1;
        if let Some(failures) = passed.failed_by_question.get_mut(node_index) {
            *failures += 1;
        }
    }
    let policy = read_learning_policy(package_dir);
    let error_tolerance =
        policy_number_i64(&policy, &["preLearning", "errorTolerance"], 2) as usize;
    let weak_item_limit = policy_number_i64(&policy, &["preLearning", "weakItemLimit"], 3) as usize;
    let weak_items = passed.weak_item_count();
    let checked = passed.checked;
    let total = passed.total_prompts();
    let errors = passed.errors;
    set_learning_progress_into_window(core, checked, total, errors);
    if checked >= total {
        let preflight = preflight_storylock_core_package(package_dir);
        if !preflight.errors.is_empty() {
            core.set_export_ready(false);
            anyhow::bail!(
                "all learning questions passed, but package preflight failed: {}",
                preflight
                    .errors
                    .iter()
                    .map(|issue| issue.code)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if passed.errors > error_tolerance || weak_items > weak_item_limit {
            let failed_question = passed.first_failed_question().unwrap_or(0);
            drop(passed);
            load_learning_node_into_window(core, package_dir, failed_question as i32);
            let mut draft = read_effective_author_draft(package_dir);
            ensure_draft_nodes(&mut draft);
            if let Some(node) = draft
                .get("nodes")
                .and_then(Value::as_array)
                .and_then(|nodes| nodes.get(failed_question))
            {
                let expected = node_answer_options(node)
                    .iter()
                    .map(|option| {
                        option
                            .get("isCorrect")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    })
                    .collect::<Vec<_>>();
                set_learning_answer_states_into_window(core, &expected);
            }
            core.set_export_ready(false);
            core.set_learning_action_hint(SharedString::from(format!(
                "Question {} is now showing the correct answers. Review it, then use Restart Training to begin the full test again.",
                failed_question + 1
            )));
            return Ok(format!(
                "Training finished but did not pass. Completed {}/{} prompts, errors={}/{}, weakItems={}/{}. Review question {} with the correct answers now shown, then train again.",
                checked,
                total,
                errors,
                error_tolerance,
                weak_items,
                weak_item_limit,
                failed_question + 1
            ));
        }
        core.set_export_ready(true);
        core.set_learning_action_hint(SharedString::from(
            "Training passed. You can export now.",
        ));
        return Ok(format!(
            "Training complete: {}/{} prompts checked, errors={}, weakItems={}. Export is enabled.",
            checked,
            total,
            errors,
            weak_items
        ));
    }
    core.set_export_ready(false);
    core.set_learning_action_hint(SharedString::from(
        "Mark the current 3x3 answers from memory, then check this prompt.",
    ));
    passed.advance();
    let next_index = passed.current_node_index();
    drop(passed);
    load_learning_node_into_window(core, package_dir, next_index as i32);
    if matched {
        Ok(format!(
            "Prompt passed. Training progress: {} / {}. errors={}. Continue with question {}.",
            checked,
            total,
            errors,
            next_index + 1
        ))
    } else {
        Ok(format!(
            "Prompt recorded. Training progress: {} / {}. errors={}. Continue with question {}.",
            checked,
            total,
            errors,
            next_index + 1
        ))
    }
}
