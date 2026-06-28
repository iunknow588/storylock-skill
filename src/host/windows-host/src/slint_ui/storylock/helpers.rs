use super::*;

pub(super) fn default_author_draft_json() -> Value {
    shouzhudaitu_author_draft_json()
}

fn story_draft_asset_json(raw: &str, fallback_template_id: &str, fallback_title: &str) -> Value {
    serde_json::from_str::<Value>(raw).unwrap_or_else(|_| {
        story_template_author_draft_json(
            fallback_template_id,
            fallback_title,
            "Bundled StoryLock story draft template.",
            "This fallback template is used only if the bundled JSON asset cannot be parsed.",
            &["story", "memory", "question"],
        )
    })
}

pub(super) fn story_template_author_draft_json(
    template_id: &str,
    title: &str,
    summary: &str,
    plot: &str,
    anchors: &[&str],
) -> Value {
    const ELEMENTS: [&str; 8] = [
        "time", "place", "person", "object", "event", "reaction", "choice", "result",
    ];
    const QUESTION_PATTERNS: [&str; 24] = [
        "In {title}, which memory anchor marks the starting scene?",
        "Which place or setting should be linked to {title}?",
        "Who is the central character the user must remember?",
        "Which object should be treated as the concrete memory cue?",
        "What event changes the direction of the story?",
        "What reaction should the user recall after the event?",
        "Which choice creates the story's main consequence?",
        "What final result should be remembered?",
        "Which anchor helps distinguish this story from similar stories?",
        "Which detail should be recalled before any protected action?",
        "Which character or role tests the user's judgment?",
        "Which object or scene proves the memory belongs to this story?",
        "What warning does this story preserve?",
        "Which answer best represents the safe boundary in the story?",
        "Which cue should appear when reviewing the story after a delay?",
        "Which detail should not be confused with a distractor?",
        "What cause comes before the final consequence?",
        "Which remembered detail confirms the user still knows the plot?",
        "Which anchor can be used as a quick recall key?",
        "Which choice would change the outcome if remembered incorrectly?",
        "What should the user recall about the ending?",
        "Which memory element connects the story to authorization?",
        "Which detail should be checked during retention review?",
        "Which three anchors together identify this StoryLock draft?",
    ];
    let nodes = (1..=24)
        .map(|index| {
            let element_id = ELEMENTS[(index - 1) % ELEMENTS.len()];
            let anchor_count = anchors.len().max(1);
            let anchor_a = anchors
                .get((index - 1) % anchor_count)
                .copied()
                .unwrap_or("main anchor");
            let anchor_b = anchors
                .get(index % anchor_count)
                .copied()
                .unwrap_or("support anchor");
            let anchor_c = anchors
                .get((index + 1) % anchor_count)
                .copied()
                .unwrap_or("final anchor");
            let correct = [
                anchor_a.to_string(),
                anchor_b.to_string(),
                anchor_c.to_string(),
            ];
            let wrong = [
                format!("unrelated cue {index:02}"),
                format!("wrong scene {index:02}"),
                format!("false character {index:02}"),
                format!("unused object {index:02}"),
                format!("later distractor {index:02}"),
                format!("generic memory {index:02}"),
            ];
            let question = QUESTION_PATTERNS[index - 1].replace("{title}", title);
            let answer_options = correct
                .iter()
                .map(|text| json!({ "text": text, "isCorrect": true }))
                .chain(
                    wrong
                        .iter()
                        .map(|text| json!({ "text": text, "isCorrect": false })),
                )
                .collect::<Vec<_>>();
            json!({
                "nodeId": format!("node-{index:02}"),
                "title": format!("{title} Q{index:02}"),
                "elementId": element_id,
                "question": question,
                "recommendedSelectionMode": "multi_select",
                "recommendedCorrectCount": 3,
                "candidatePoolSize": 9,
                "recallPriority": "high",
                "verifyPolicy": "caseInsensitive + trim",
                "editorNotes": "StoryLock UI local draft only.",
                "canonicalAnswerLocalOnly": correct[0],
                "acceptedAnswersLocalOnly": correct,
                "answerOptionsLocalOnly": answer_options
            })
        })
        .collect::<Vec<_>>();
    json!({
        "version": "1",
        "templateId": template_id,
        "storyTitle": title,
        "summary": summary,
        "storyPlot": plot,
        "memoryAnchors": anchors,
        "elementGroups": ["time", "place", "person", "object", "event", "reaction", "choice", "result"],
        "nodes": nodes
    })
}

pub(super) fn zhizi_yilin_author_draft_json() -> Value {
    story_draft_asset_json(
        include_str!("../../../assets/story-drafts/zhizi-yilin-zh.json"),
        "zhizi-yilin-zh",
        "Zhizi Yilin",
    )
}

