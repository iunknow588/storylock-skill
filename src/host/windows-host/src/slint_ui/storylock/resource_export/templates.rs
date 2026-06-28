use super::*;

pub(crate) fn save_template_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    save_story_draft_template_from_window(core, package_dir)
}

pub(crate) fn save_story_draft_template_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    let mut draft = read_effective_author_draft(package_dir);
    draft["version"] = json!("1");
    let window_template_id = core.get_template_id().to_string();
    let template_id = if window_template_id.trim().is_empty() || window_template_id == "github.com"
    {
        draft
            .get("templateId")
            .and_then(Value::as_str)
            .unwrap_or("current-author-draft")
            .to_string()
    } else {
        window_template_id
    };
    draft["templateId"] = json!(template_id);
    draft["storyTitle"] = json!(core.get_story_title().to_string());
    draft["summary"] = json!(core.get_story_summary().to_string());
    draft["storyPlot"] = json!(core.get_story_plot().to_string());
    draft["memoryAnchors"] = json!(split_list(core.get_memory_anchors().as_str(), "/"));
    draft["elementGroups"] = json!(split_list(core.get_element_group().as_str(), ","));
    write_current_node_to_draft(core, &mut draft);
    normalize_author_draft_schema(&mut draft);

    let mut vault = read_storylock_vault_payload(package_dir);
    let mut templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    if !templates.is_object() {
        templates = default_story_draft_templates_json();
    }
    let draft_template_id = draft
        .get("templateId")
        .and_then(Value::as_str)
        .unwrap_or("current-author-draft")
        .to_string();
    let mut items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    items.retain(|item| {
        item.get("templateId").and_then(Value::as_str) != Some(draft_template_id.as_str())
    });
    items.insert(0, draft.clone());
    templates["schemaVersion"] = json!("storylock-story-draft-templates-v1");
    templates["defaultTemplateId"] = json!(draft_template_id);
    templates["items"] = Value::Array(items);
    vault["storyDraftTemplates"] = templates;
    vault["pendingAuthorDraft"] = draft;
    save_storylock_vault_payload(package_dir, vault)?;
    core.set_template_display_name(json_string(
        &read_effective_author_draft(package_dir),
        &["storyTitle"],
    ));
    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(
        package_dir,
    )));
    Ok(())
}

pub(crate) fn apply_story_draft_template_to_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    merge_builtin_story_draft_templates(&mut vault);
    let requested_template_id = core.get_template_id().to_string();
    let mut draft =
        story_template_from_runtime_directory(package_dir, requested_template_id.trim())
            .or_else(|| {
                vault
                    .get("storyDraftTemplates")
                    .and_then(|templates| templates.get("items"))
                    .and_then(Value::as_array)
                    .and_then(|items| {
                        items
                            .iter()
                            .find(|item| {
                                !requested_template_id.trim().is_empty()
                                    && item.get("templateId").and_then(Value::as_str)
                                        == Some(requested_template_id.as_str())
                            })
                            .or_else(|| items.first())
                    })
                    .cloned()
            })
            .unwrap_or_else(default_author_draft_json);
    normalize_author_draft_schema(&mut draft);
    vault["pendingAuthorDraft"] = draft;
    save_storylock_vault_payload(package_dir, vault)?;
    initialize_storylock_core_window(core, package_dir);
    Ok(())
}

fn story_template_from_runtime_directory(
    package_dir: &Path,
    requested_template_id: &str,
) -> Option<Value> {
    if requested_template_id.is_empty() {
        return None;
    }
    list_runtime_story_templates(package_dir)
        .into_iter()
        .find(|draft| {
            draft.get("templateId").and_then(Value::as_str) == Some(requested_template_id)
        })
}

