use super::*;

pub(crate) fn host_learning_plan_status(package_dir: &Path) -> String {
    let policy_path = storylock_core_learning_policy_path(package_dir);
    if !policy_path.exists() {
        return "Learning plan: not configured. Open StoryLock Core Export and save learning-policy.json.".to_string();
    }
    let policy = read_learning_policy(package_dir);
    let summary = learning_policy_summary(&policy);
    let current_phase = policy
        .get("execution")
        .and_then(|value| value.get("currentPhase"))
        .and_then(Value::as_str)
        .unwrap_or("initial");
    let status = policy
        .get("execution")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("pending_export");
    format!("Learning plan: {status}, current phase={current_phase}\n{summary}")
}

pub(crate) fn default_learning_policy_json() -> Value {
    json!({
        "schemaVersion": "1",
        "policyId": "storylock-default-learning-policy",
        "updatedAt": ui_now_timestamp(),
        "hostReadable": true,
        "preLearning": {
            "questionCount": 24,
            "promptsPerQuestion": 2,
            "totalPrompts": 48,
            "minRepeatGap": 12,
            "errorTolerance": 2,
            "weakItemLimit": 3
        },
        "retentionLearning": {
            "description": "Prevents users from forgetting StoryLock answers by forcing periodic review after export.",
            "questionCount": 22,
            "questionCountMeaning": "Each retention review requires 22 fixed questions.",
            "frequencyDesign": "Review frequency decreases over time: daily, weekly, monthly, then yearly.",
            "phaseParameterMeaning": "Duration sets how long a phase lasts; frequency sets how often review is triggered in that phase.",
            "phases": [
                { "phase": "initial", "duration": { "unit": "day", "value": 3 }, "frequency": { "unit": "day", "value": 1 } },
                { "phase": "consolidation", "duration": { "unit": "day", "value": 4 }, "frequency": { "unit": "day", "value": 2 } },
                { "phase": "adaptation", "duration": { "unit": "week", "value": 3 }, "frequency": { "unit": "week", "value": 1 } },
                { "phase": "stable", "duration": { "unit": "month", "value": 4 }, "frequency": { "unit": "month", "value": 1 } },
                { "phase": "long_term", "duration": { "unit": "year", "value": 1 }, "frequency": { "unit": "year", "value": 1 } }
            ]
        },
        "execution": {
            "status": "pending_export",
            "currentPhase": "initial",
            "nextCheckAfter": { "unit": "day", "value": 1 },
            "lastResult": "not_started"
        }
    })
}

pub(crate) fn read_learning_policy(package_dir: &Path) -> Value {
    read_json_or_default(
        &storylock_core_learning_policy_path(package_dir),
        default_learning_policy_json(),
    )
}

pub(crate) fn write_learning_policy(package_dir: &Path, policy: &Value) -> Result<()> {
    let path = storylock_core_learning_policy_path(package_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(policy)?)?;
    ensure_manifest_lists_required_files(package_dir)
}

pub(crate) fn bounded_policy_int(value: &str, field_name: &str) -> Result<i64> {
    let parsed = value
        .trim()
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("{field_name} must be a number from 1 to 9"))?;
    if !(1..=9).contains(&parsed) {
        anyhow::bail!("{field_name} must be between 1 and 9");
    }
    Ok(parsed)
}

pub(crate) fn learning_policy_from_window(core: &StoryLockCoreApp) -> Result<Value> {
    Ok(json!({
        "schemaVersion": "1",
        "policyId": "storylock-core-learning-policy",
        "updatedAt": ui_now_timestamp(),
        "hostReadable": true,
        "preLearning": {
            "questionCount": 24,
            "promptsPerQuestion": 2,
            "totalPrompts": 48,
            "minRepeatGap": 12,
            "errorTolerance": bounded_policy_int(core.get_pre_learning_error_tolerance().as_str(), "pre-learning error tolerance")?,
            "weakItemLimit": bounded_policy_int(core.get_weak_item_limit().as_str(), "weak item limit")?
        },
        "retentionLearning": {
            "description": "Prevents users from forgetting StoryLock answers by forcing periodic review after export.",
            "questionCount": 22,
            "questionCountMeaning": "Each retention review requires 22 fixed questions.",
            "frequencyDesign": "Review frequency decreases over time: daily, weekly, monthly, then yearly.",
            "phaseParameterMeaning": "Duration sets how long a phase lasts; frequency sets how often review is triggered in that phase.",
            "phases": [
                {
                    "phase": "initial",
                    "duration": { "unit": "day", "value": bounded_policy_int(core.get_initial_days().as_str(), "initial days")? },
                    "frequency": { "unit": "day", "value": bounded_policy_int(core.get_initial_frequency_days().as_str(), "initial frequency")? }
                },
                {
                    "phase": "consolidation",
                    "duration": { "unit": "day", "value": bounded_policy_int(core.get_consolidation_days().as_str(), "consolidation days")? },
                    "frequency": { "unit": "day", "value": bounded_policy_int(core.get_consolidation_frequency_days().as_str(), "consolidation frequency")? }
                },
                {
                    "phase": "adaptation",
                    "duration": { "unit": "week", "value": bounded_policy_int(core.get_adaptation_weeks().as_str(), "adaptation weeks")? },
                    "frequency": { "unit": "week", "value": bounded_policy_int(core.get_adaptation_frequency_weeks().as_str(), "adaptation frequency")? }
                },
                {
                    "phase": "stable",
                    "duration": { "unit": "month", "value": bounded_policy_int(core.get_stable_months().as_str(), "stable months")? },
                    "frequency": { "unit": "month", "value": bounded_policy_int(core.get_stable_frequency_months().as_str(), "stable frequency")? }
                },
                {
                    "phase": "long_term",
                    "duration": { "unit": "year", "value": bounded_policy_int(core.get_long_term_years().as_str(), "long-term years")? },
                    "frequency": { "unit": "year", "value": bounded_policy_int(core.get_long_term_frequency_years().as_str(), "long-term frequency")? }
                }
            ]
        },
        "execution": {
            "status": "active_after_export",
            "currentPhase": "initial",
            "nextCheckAfter": {
                "unit": "day",
                "value": bounded_policy_int(core.get_initial_frequency_days().as_str(), "initial frequency")?
            },
            "lastResult": "not_started"
        }
    }))
}

