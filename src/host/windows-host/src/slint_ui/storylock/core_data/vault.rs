use super::*;

pub(crate) fn default_storylock_vault_json() -> Value {
    json!({
        "schemaVersion": "1",
        "authorDraft": default_author_draft_json(),
        "pendingAuthorDraft": Value::Null,
        "protectedResources": default_protected_resources_catalog_json(),
        "storyDraftTemplates": default_story_draft_templates_json(),
        "templates": default_storylock_templates_json()
    })
}

pub(crate) fn default_story_draft_templates_json() -> Value {
    json!({
        "schemaVersion": "storylock-story-draft-templates-v1",
        "defaultTemplateId": "shouzhudaitu-zh",
        "items": [
            shouzhudaitu_author_draft_json(),
            zhizi_yilin_author_draft_json(),
            emperor_new_clothes_author_draft_json()
        ]
    })
}

pub(crate) fn default_storylock_templates_json() -> Value {
    json!({
        "loginSites": default_login_templates_json(),
        "signingActions": default_signing_templates_json(),
        "agentTasks": default_agent_templates_json()
    })
}

pub(crate) fn ensure_storylock_vault_with_optional_author_draft(
    package_dir: &Path,
    author_draft: Option<Value>,
) -> Result<()> {
    if storylock_core_vault_path(package_dir).exists() {
        let mut vault = read_storylock_vault_payload(package_dir);
        let before = vault.clone();
        if let Some(author_draft) = author_draft {
            let current = storylock_author_draft_from_vault(&vault);
            let current_template_id = current.get("templateId").and_then(Value::as_str);
            let directory_template_id = author_draft.get("templateId").and_then(Value::as_str);
            if current_template_id != directory_template_id
                || story_draft_contains_mojibake(&current)
            {
                vault["authorDraft"] = author_draft.clone();
                vault["pendingAuthorDraft"] = author_draft.clone();
                vault["storyDraftTemplates"] = story_draft_templates_from_draft(&author_draft);
            }
        }
        if vault.get("storyDraftTemplates").is_none() {
            let draft = storylock_author_draft_from_vault(&vault);
            vault["storyDraftTemplates"] = story_draft_templates_from_draft(&draft);
        }
        if vault.get("protectedResources").is_none() {
            let legacy_catalog = read_json_or_default(
                &storylock_core_catalog_path(package_dir),
                default_protected_resources_catalog_json(),
            );
            vault["protectedResources"] =
                protected_resources_from_legacy_catalog_or_default(&legacy_catalog);
        }
        normalize_builtin_protected_resources_and_templates(&mut vault);
        refresh_placeholder_author_draft_nodes(&mut vault);
        merge_builtin_story_draft_templates(&mut vault);
        if vault != before {
            save_storylock_vault_payload(package_dir, vault)?;
        }
        return Ok(());
    }
    let legacy_draft = read_json_or_default(
        &package_dir.join("author-draft.json"),
        default_author_draft_json(),
    );
    let legacy_templates = json!({
        "loginSites": read_json_or_default(
            &package_dir.join("templates").join("login-sites.json"),
            default_login_templates_json(),
        ),
        "signingActions": read_json_or_default(
            &package_dir.join("templates").join("signing-actions.json"),
            default_signing_templates_json(),
        ),
        "agentTasks": read_json_or_default(
            &package_dir.join("templates").join("agent-tasks.json"),
            default_agent_templates_json(),
        )
    });
    let author_draft = author_draft.unwrap_or(legacy_draft);
    let vault = json!({
        "schemaVersion": "1",
        "authorDraft": author_draft.clone(),
        "pendingAuthorDraft": Value::Null,
        "protectedResources": protected_resources_from_legacy_catalog_or_default(&read_json_or_default(
            &package_dir.join("resource-catalog.json"),
            default_protected_resources_catalog_json(),
        )),
        "storyDraftTemplates": story_draft_templates_from_draft(&author_draft),
        "templates": legacy_templates,
    });
    write_storylock_vault(package_dir, &vault)
}

