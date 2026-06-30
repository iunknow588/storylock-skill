use super::*;
use sha2::{Digest, Sha256};

const LEARNING_STATE_SCHEMA_VERSION: &str = "1";

pub(crate) fn storylock_core_learning_state_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("learning-state.json")
}

pub(crate) fn learning_state_fingerprint(package_dir: &Path) -> String {
    let mut hasher = Sha256::new();
    let draft = storylock_author_draft_from_vault(&read_storylock_vault_payload(package_dir));
    if let Some(nodes) = draft.get("nodes").and_then(Value::as_array) {
        for node in nodes.iter().take(24) {
            let answer_shape = json!({
                "question": node.get("question").cloned().unwrap_or(Value::Null),
                "canonicalAnswerLocalOnly": node
                    .get("canonicalAnswerLocalOnly")
                    .cloned()
                    .unwrap_or(Value::Null),
                "acceptedAnswersLocalOnly": node
                    .get("acceptedAnswersLocalOnly")
                    .cloned()
                    .unwrap_or(Value::Null),
                "answerOptionsLocalOnly": node
                    .get("answerOptionsLocalOnly")
                    .cloned()
                    .unwrap_or(Value::Null),
            });
            if let Ok(bytes) = serde_json::to_vec(&answer_shape) {
                hasher.update(b"|answerConfig:");
                hasher.update(bytes);
            }
        }
    }
    format!("{:x}", hasher.finalize())
}

pub(crate) fn write_learning_completed_state(package_dir: &Path) -> Result<()> {
    let state = json!({
        "schemaVersion": LEARNING_STATE_SCHEMA_VERSION,
        "learningCompleted": true,
        "completedAt": ui_now_timestamp(),
        "fingerprint": learning_state_fingerprint(package_dir),
    });
    fs::write(
        storylock_core_learning_state_path(package_dir),
        serde_json::to_vec_pretty(&state)?,
    )?;
    Ok(())
}

pub(crate) fn clear_learning_completed_state(package_dir: &Path) -> Result<()> {
    let path = storylock_core_learning_state_path(package_dir);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub(crate) fn stored_learning_state_fingerprint(package_dir: &Path) -> Option<String> {
    read_json_or_default(&storylock_core_learning_state_path(package_dir), Value::Null)
        .get("fingerprint")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(crate) fn clear_learning_completed_state_if_answer_config_changed(
    package_dir: &Path,
) -> Result<bool> {
    let Some(previous) = stored_learning_state_fingerprint(package_dir) else {
        return Ok(false);
    };
    if previous == learning_state_fingerprint(package_dir) {
        return Ok(false);
    }
    clear_learning_completed_state(package_dir)?;
    Ok(true)
}

pub(crate) fn has_current_learning_completed_state(package_dir: &Path) -> bool {
    let state = read_json_or_default(&storylock_core_learning_state_path(package_dir), Value::Null);
    state.get("schemaVersion").and_then(Value::as_str) == Some(LEARNING_STATE_SCHEMA_VERSION)
        && state
            .get("learningCompleted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        && state.get("fingerprint").and_then(Value::as_str)
            == Some(learning_state_fingerprint(package_dir).as_str())
}