pub(super) fn shouzhudaitu_author_draft_json() -> Value {
    story_draft_asset_json(
        include_str!("../../../assets/story-drafts/shouzhudaitu-zh.json"),
        "shouzhudaitu-zh",
        "Shou Zhu Dai Tu",
    )
}

pub(super) fn emperor_new_clothes_author_draft_json() -> Value {
    story_draft_asset_json(
        include_str!("../../../assets/story-drafts/emperor-new-clothes-en.json"),
        "emperor-new-clothes-en",
        "The Emperor's New Clothes",
    )
}

pub(super) fn default_resource_catalog_json() -> Value {
    json!({
        "version": "1",
        "resources": [
            {
                "resourceId": "github-main",
                "resourceKind": "website_account",
                "providerId": "github",
                "displayName": "GitHub main login",
                "resourceGroup": "secret",
                "bindings": [
                    {
                        "role": "username",
                        "objectId": "credential/github/main/username",
                        "objectMeta": { "objectKind": "username", "encoding": "text", "sensitivity": "private" }
                    },
                    {
                        "role": "password",
                        "objectId": "credential/github/main/password",
                        "objectMeta": { "objectKind": "password", "encoding": "secret", "sensitivity": "secret" }
                    }
                ]
            },
            {
                "resourceId": "wallet-main",
                "resourceKind": "key_pair",
                "providerId": "evm",
                "displayName": "EVM signing key pair",
                "resourceGroup": "secret",
                "bindings": [
                    {
                        "role": "public_key",
                        "objectId": "keypair/evm/main/public_key",
                        "objectMeta": { "objectKind": "public_key", "encoding": "text", "sensitivity": "private" }
                    },
                    {
                        "role": "private_key",
                        "objectId": "keypair/evm/main/private_key",
                        "objectMeta": { "objectKind": "private_key", "encoding": "secret", "sensitivity": "secret" }
                    }
                ]
            }
        ]
    })
}

pub(super) fn default_login_templates_json() -> Value {
    json!({
        "version": "1",
        "templateType": "login-sites",
        "items": [{
            "templateId": "github.com",
            "displayName": "GitHub main login",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "username", "role": "username" },
                { "fieldName": "password", "role": "password" }
            ]
        }]
    })
}

pub(super) fn default_signing_templates_json() -> Value {
    json!({
        "version": "1",
        "templateType": "signing-actions",
        "items": [{
            "templateId": "evm-signing-key",
            "displayName": "EVM signing key",
            "resourceId": "wallet-main",
            "bindings": [
                { "fieldName": "publicKey", "role": "public_key" },
                { "fieldName": "privateKey", "role": "private_key" }
            ]
        }]
    })
}

pub(super) fn default_agent_templates_json() -> Value {
    json!({
        "version": "1",
        "templateType": "agent-tasks",
        "items": [{
            "templateId": "local-agent-placeholder",
            "displayName": "Local agent placeholder",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "username", "role": "username" }
            ]
        }]
    })
}

pub(super) fn json_string(value: &Value, path: &[&str]) -> SharedString {
    let mut current = value;
    for key in path {
        current = current.get(*key).unwrap_or(&Value::Null);
    }
    SharedString::from(current.as_str().unwrap_or(""))
}

pub(super) fn split_list(value: &str, delimiter: &str) -> Vec<String> {
    value
        .split(delimiter)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn answer_options_from_window(core: &StoryLockCoreApp) -> Vec<Value> {
    [
        (core.get_answer_1(), core.get_answer_1_state()),
        (core.get_answer_2(), core.get_answer_2_state()),
        (core.get_answer_3(), core.get_answer_3_state()),
        (core.get_answer_4(), core.get_answer_4_state()),
        (core.get_answer_5(), core.get_answer_5_state()),
        (core.get_answer_6(), core.get_answer_6_state()),
        (core.get_answer_7(), core.get_answer_7_state()),
        (core.get_answer_8(), core.get_answer_8_state()),
        (core.get_answer_9(), core.get_answer_9_state()),
    ]
    .into_iter()
    .map(|(text, state)| {
        json!({
            "text": text.to_string(),
            "isCorrect": state.as_str().eq_ignore_ascii_case("correct")
        })
    })
    .collect()
}

pub(super) fn set_answer_options_into_window(core: &StoryLockCoreApp, options: &[Value]) {
    let get = |index: usize| -> (SharedString, SharedString) {
        let option = options.get(index).unwrap_or(&Value::Null);
        let text = option.get("text").and_then(Value::as_str).unwrap_or("");
        let state = if option
            .get("isCorrect")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            "correct"
        } else {
            "wrong"
        };
        (SharedString::from(text), SharedString::from(state))
    };
    let (text, state) = get(0);
    core.set_answer_1(text);
    core.set_answer_1_state(state);
    let (text, state) = get(1);
    core.set_answer_2(text);
    core.set_answer_2_state(state);
    let (text, state) = get(2);
    core.set_answer_3(text);
    core.set_answer_3_state(state);
    let (text, state) = get(3);
    core.set_answer_4(text);
    core.set_answer_4_state(state);
    let (text, state) = get(4);
    core.set_answer_5(text);
    core.set_answer_5_state(state);
    let (text, state) = get(5);
    core.set_answer_6(text);
    core.set_answer_6_state(state);
    let (text, state) = get(6);
    core.set_answer_7(text);
    core.set_answer_7_state(state);
    let (text, state) = get(7);
    core.set_answer_8(text);
    core.set_answer_8_state(state);
    let (text, state) = get(8);
    core.set_answer_9(text);
    core.set_answer_9_state(state);
}

