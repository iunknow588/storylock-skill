use super::*;

#[derive(Clone, Debug)]
pub(crate) struct PreflightIssue {
    pub(crate) code: &'static str,
    pub(crate) path: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PreflightResult {
    pub(crate) errors: Vec<PreflightIssue>,
}

pub(crate) fn preflight_storylock_core_package(package_dir: &Path) -> PreflightResult {
    let mut errors = Vec::new();
    for required_file in required_storylock_package_files() {
        if !package_dir.join(required_file).exists() {
            errors.push(PreflightIssue {
                code: "SL_PKG_REQUIRED_FILE_MISSING",
                path: "$.files".to_string(),
                message: format!("missing required file: {required_file}"),
            });
        }
    }

    let manifest = read_json_or_default(&storylock_core_manifest_path(package_dir), Value::Null);
    if let Some(files) = manifest.get("files").and_then(Value::as_array) {
        for required_file in required_storylock_package_files() {
            if !files.iter().any(|item| item.as_str() == Some(required_file)) {
                errors.push(PreflightIssue {
                    code: "SL_PKG_REQUIRED_FILE_MISSING",
                    path: "$.files".to_string(),
                    message: format!("manifest does not list required file: {required_file}"),
                });
            }
        }
    } else {
        errors.push(PreflightIssue {
            code: "SL_MANIFEST_MISSING_CATALOG_FILE",
            path: "$.files".to_string(),
            message: "manifest files must be an array".to_string(),
        });
    }

    let policy = read_learning_policy(package_dir);
    validate_learning_policy(&policy, &mut errors);

    let vault = read_storylock_vault_payload(package_dir);
    let draft = storylock_author_draft_from_vault(&vault);
    match draft.get("nodes").and_then(Value::as_array) {
        Some(nodes) if nodes.len() == 24 => {}
        Some(nodes) => errors.push(PreflightIssue {
            code: "SL_PKG_AUTHOR_DRAFT_NODE_COUNT",
            path: "$.nodes".to_string(),
            message: format!("author draft must contain exactly 24 nodes, got {}", nodes.len()),
        }),
        None => errors.push(PreflightIssue {
            code: "SL_PKG_AUTHOR_DRAFT_NODE_COUNT",
            path: "$.nodes".to_string(),
            message: "author draft nodes must be an array".to_string(),
        }),
    }

    validate_story_draft_templates(&vault, &mut errors);

    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let role_index = build_catalog_role_index(&catalog, &mut errors);
    for (file_name, bundle) in storylock_templates_from_vault(&vault)
        .as_object()
        .cloned()
        .unwrap_or_default()
    {
        validate_template_references(&file_name, &bundle, &role_index, &mut errors);
    }

    PreflightResult { errors }
}

pub(crate) fn validate_story_draft_templates(vault: &Value, errors: &mut Vec<PreflightIssue>) {
    let Some(items) = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
    else {
        errors.push(PreflightIssue {
            code: "SL_STORY_TEMPLATE_MISSING",
            path: "$.storyDraftTemplates.items".to_string(),
            message: "story draft templates must be stored as authorDraft-compatible items"
                .to_string(),
        });
        return;
    };

    for (index, item) in items.iter().enumerate() {
        for field in ["storyTitle", "summary", "storyPlot"] {
            if item.get(field).and_then(Value::as_str).unwrap_or("").is_empty() {
                errors.push(PreflightIssue {
                    code: "SL_STORY_TEMPLATE_FIELD_MISSING",
                    path: format!("$.storyDraftTemplates.items[{index}].{field}"),
                    message: format!("story draft template must include {field}"),
                });
            }
        }
        match item.get("nodes").and_then(Value::as_array) {
            Some(nodes) if nodes.len() == 24 => {}
            Some(nodes) => errors.push(PreflightIssue {
                code: "SL_STORY_TEMPLATE_NODE_COUNT",
                path: format!("$.storyDraftTemplates.items[{index}].nodes"),
                message: format!(
                    "story draft template must contain exactly 24 nodes, got {}",
                    nodes.len()
                ),
            }),
            None => errors.push(PreflightIssue {
                code: "SL_STORY_TEMPLATE_NODE_COUNT",
                path: format!("$.storyDraftTemplates.items[{index}].nodes"),
                message: "story draft template nodes must be an array".to_string(),
            }),
        }
    }
}

pub(crate) fn validate_learning_policy(policy: &Value, errors: &mut Vec<PreflightIssue>) {
    if policy.get("schemaVersion").and_then(Value::as_str) != Some("1") {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.schemaVersion".to_string(),
            message: "learning-policy.json schemaVersion must be 1".to_string(),
        });
    }
    if policy
        .get("hostReadable")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        != true
    {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.hostReadable".to_string(),
            message: "learning-policy.json must be host-readable for retention execution"
                .to_string(),
        });
    }
    validate_fixed_policy_number(
        policy,
        &["preLearning", "questionCount"],
        24,
        "$.preLearning.questionCount",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["preLearning", "promptsPerQuestion"],
        2,
        "$.preLearning.promptsPerQuestion",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["preLearning", "totalPrompts"],
        48,
        "$.preLearning.totalPrompts",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["preLearning", "minRepeatGap"],
        12,
        "$.preLearning.minRepeatGap",
        errors,
    );
    validate_range_policy_number(
        policy,
        &["preLearning", "errorTolerance"],
        1,
        9,
        "$.preLearning.errorTolerance",
        errors,
    );
    validate_range_policy_number(
        policy,
        &["preLearning", "weakItemLimit"],
        1,
        9,
        "$.preLearning.weakItemLimit",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["retentionLearning", "questionCount"],
        22,
        "$.retentionLearning.questionCount",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["retentionLearning", "questionCount"],
        22,
        "$.retentionLearning.questionCount",
        errors,
    );

    let expected_phases = [
        ("initial", "day", 3, "day", 1),
        ("consolidation", "day", 4, "day", 2),
        ("adaptation", "week", 3, "week", 1),
        ("stable", "month", 4, "month", 1),
        ("long_term", "year", 1, "year", 1),
    ];
    let Some(phases) = policy
        .get("retentionLearning")
        .and_then(|value| value.get("phases"))
        .and_then(Value::as_array)
    else {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.retentionLearning.phases".to_string(),
            message: "retentionLearning.phases must be an array".to_string(),
        });
        return;
    };
    if phases.len() != expected_phases.len() {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.retentionLearning.phases".to_string(),
            message: format!("expected {} retention phases", expected_phases.len()),
        });
    }
    for (index, (phase, duration_unit, duration_value, frequency_unit, frequency_value)) in
        expected_phases.iter().enumerate()
    {
        validate_phase_entry(
            phases.get(index),
            index,
            phase,
            duration_unit,
            *duration_value,
            frequency_unit,
            *frequency_value,
            errors,
        );
    }
}