pub(crate) fn normalize_builtin_protected_resources_and_templates(vault: &mut Value) {
    let default_resources = default_protected_resources_catalog_json();
    let mut protected = protected_resources_from_vault(vault);
    let mut resources = protected
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for default_resource in default_resources
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let resource_id = default_resource
            .get("resourceId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !resource_id.is_empty()
            && !resources.iter().any(|resource| {
                resource.get("resourceId").and_then(Value::as_str) == Some(resource_id)
            })
        {
            resources.push(default_resource);
        }
    }
    protected["version"] = json!("1");
    protected["resources"] = Value::Array(resources);
    vault["protectedResources"] = protected;

    let mut templates = storylock_templates_from_vault(vault);
    remove_template_items_with_missing_resource_roles(&mut templates, &vault["protectedResources"]);
    normalize_builtin_template_bundle(
        &mut templates,
        "loginSites",
        "github.com",
        default_login_templates_json,
    );
    normalize_builtin_template_bundle(
        &mut templates,
        "signingActions",
        "evm-signing-key",
        default_signing_templates_json,
    );
    normalize_builtin_template_bundle(
        &mut templates,
        "agentTasks",
        "local-agent-placeholder",
        default_agent_templates_json,
    );
    vault["templates"] = templates;
}

fn remove_template_items_with_missing_resource_roles(templates: &mut Value, protected: &Value) {
    let role_index = protected
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|resource| {
            let resource_id = resource
                .get("resourceId")
                .and_then(Value::as_str)?
                .to_string();
            let roles = resource
                .get("bindings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|binding| {
                    binding
                        .get("role")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
                .collect::<HashSet<_>>();
            Some((resource_id, roles))
        })
        .collect::<HashMap<_, _>>();

    for spec in TEMPLATE_CHILD_SPECS {
        let fallback = template_bundle_fallback(spec.bundle_key);
        let mut bundle = templates
            .get(spec.bundle_key)
            .cloned()
            .unwrap_or_else(|| fallback.clone());
        if !bundle.is_object() {
            bundle = fallback.clone();
        }
        let mut items = bundle
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        items.retain(|item| {
            let resource_id = item
                .get("resourceId")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let Some(resource_roles) = role_index.get(resource_id) else {
                return false;
            };
            item.get("bindings")
                .and_then(Value::as_array)
                .is_some_and(|bindings| {
                    bindings.iter().all(|binding| {
                        binding
                            .get("role")
                            .and_then(Value::as_str)
                            .is_some_and(|role| resource_roles.contains(role))
                    })
                })
        });
        bundle["items"] = Value::Array(items);
        templates[spec.bundle_key] = bundle;
    }
}

fn normalize_builtin_template_bundle(
    templates: &mut Value,
    bundle_key: &str,
    template_id: &str,
    fallback_bundle: fn() -> Value,
) {
    let fallback = fallback_bundle();
    let fallback_item = fallback
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("templateId").and_then(Value::as_str) == Some(template_id))
                .cloned()
        });
    let mut bundle = templates
        .get(bundle_key)
        .cloned()
        .unwrap_or_else(|| fallback.clone());
    if !bundle.is_object() {
        bundle = fallback.clone();
    }
    bundle["version"] = fallback
        .get("version")
        .cloned()
        .unwrap_or_else(|| json!("1"));
    bundle["templateType"] = fallback
        .get("templateType")
        .cloned()
        .unwrap_or_else(|| json!(bundle_key));
    let mut items = bundle
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(fallback_item) = fallback_item {
        match items
            .iter()
            .position(|item| item.get("templateId").and_then(Value::as_str) == Some(template_id))
        {
            Some(index) => items[index] = fallback_item,
            None => items.push(fallback_item),
        }
    }
    bundle["items"] = Value::Array(items);
    templates[bundle_key] = bundle;
}