pub(super) fn node_answer_options(node: &Value) -> Vec<Value> {
    if let Some(options) = node.get("answerOptionsLocalOnly").and_then(Value::as_array) {
        let mut normalized = options
            .iter()
            .take(9)
            .map(|option| {
                json!({
                    "text": option.get("text").and_then(Value::as_str).unwrap_or(""),
                    "isCorrect": option.get("isCorrect").and_then(Value::as_bool).unwrap_or(false)
                })
            })
            .collect::<Vec<_>>();
        while normalized.len() < 9 {
            let index = normalized.len() + 1;
            normalized.push(json!({ "text": format!("候选答案 {index}"), "isCorrect": false }));
        }
        return normalized;
    }
    let correct = node
        .get("correctOptionsLocalOnly")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let distractors = node
        .get("distractorsLocalOnly")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    correct
        .into_iter()
        .map(|item| json!({ "text": item.as_str().unwrap_or(""), "isCorrect": true }))
        .chain(
            distractors
                .into_iter()
                .map(|item| json!({ "text": item.as_str().unwrap_or(""), "isCorrect": false })),
        )
        .chain(
            (1..=9).map(|index| json!({ "text": format!("候选答案 {index}"), "isCorrect": false })),
        )
        .take(9)
        .collect()
}

pub(super) fn format_answer_options(options: &[Value]) -> String {
    options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let text = option.get("text").and_then(Value::as_str).unwrap_or("");
            let marker = if option
                .get("isCorrect")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                "correct"
            } else {
                "wrong"
            };
            format!("{}. {} | {}", index + 1, text, marker)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn format_correct_option_indexes(options: &[Value]) -> String {
    options
        .iter()
        .enumerate()
        .filter_map(|(index, option)| {
            option
                .get("isCorrect")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                .then(|| (index + 1).to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn sanitize_segment(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if normalized.trim_matches('_').is_empty() {
        "local".to_string()
    } else {
        normalized
    }
}

pub(super) fn format_bindings(resource: &Value) -> String {
    resource
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .map(|binding| {
                    format!(
                        "{} -> {}",
                        binding.get("role").and_then(Value::as_str).unwrap_or(""),
                        binding
                            .get("objectId")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

pub(super) fn format_object_meta(resource: &Value) -> String {
    resource
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .map(|binding| {
                    let meta = binding.get("objectMeta").unwrap_or(&Value::Null);
                    format!(
                        "{}: {} {}",
                        binding.get("role").and_then(Value::as_str).unwrap_or(""),
                        meta.get("objectKind")
                            .and_then(Value::as_str)
                            .unwrap_or("secret"),
                        meta.get("sensitivity")
                            .and_then(Value::as_str)
                            .unwrap_or("private")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

#[allow(dead_code)]
pub(super) fn format_template_bindings(template: &Value) -> String {
    template
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .map(|binding| {
                    format!(
                        "{} -> {}",
                        binding
                            .get("fieldName")
                            .and_then(Value::as_str)
                            .unwrap_or(""),
                        binding.get("role").and_then(Value::as_str).unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

#[allow(dead_code)]
pub(super) fn format_all_template_bundles(package_dir: &Path) -> String {
    let templates = storylock_templates_from_vault(&read_storylock_vault(package_dir));
    [
        ("login-sites.json", default_login_templates_json()),
        ("signing-actions.json", default_signing_templates_json()),
        ("agent-tasks.json", default_agent_templates_json()),
    ]
    .iter()
    .map(|(file_name, fallback)| {
        let key = match *file_name {
            "login-sites.json" => "loginSites",
            "signing-actions.json" => "signingActions",
            "agent-tasks.json" => "agentTasks",
            _ => "",
        };
        let bundle = templates
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.clone());
        let items = bundle
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        format!(
                            "  {} ({})\n{}",
                            item.get("templateId")
                                .and_then(Value::as_str)
                                .unwrap_or("template"),
                            item.get("resourceId")
                                .and_then(Value::as_str)
                                .unwrap_or("resource"),
                            format_template_bindings(item)
                                .lines()
                                .map(|line| format!("    {line}"))
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        format!("{file_name}\n{items}")
    })
    .collect::<Vec<_>>()
    .join("\n\n")
}