pub(crate) fn policy_value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

pub(crate) fn validate_fixed_policy_number(
    value: &Value,
    path: &[&str],
    expected: i64,
    json_path: &str,
    errors: &mut Vec<PreflightIssue>,
) {
    if policy_value_at(value, path).and_then(Value::as_i64) != Some(expected) {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: json_path.to_string(),
            message: format!("expected {expected}"),
        });
    }
}

pub(crate) fn validate_range_policy_number(
    value: &Value,
    path: &[&str],
    min: i64,
    max: i64,
    json_path: &str,
    errors: &mut Vec<PreflightIssue>,
) {
    let valid = policy_value_at(value, path)
        .and_then(Value::as_i64)
        .is_some_and(|number| number >= min && number <= max);
    if !valid {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: json_path.to_string(),
            message: format!("expected range {min}..={max}"),
        });
    }
}

pub(crate) fn validate_phase_entry(
    phase_value: Option<&Value>,
    phase_index: usize,
    expected_phase: &str,
    expected_duration_unit: &str,
    expected_duration_value: i64,
    expected_frequency_unit: &str,
    expected_frequency_value: i64,
    errors: &mut Vec<PreflightIssue>,
) {
    let path_prefix = format!("$.retentionLearning.phases[{phase_index}]");
    let Some(phase_value) = phase_value else {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: path_prefix,
            message: "missing phase entry".to_string(),
        });
        return;
    };

    if phase_value.get("phase").and_then(Value::as_str) != Some(expected_phase) {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: format!("{path_prefix}.phase"),
            message: format!("expected phase {expected_phase}"),
        });
    }
    if phase_value
        .get("duration")
        .and_then(|value| value.get("unit"))
        .and_then(Value::as_str)
        != Some(expected_duration_unit)
    {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: format!("{path_prefix}.duration.unit"),
            message: format!("expected unit {expected_duration_unit}"),
        });
    }
    if phase_value
        .get("duration")
        .and_then(|value| value.get("value"))
        .and_then(Value::as_i64)
        != Some(expected_duration_value)
    {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: format!("{path_prefix}.duration.value"),
            message: format!("expected value {expected_duration_value}"),
        });
    }
    if phase_value
        .get("frequency")
        .and_then(|value| value.get("unit"))
        .and_then(Value::as_str)
        != Some(expected_frequency_unit)
    {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: format!("{path_prefix}.frequency.unit"),
            message: format!("expected unit {expected_frequency_unit}"),
        });
    }
    if phase_value
        .get("frequency")
        .and_then(|value| value.get("value"))
        .and_then(Value::as_i64)
        != Some(expected_frequency_value)
    {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: format!("{path_prefix}.frequency.value"),
            message: format!("expected value {expected_frequency_value}"),
        });
    }
}