pub(crate) fn refresh_placeholder_author_draft_nodes(vault: &mut Value) {
    for key in ["authorDraft", "pendingAuthorDraft"] {
        if vault.get(key).is_some_and(Value::is_null) {
            continue;
        }
        if vault
            .get(key)
            .is_some_and(story_draft_template_needs_refresh)
        {
            let mut refreshed = default_author_draft_json();
            if let Some(existing) = vault.get(key) {
                if !story_draft_contains_mojibake(existing) {
                    for field in [
                        "templateId",
                        "storyTitle",
                        "summary",
                        "storyPlot",
                        "memoryAnchors",
                        "elementGroups",
                    ] {
                        if let Some(value) = existing.get(field) {
                            refreshed[field] = value.clone();
                        }
                    }
                }
            }
            vault[key] = refreshed;
        }
    }
}

pub(crate) fn story_draft_templates_from_draft(draft: &Value) -> Value {
    let template_id = draft
        .get("templateId")
        .and_then(Value::as_str)
        .unwrap_or("current-author-draft");
    json!({
        "schemaVersion": "storylock-story-draft-templates-v1",
        "defaultTemplateId": template_id,
        "items": [draft.clone()]
    })
}

pub(crate) fn merge_builtin_story_draft_templates(vault: &mut Value) {
    let mut templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    if !templates.is_object() {
        templates = default_story_draft_templates_json();
    }
    let mut items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for builtin in [
        shouzhudaitu_author_draft_json(),
        zhizi_yilin_author_draft_json(),
        emperor_new_clothes_author_draft_json(),
    ] {
        let template_id = builtin
            .get("templateId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match items
            .iter()
            .position(|item| item.get("templateId").and_then(Value::as_str) == Some(template_id))
        {
            Some(index) if story_draft_template_needs_refresh(&items[index]) => {
                items[index] = builtin;
            }
            Some(_) => {}
            None => items.push(builtin),
        }
    }
    templates["schemaVersion"] = json!("storylock-story-draft-templates-v1");
    templates["defaultTemplateId"] = json!("shouzhudaitu-zh");
    templates["items"] = Value::Array(items);
    vault["storyDraftTemplates"] = templates;
}

pub(crate) fn story_draft_template_needs_refresh(template: &Value) -> bool {
    story_draft_contains_mojibake(template)
        || template
            .get("nodes")
            .and_then(Value::as_array)
            .map(|nodes| {
                nodes.len() != 24
                    || nodes.iter().any(|node| {
                        let question = node
                            .get("question")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .trim();
                        question.is_empty()
                            || question.starts_with("Which three anchors belong to memory node")
                            || question.contains("记忆点")
                            || question.contains("memory point")
                    })
            })
            .unwrap_or(true)
}

pub(crate) fn story_draft_contains_mojibake(value: &Value) -> bool {
    match value {
        Value::String(text) => looks_like_mojibake(text),
        Value::Array(items) => items.iter().any(story_draft_contains_mojibake),
        Value::Object(fields) => fields.values().any(story_draft_contains_mojibake),
        _ => false,
    }
}

pub(crate) fn looks_like_mojibake(text: &str) -> bool {
    text.contains('\u{fffd}')
        || text
            .chars()
            .any(|ch| ('\u{0080}'..='\u{009f}').contains(&ch))
        || ["鍊欓", "夌瓟", "妗?", "瀹堟", "氓庐", "聢", "€"]
            .iter()
            .any(|marker| text.contains(marker))
}

pub(crate) fn read_storylock_vault(package_dir: &Path) -> Value {
    let path = storylock_core_vault_path(package_dir);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(envelope) = serde_json::from_str::<ProtectedEnvelope>(&content) {
                if let Ok(bytes) = dpapi_unprotect_from_base64(&envelope.cipher_text) {
                    if let Ok(vault) = serde_json::from_slice::<Value>(&bytes) {
                        return vault;
                    }
                }
            }
        }
    }
    default_storylock_vault_json()
}

