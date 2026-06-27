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
        let mut plan = Vec::with_capacity(48);
        for _round in 0..2 {
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
    let matched = expected.iter().zip(actual.iter()).all(|(expected, actual)| expected == actual);
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
    if passed.errors > error_tolerance {
        core.set_export_ready(false);
        return Ok(format!(
            "Pre-learning failed: errors {}/{} exceeded tolerance. Restart learning after fixing weak questions.",
            passed.errors, error_tolerance
        ));
    }
    if weak_items > weak_item_limit {
        core.set_export_ready(false);
        return Ok(format!(
            "Pre-learning failed: weak items {}/{} exceeded limit. Restart learning after review.",
            weak_items, weak_item_limit
        ));
    }
    if passed.checked >= passed.total_prompts() {
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
        core.set_export_ready(true);
        return Ok(format!(
            "Pre-learning complete: {}/{} prompts checked, errors={}, weakItems={}. Export is enabled.",
            passed.checked,
            passed.total_prompts(),
            passed.errors,
            weak_items
        ));
    }
    core.set_export_ready(false);
    passed.advance();
    let next_index = passed.current_node_index();
    let checked = passed.checked;
    let total = passed.total_prompts();
    let errors = passed.errors;
    drop(passed);
    load_learning_node_into_window(core, package_dir, next_index as i32);
    if matched {
        Ok(format!(
            "Prompt passed. Pre-learning progress: {} / {}. errors={}. Continue with question {}.",
            checked,
            total,
            errors,
            next_index + 1
        ))
    } else {
        Ok(format!(
            "Prompt did not match. Pre-learning progress: {} / {}. errors={}. Continue with question {}.",
            checked,
            total,
            errors,
            next_index + 1
        ))
    }
}