pub(crate) fn policy_number(policy: &Value, path: &[&str], fallback: i64) -> String {
    policy_number_i64(policy, path, fallback).to_string()
}

pub(crate) fn policy_number_i64(policy: &Value, path: &[&str], fallback: i64) -> i64 {
    let mut current = policy;
    for key in path {
        current = current.get(*key).unwrap_or(&Value::Null);
    }
    current.as_i64().unwrap_or(fallback)
}

pub(crate) fn phase_number(policy: &Value, phase: &str, section: &str, fallback: i64) -> String {
    policy
        .get("retentionLearning")
        .and_then(|value| value.get("phases"))
        .and_then(Value::as_array)
        .and_then(|phases| {
            phases
                .iter()
                .find(|item| item.get("phase").and_then(Value::as_str) == Some(phase))
        })
        .and_then(|item| item.get(section))
        .and_then(|value| value.get("value"))
        .and_then(Value::as_i64)
        .unwrap_or(fallback)
        .to_string()
}

pub(crate) fn learning_policy_summary(policy: &Value) -> String {
    let pre_errors = policy_number(policy, &["preLearning", "errorTolerance"], 2);
    let weak_limit = policy_number(policy, &["preLearning", "weakItemLimit"], 3);
    let initial_frequency = phase_number(policy, "initial", "frequency", 1);
    let consolidation_frequency = phase_number(policy, "consolidation", "frequency", 2);
    let adaptation_frequency = phase_number(policy, "adaptation", "frequency", 1);
    let stable_frequency = phase_number(policy, "stable", "frequency", 1);
    let long_frequency = phase_number(policy, "long_term", "frequency", 1);
    format!(
        "Pre-learning: 48 prompts, max errors {pre_errors}, weak items <= {weak_limit}. Retention: 22 questions; initial every {initial_frequency} day(s), consolidation every {consolidation_frequency} day(s), adaptation every {adaptation_frequency} week(s), stable every {stable_frequency} month(s), long-term every {long_frequency} year(s)."
    )
}

pub(crate) fn load_learning_policy_into_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let policy = read_learning_policy(package_dir);
    core.set_pre_learning_error_tolerance(SharedString::from(policy_number(
        &policy,
        &["preLearning", "errorTolerance"],
        2,
    )));
    core.set_weak_item_limit(SharedString::from(policy_number(
        &policy,
        &["preLearning", "weakItemLimit"],
        3,
    )));
    core.set_initial_days(SharedString::from(phase_number(&policy, "initial", "duration", 3)));
    core.set_initial_frequency_days(SharedString::from(phase_number(&policy, "initial", "frequency", 1)));
    core.set_consolidation_days(SharedString::from(phase_number(&policy, "consolidation", "duration", 4)));
    core.set_consolidation_frequency_days(SharedString::from(phase_number(&policy, "consolidation", "frequency", 2)));
    core.set_adaptation_weeks(SharedString::from(phase_number(&policy, "adaptation", "duration", 3)));
    core.set_adaptation_frequency_weeks(SharedString::from(phase_number(&policy, "adaptation", "frequency", 1)));
    core.set_stable_months(SharedString::from(phase_number(&policy, "stable", "duration", 4)));
    core.set_stable_frequency_months(SharedString::from(phase_number(&policy, "stable", "frequency", 1)));
    core.set_long_term_years(SharedString::from(phase_number(&policy, "long_term", "duration", 1)));
    core.set_long_term_frequency_years(SharedString::from(phase_number(&policy, "long_term", "frequency", 1)));
    core.set_learning_plan_summary(SharedString::from(learning_policy_summary(&policy)));
}

pub(crate) fn save_learning_policy_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    let policy = learning_policy_from_window(core)?;
    write_learning_policy(package_dir, &policy)?;
    core.set_learning_plan_summary(SharedString::from(learning_policy_summary(&policy)));
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    Ok(())
}