fn list_runtime_story_templates(package_dir: &Path) -> Vec<Value> {
    let templates_dir = storylock_runtime_templates_dir(package_dir);
    let Ok(entries) = fs::read_dir(templates_dir) else {
        return Vec::new();
    };
    let mut drafts = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path().join("story-template.json");
            if !path.exists() {
                return None;
            }
            let mut draft = read_json_or_default(&path, Value::Null);
            if !draft.is_object() {
                return None;
            }
            normalize_author_draft_schema(&mut draft);
            Some(draft)
        })
        .collect::<Vec<_>>();
    drafts.sort_by(|left, right| {
        left.get("templateId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(
                right
                    .get("templateId")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            )
    });
    drafts
}

pub(crate) fn pull_story_template_candidates_into_vault(
    _core: &StoryLockCoreApp,
    package_dir: &Path,
    host_port: u16,
) -> Result<String> {
    let url = format!("http://127.0.0.1:{host_port}/story-template/candidates?limit=10");
    let response = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(url)
        .send()?;
    if !response.status().is_success() {
        anyhow::bail!("Host returned HTTP {}", response.status());
    }
    let payload: Value = response.json()?;
    let candidates = payload
        .get("result")
        .and_then(|result| result.get("candidates"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if candidates.is_empty() {
        return Ok("No queued story template candidates.".to_string());
    }

    let mut vault = read_storylock_vault_payload(package_dir);
    merge_builtin_story_draft_templates(&mut vault);
    let mut templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    let mut items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut imported = 0usize;
    for candidate in candidates {
        let draft = story_draft_from_candidate(&candidate);
        let template_id = draft
            .get("templateId")
            .and_then(Value::as_str)
            .unwrap_or("host-candidate")
            .to_string();
        if items.iter().any(|item| {
            item.get("templateId").and_then(Value::as_str) == Some(template_id.as_str())
        }) {
            continue;
        }
        items.push(draft);
        imported += 1;
    }
    templates["schemaVersion"] = json!("storylock-story-draft-templates-v1");
    templates["items"] = Value::Array(items);
    vault["storyDraftTemplates"] = templates;
    save_storylock_vault_payload(package_dir, vault)?;
    Ok(format!("Pulled {imported} new candidate template(s)."))
}

pub(crate) fn story_draft_from_candidate(candidate: &Value) -> Value {
    let framework = candidate.get("framework").unwrap_or(candidate);
    let candidate_id = candidate
        .get("candidateId")
        .and_then(Value::as_str)
        .unwrap_or("host-candidate");
    let title = framework
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Host candidate story");
    let summary = framework
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("A Host-generated candidate framework waiting for manual StoryLock editing.");
    let plot = framework
        .get("storyPlot")
        .and_then(Value::as_str)
        .unwrap_or("This candidate was queued by Host. StoryLock should manually edit it into a private 24-question story before export.");
    let anchors = framework
        .get("memoryAnchors")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .take(8)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec!["host candidate", "private clue", "manual edit"]);
    let mut draft = story_template_author_draft_json(candidate_id, title, summary, plot, &anchors);
    draft["source"] = json!({
        "kind": "host-story-template-candidate",
        "candidateId": candidate_id,
        "hostInvokesStoryLock": false
    });
    draft
}

pub(crate) fn format_story_draft_template_summary(package_dir: &Path) -> String {
    let runtime_templates = list_runtime_story_templates(package_dir);
    if !runtime_templates.is_empty() {
        return format_story_template_items(&runtime_templates);
    }
    let vault = read_storylock_vault_payload(package_dir);
    let templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    let items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        return "No story draft template is stored.".to_string();
    }
    format_story_template_items(&items)
}

fn format_story_template_items(items: &[Value]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(index, draft)| {
            let node_count = draft
                .get("nodes")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            format!(
                "{}. templateId={}\nstoryTitle={}\nsummary={}\nnodes={}\nformat=authorDraft\n",
                index + 1,
                draft
                    .get("templateId")
                    .and_then(Value::as_str)
                    .unwrap_or("current-author-draft"),
                draft
                    .get("storyTitle")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                draft.get("summary").and_then(Value::as_str).unwrap_or(""),
                node_count
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
