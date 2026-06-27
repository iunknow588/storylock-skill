use super::*;

pub(super) fn default_author_draft_json() -> Value {
    dongguo_wolf_author_draft_json()
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
    let nodes = (1..=24)
        .map(|index| {
            let element_id = ELEMENTS[(index - 1) % ELEMENTS.len()];
            let correct = [
                format!("node {index:02} anchor one"),
                format!("node {index:02} anchor two"),
                format!("node {index:02} anchor three"),
            ];
            let wrong = [
                format!("node {index:02} distractor four"),
                format!("node {index:02} distractor five"),
                format!("node {index:02} distractor six"),
                format!("node {index:02} distractor seven"),
                format!("node {index:02} distractor eight"),
                format!("node {index:02} distractor nine"),
            ];
            let answer_options = correct
                .iter()
                .map(|text| json!({ "text": text, "isCorrect": true }))
                .chain(wrong.iter().map(|text| json!({ "text": text, "isCorrect": false })))
                .collect::<Vec<_>>();
            json!({
                "nodeId": format!("node-{index:02}"),
                "title": format!("Question {index:02}"),
                "elementId": element_id,
                "question": format!("Which three anchors belong to memory node {index:02}?"),
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

pub(super) fn dongguo_wolf_author_draft_json() -> Value {
    story_template_author_draft_json(
        "dongguo-wolf",
        "东郭先生和狼",
        "东郭先生救了被追捕的狼，狼脱险后反要吃掉恩人；最后借助旁人判断，揭示善良必须有边界。",
        "东郭先生在路上遇见一只被猎人追赶的狼。狼恳求他把自己藏进书袋，东郭先生一时心软救了它。猎人离开后，狼却露出凶相，说自己饥饿难忍，要吃掉东郭先生。双方争执不下，便请路边老人评理。老人让狼重新钻进袋子，确认事情经过后扎紧袋口，提醒东郭先生：善良需要判断对象，也需要边界。这个模板适合扩展恩情、伪装、风险判断、边界选择等 24 个记忆问题。",
        &["路上", "猎人", "书袋", "狼", "老人", "边界"],
    )
}

pub(super) fn zhizi_yilin_author_draft_json() -> Value {
    story_template_author_draft_json(
        "zhizi-yilin",
        "智子疑邻",
        "同样的提醒，因为说话人身份不同而被截然不同地理解；故事适合扩展偏见、信任和证据判断。",
        "宋国有一家人的墙被雨水冲坏了。儿子提醒父亲，墙坏了如果不修，夜里可能会有盗贼进来；邻居也说了同样的话。当天夜里果然丢了东西，主人却夸儿子聪明，怀疑邻居偷窃。这个故事把同一句话放在亲疏不同的位置上，展示人会被身份偏见影响判断。它适合扩展墙、雨夜、邻居、儿子、失窃、怀疑、证据等 24 个问题。",
        &["雨夜", "破墙", "儿子", "邻居", "失窃", "偏见"],
    )
}

pub(super) fn shouzhudaitu_author_draft_json() -> Value {
    story_template_author_draft_json(
        "shouzhudaitu",
        "守株待兔",
        "农夫偶然捡到撞树而死的兔子，随后放弃耕作，等待偶然再次发生，最终田地荒芜。",
        "宋国有个农夫正在田里劳作，一只兔子突然撞到树桩死了。农夫捡到兔子后，以为只要守着树桩就能再次得到兔子，于是放下农具，不再耕作。日子一天天过去，兔子没有再来，田地却荒芜了。这个模板可以扩展偶然、经验误判、等待、代价、结果等记忆元素。",
        &["田地", "树桩", "兔子", "农夫", "等待", "荒芜"],
    )
}
pub(super) fn default_resource_catalog_json() -> Value {
    json!({
        "version": "1",
        "resources": [{
            "resourceId": "github-main",
            "resourceKind": "website_account",
            "providerId": "github",
            "displayName": "GitHub main account",
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
        }]
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
            "templateId": "local-signature-placeholder",
            "displayName": "Local signature placeholder",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "username", "role": "username" }
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
        .chain((1..=9).map(|index| json!({ "text": format!("候选答案 {index}"), "isCorrect": false })))
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