pub(crate) fn write_storylock_vault(package_dir: &Path, vault: &Value) -> Result<()> {
    let path = storylock_core_vault_path(package_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_vec(vault)?;
    let envelope = ProtectedEnvelope {
        schema_version: "dpapi-protected-v1".to_string(),
        protected_by: "windows-dpapi".to_string(),
        created_at: ui_now_timestamp(),
        cipher_text: dpapi_protect_to_base64(&serialized)?,
    };
    fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;
    Ok(())
}

pub(crate) fn read_storylock_vault_payload(package_dir: &Path) -> Value {
    read_storylock_vault(package_dir)
}

pub(crate) fn save_storylock_vault_payload(package_dir: &Path, mut vault: Value) -> Result<()> {
    if vault.get("schemaVersion").is_none() {
        vault["schemaVersion"] = json!("1");
    }
    write_storylock_vault(package_dir, &vault)
}

pub(crate) fn storylock_author_draft_from_vault(vault: &Value) -> Value {
    vault
        .get("pendingAuthorDraft")
        .cloned()
        .filter(|value| !value.is_null())
        .or_else(|| vault.get("authorDraft").cloned())
        .unwrap_or_else(default_author_draft_json)
}

pub(crate) fn storylock_templates_from_vault(vault: &Value) -> Value {
    vault
        .get("templates")
        .and_then(Value::as_object)
        .map(|templates| Value::Object(templates.clone()))
        .unwrap_or_else(default_storylock_templates_json)
}

pub(crate) fn protected_resources_from_catalog_value(catalog: &Value) -> Value {
    json!({
        "version": catalog
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("1"),
        "resources": catalog
            .get("resources")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    })
}

pub(crate) fn protected_resources_from_legacy_catalog_or_default(catalog: &Value) -> Value {
    let migrated = protected_resources_from_catalog_value(catalog);
    if migrated
        .get("resources")
        .and_then(Value::as_array)
        .is_some_and(|resources| !resources.is_empty())
    {
        migrated
    } else {
        default_protected_resources_catalog_json()
    }
}

pub(crate) fn protected_resources_from_vault(vault: &Value) -> Value {
    vault
        .get("protectedResources")
        .cloned()
        .unwrap_or_else(default_protected_resources_catalog_json)
}

pub(crate) fn read_protected_resources(package_dir: &Path) -> Value {
    protected_resources_from_vault(&read_storylock_vault_payload(package_dir))
}

pub(crate) fn save_protected_resources(package_dir: &Path, resources: Value) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    vault["protectedResources"] = protected_resources_from_catalog_value(&resources);
    save_storylock_vault_payload(package_dir, vault)
}

pub(crate) fn read_effective_author_draft(package_dir: &Path) -> Value {
    let vault = read_storylock_vault_payload(package_dir);
    storylock_author_draft_from_vault(&vault)
}

pub(crate) fn write_pending_author_draft(package_dir: &Path, draft: &Value) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    let mut normalized = draft.clone();
    normalize_author_draft_schema(&mut normalized);
    vault["pendingAuthorDraft"] = normalized.clone();
    save_storylock_vault_payload(package_dir, vault)?;
    persist_plain_story_template_if_present(package_dir, &normalized)?;
    Ok(())
}

fn persist_plain_story_template_if_present(package_dir: &Path, draft: &Value) -> Result<()> {
    let story_template_path = package_dir.join("story-template.json");
    if story_template_path.exists() {
        fs::write(&story_template_path, serde_json::to_vec_pretty(draft)?)?;
    }
    let current_story_template_path = package_dir
        .join("story-drafts")
        .join("current-story-template.json");
    if current_story_template_path.exists() {
        fs::write(
            current_story_template_path,
            serde_json::to_vec_pretty(draft)?,
        )?;
    }
    Ok(())
}

pub(crate) fn normalize_author_draft_schema(draft: &mut Value) {
    if draft.get("version").is_none() {
        draft["version"] = json!("1");
    }
    for key in ["storyTitle", "summary", "storyPlot"] {
        if draft.get(key).and_then(Value::as_str).is_none() {
            draft[key] = json!("");
        }
    }
    if draft
        .get("memoryAnchors")
        .and_then(Value::as_array)
        .is_none()
    {
        draft["memoryAnchors"] = json!([]);
    }
    if draft
        .get("elementGroups")
        .and_then(Value::as_array)
        .is_none()
    {
        draft["elementGroups"] = json!([]);
    }
    ensure_draft_nodes(draft);
}
