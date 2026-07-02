use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryLockChallengeCell {
    pub cell_id: String,
    pub prompt_text: String,
    pub answer_options: Vec<String>,
}

pub fn create_open_challenge_from_draft(
    draft: &Value,
    required_cells: usize,
) -> Result<Vec<StoryLockChallengeCell>> {
    let nodes = draft
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("StoryLock draft has no question nodes"))?;
    if nodes.len() < required_cells {
        return Err(anyhow!(
            "StoryLock draft has fewer than {required_cells} questions"
        ));
    }

    let cells = nodes
        .iter()
        .take(required_cells)
        .enumerate()
        .map(|(index, node)| {
            let options = node
                .get("answerOptionsLocalOnly")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut answer_options = options
                .iter()
                .take(9)
                .map(|option| {
                    option
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string()
                })
                .collect::<Vec<_>>();
            while answer_options.len() < 9 {
                answer_options.push(String::new());
            }
            StoryLockChallengeCell {
                cell_id: node
                    .get("nodeId")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("node-{:02}", index + 1)),
                prompt_text: node
                    .get("question")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                answer_options,
            }
        })
        .collect::<Vec<_>>();

    if cells.iter().any(|cell| {
        cell.prompt_text.trim().is_empty()
            || cell
                .answer_options
                .iter()
                .all(|answer| answer.trim().is_empty())
    }) {
        return Err(anyhow!(
            "StoryLock challenge contains incomplete question or option data"
        ));
    }

    Ok(cells)
}

pub fn toggle_selection(
    cells: &[StoryLockChallengeCell],
    selections: &mut [Vec<String>],
    current_index: usize,
    answer_index: usize,
) {
    let selected = cells
        .get(current_index)
        .and_then(|cell| cell.answer_options.get(answer_index))
        .cloned()
        .unwrap_or_default();
    if selected.trim().is_empty() {
        return;
    }
    if let Some(slot) = selections.get_mut(current_index) {
        let normalized = normalize_answer(&selected);
        if let Some(existing_index) = slot
            .iter()
            .position(|answer| normalize_answer(answer) == normalized)
        {
            slot.remove(existing_index);
        } else {
            slot.push(selected);
        }
    }
}

pub fn normalize_answer(answer: &str) -> String {
    answer.trim().to_lowercase()
}