pub(crate) fn build_catalog_role_index(
    catalog: &Value,
    errors: &mut Vec<PreflightIssue>,
) -> HashMap<String, HashSet<String>> {
    let mut role_index = HashMap::new();
    for resource in catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let resource_id = resource
            .get("resourceId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if resource_id.is_empty() {
            errors.push(PreflightIssue {
                code: "SL_CATALOG_RESOURCE_ID_MISSING",
                path: "$.resources[*].resourceId".to_string(),
                message: "resourceId must be present".to_string(),
            });
            continue;
        }
        let mut roles = HashSet::new();
        for binding in resource
            .get("bindings")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            let role = binding.get("role").and_then(Value::as_str).unwrap_or("");
            let object_id = binding.get("objectId").and_then(Value::as_str).unwrap_or("");
            if role.is_empty() {
                errors.push(PreflightIssue {
                    code: "SL_CATALOG_BINDING_ROLE_MISSING",
                    path: format!("$.resources[{}].bindings[*].role", resource_id),
                    message: "binding role must be present".to_string(),
                });
            }
            if !is_four_segment_object_id(object_id) {
                errors.push(PreflightIssue {
                    code: "SL_CATALOG_OBJECT_ID_INVALID",
                    path: format!("$.resources[{}].bindings[*].objectId", resource_id),
                    message: "objectId must use four segments".to_string(),
                });
            }
            if !role.is_empty() {
                roles.insert(role.to_string());
            }
        }
        role_index.insert(resource_id, roles);
    }
    role_index
}

pub(crate) fn validate_template_references(
    file_name: &str,
    bundle: &Value,
    role_index: &HashMap<String, HashSet<String>>,
    errors: &mut Vec<PreflightIssue>,
) {
    for (index, item) in bundle
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let resource_id = item.get("resourceId").and_then(Value::as_str).unwrap_or("");
        let Some(roles) = role_index.get(resource_id) else {
            errors.push(PreflightIssue {
                code: "SL_TEMPLATE_UNKNOWN_RESOURCE",
                path: format!("$.templates.{file_name}.items[{index}].resourceId"),
                message: format!("unknown resourceId: {resource_id}"),
            });
            continue;
        };
        for binding in item
            .get("bindings")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            let role = binding.get("role").and_then(Value::as_str).unwrap_or("");
            if !roles.contains(role) {
                errors.push(PreflightIssue {
                    code: "SL_TEMPLATE_UNKNOWN_ROLE",
                    path: format!("$.templates.{file_name}.items[{index}].bindings[*].role"),
                    message: format!("resource {resource_id} does not expose role {role}"),
                });
            }
        }
    }
}

pub(crate) fn is_four_segment_object_id(value: &str) -> bool {
    value.split('/').count() == 4
}
