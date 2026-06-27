use super::*;

pub(crate) fn storylock_core_manifest_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("package-manifest.json")
}

pub(crate) fn storylock_core_package_dir() -> std::path::PathBuf {
    if let Ok(configured) = std::env::var("STORYLOCK_CORE_DATA_DIR") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            return std::path::PathBuf::from(trimmed).join("identity-package");
        }
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            return exe_dir.join("identity-package");
        }
    }
    std::path::PathBuf::from(".").join("identity-package")
}

pub(crate) fn storylock_core_package_dir_from_window(
    core: &StoryLockCoreApp,
    fallback: &Path,
) -> std::path::PathBuf {
    let configured = core.get_core_data_dir();
    let trimmed = configured.as_str().trim();
    if trimmed.is_empty() {
        fallback.to_path_buf()
    } else {
        std::path::PathBuf::from(trimmed)
    }
}

pub(crate) fn ensure_storylock_core_package_dir_from_window(
    core: &StoryLockCoreApp,
    fallback: &Path,
) -> Result<std::path::PathBuf> {
    let package_dir = storylock_core_package_dir_from_window(core, fallback);
    ensure_storylock_core_package(&package_dir)?;
    Ok(package_dir)
}

pub(crate) fn storylock_core_catalog_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("resource-catalog.json")
}

pub(crate) fn storylock_core_vault_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("vault.stlk")
}

pub(crate) fn storylock_core_learning_policy_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("learning-policy.json")
}

pub(crate) fn storylock_core_templates_dir(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("templates")
}

pub(crate) fn storylock_core_story_drafts_dir(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("story-drafts")
}

pub(crate) fn storylock_core_story_template_directories_dir(
    package_dir: &Path,
) -> std::path::PathBuf {
    storylock_core_templates_dir(package_dir).join("story-template-directories")
}

pub(crate) fn required_storylock_package_files() -> [&'static str; 20] {
    [
        "package-manifest.json",
        "resource-catalog.json",
        "vault.stlk",
        "learning-policy.json",
        "templates/login-sites.json",
        "templates/signing-actions.json",
        "templates/agent-tasks.json",
        "story-drafts/manifest.json",
        "story-drafts/shouzhudaitu-zh.json",
        "story-drafts/zhizi-yilin-zh.json",
        "story-drafts/emperor-new-clothes-en.json",
        "templates/story-template-directories/manifest.json",
        "templates/story-template-directories/README.md",
        "templates/story-template-directories/shouzhudaitu-zh/README.md",
        "templates/story-template-directories/shouzhudaitu-zh/story-template.json",
        "templates/story-template-directories/zhizi-yilin-zh/README.md",
        "templates/story-template-directories/zhizi-yilin-zh/story-template.json",
        "templates/story-template-directories/emperor-new-clothes-en/README.md",
        "templates/story-template-directories/emperor-new-clothes-en/story-template.json",
        "templates/story-draft-templates.json",
    ]
}

pub(crate) fn cleanup_legacy_storylock_package_files(package_dir: &Path) -> Result<()> {
    for path in [
        package_dir.join("author-draft.json"),
        package_dir.join(".tmp").join("author-draft.pending.json"),
    ] {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    let tmp_dir = package_dir.join(".tmp");
    if tmp_dir.exists() && tmp_dir.is_dir() && fs::read_dir(&tmp_dir)?.next().is_none() {
        fs::remove_dir(&tmp_dir)?;
    }
    Ok(())
}

pub(crate) fn ensure_storylock_core_package(package_dir: &Path) -> Result<()> {
    fs::create_dir_all(package_dir)?;
    let directory_story_template = read_directory_story_template(package_dir);
    write_json_if_missing(
        &storylock_core_manifest_path(package_dir),
        &json!({
            "packageId": "windows-storylock-core-local",
            "version": "0.1.0",
            "createdAt": ui_now_timestamp(),
            "description": "Local Windows StoryLock Core package.",
            "files": required_storylock_package_files()
        }),
    )?;
    ensure_manifest_lists_required_files(package_dir)?;
    write_json_if_missing(
        &storylock_core_catalog_path(package_dir),
        &default_resource_catalog_json(),
    )?;
    write_json_if_missing(
        &storylock_core_learning_policy_path(package_dir),
        &default_learning_policy_json(),
    )?;
    ensure_storylock_vault_with_optional_author_draft(package_dir, directory_story_template)?;
    ensure_storylock_template_files(package_dir)?;
    ensure_story_draft_template_files(package_dir)?;
    cleanup_legacy_nested_storylock_template_package_files(package_dir)?;
    cleanup_legacy_storylock_package_files(package_dir)?;
    Ok(())
}

pub(crate) fn ensure_storylock_template_files(package_dir: &Path) -> Result<()> {
    let templates_dir = storylock_core_templates_dir(package_dir);
    fs::create_dir_all(&templates_dir)?;
    let templates = storylock_templates_from_vault(&read_storylock_vault(package_dir));
    for (file_name, key, fallback) in [
        (
            "login-sites.json",
            "loginSites",
            default_login_templates_json(),
        ),
        (
            "signing-actions.json",
            "signingActions",
            default_signing_templates_json(),
        ),
        (
            "agent-tasks.json",
            "agentTasks",
            default_agent_templates_json(),
        ),
    ] {
        let value = templates.get(key).cloned().unwrap_or(fallback);
        fs::write(
            templates_dir.join(file_name),
            serde_json::to_vec_pretty(&value)?,
        )?;
    }
    ensure_manifest_lists_required_files(package_dir)
}

pub(crate) fn ensure_story_draft_template_files(package_dir: &Path) -> Result<()> {
    let story_drafts_dir = storylock_core_story_drafts_dir(package_dir);
    fs::create_dir_all(&story_drafts_dir)?;
    let story_template_directories_dir = storylock_core_story_template_directories_dir(package_dir);
    fs::create_dir_all(&story_template_directories_dir)?;
    let drafts = [
        ("shouzhudaitu-zh.json", shouzhudaitu_author_draft_json()),
        ("zhizi-yilin-zh.json", zhizi_yilin_author_draft_json()),
        (
            "emperor-new-clothes-en.json",
            emperor_new_clothes_author_draft_json(),
        ),
    ];
    for (file_name, draft) in drafts {
        let template_id = draft
            .get("templateId")
            .and_then(Value::as_str)
            .unwrap_or(file_name.trim_end_matches(".json"));
        fs::write(
            story_drafts_dir.join(file_name),
            serde_json::to_vec_pretty(&draft)?,
        )?;
        let template_dir = story_template_directories_dir.join(template_id);
        fs::create_dir_all(&template_dir)?;
        fs::write(
            template_dir.join("story-template.json"),
            serde_json::to_vec_pretty(&draft)?,
        )?;
        fs::write(
            template_dir.join("README.md"),
            story_template_readme(&draft).as_bytes(),
        )?;
        write_story_template_package_dir(&template_dir, &draft)?;
    }
    fs::write(
        story_drafts_dir.join("manifest.json"),
        include_bytes!("../../../../assets/story-drafts/manifest.json"),
    )?;
    fs::write(
        story_template_directories_dir.join("manifest.json"),
        include_bytes!("../../../../assets/story-drafts/manifest.json"),
    )?;
    fs::write(
        story_template_directories_dir.join("README.md"),
        "StoryLock story template directories. Each directory contains one story-template.json with 24 questions.\n",
    )?;
    fs::write(
        storylock_core_templates_dir(package_dir).join("story-draft-templates.json"),
        serde_json::to_vec_pretty(&default_story_draft_templates_json())?,
    )?;
    ensure_manifest_lists_required_files(package_dir)
}

fn write_story_template_package_dir(template_dir: &Path, draft: &Value) -> Result<()> {
    fs::create_dir_all(template_dir)?;
    let package_files = [
        "story-template.json",
        "package-manifest.json",
        "resource-catalog.json",
        "learning-policy.json",
        "vault.stlk",
        "templates/login-sites.json",
        "templates/signing-actions.json",
        "templates/agent-tasks.json",
        "story-drafts/manifest.json",
        "story-drafts/current-story-template.json",
    ];
    fs::write(
        template_dir.join("package-manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "packageId": draft
                .get("templateId")
                .and_then(Value::as_str)
                .map(|template_id| format!("windows-storylock-template-{template_id}"))
                .unwrap_or_else(|| "windows-storylock-template-local".to_string()),
            "version": "0.1.0",
            "createdAt": ui_now_timestamp(),
            "description": "Standalone StoryLock story template package.",
            "files": package_files
        }))?,
    )?;
    fs::write(
        template_dir.join("resource-catalog.json"),
        serde_json::to_vec_pretty(&default_resource_catalog_json())?,
    )?;
    fs::write(
        template_dir.join("learning-policy.json"),
        serde_json::to_vec_pretty(&default_learning_policy_json())?,
    )?;
    let nested_templates_dir = template_dir.join("templates");
    fs::create_dir_all(&nested_templates_dir)?;
    fs::write(
        nested_templates_dir.join("login-sites.json"),
        serde_json::to_vec_pretty(&default_login_templates_json())?,
    )?;
    fs::write(
        nested_templates_dir.join("signing-actions.json"),
        serde_json::to_vec_pretty(&default_signing_templates_json())?,
    )?;
    fs::write(
        nested_templates_dir.join("agent-tasks.json"),
        serde_json::to_vec_pretty(&default_agent_templates_json())?,
    )?;
    let nested_story_drafts_dir = template_dir.join("story-drafts");
    fs::create_dir_all(&nested_story_drafts_dir)?;
    fs::write(
        nested_story_drafts_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "storylock-story-draft-manifest-v1",
            "defaultTemplateId": draft
                .get("templateId")
                .and_then(Value::as_str)
                .unwrap_or("current-story-template"),
            "items": [{
                "templateId": draft
                    .get("templateId")
                    .and_then(Value::as_str)
                    .unwrap_or("current-story-template"),
                "language": draft
                    .get("language")
                    .and_then(Value::as_str)
                    .unwrap_or("zh-CN"),
                "storyTitle": draft
                    .get("storyTitle")
                    .and_then(Value::as_str)
                    .unwrap_or("Story template"),
                "fileName": "current-story-template.json"
            }]
        }))?,
    )?;
    fs::write(
        nested_story_drafts_dir.join("current-story-template.json"),
        serde_json::to_vec_pretty(draft)?,
    )?;
    let vault = json!({
        "schemaVersion": "1",
        "authorDraft": draft,
        "pendingAuthorDraft": draft,
        "storyDraftTemplates": {
            "schemaVersion": "storylock-story-draft-templates-v1",
            "defaultTemplateId": draft
                .get("templateId")
                .and_then(Value::as_str)
                .unwrap_or("current-story-template"),
            "items": [draft.clone()]
        },
        "templates": default_storylock_templates_json()
    });
    write_storylock_vault(template_dir, &vault)?;
    Ok(())
}

pub(crate) fn cleanup_legacy_nested_storylock_template_package_files(
    package_dir: &Path,
) -> Result<()> {
    let templates_dir = storylock_core_templates_dir(package_dir);
    for path in [
        templates_dir.join("package-manifest.json"),
        templates_dir.join("resource-catalog.json"),
        templates_dir.join("learning-policy.json"),
        templates_dir.join("vault.stlk"),
        templates_dir.join("story-drafts"),
        templates_dir.join("templates"),
    ] {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn story_template_readme(draft: &Value) -> String {
    let title = draft
        .get("storyTitle")
        .and_then(Value::as_str)
        .unwrap_or("Story template");
    let nodes = draft
        .get("nodes")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    format!("# {title}\n\nStoryLock story template with {nodes} questions.\n")
}

pub(crate) fn read_directory_story_template(package_dir: &Path) -> Option<Value> {
    let path = package_dir.join("story-template.json");
    let mut draft = fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())?;
    normalize_author_draft_schema(&mut draft);
    Some(draft)
}

pub(crate) fn ensure_manifest_lists_required_files(package_dir: &Path) -> Result<()> {
    let path = storylock_core_manifest_path(package_dir);
    let mut manifest = read_json_or_default(&path, json!({}));
    if !manifest.is_object() {
        manifest = json!({});
    }
    if manifest.get("packageId").is_none() {
        manifest["packageId"] = json!("windows-storylock-core-local");
    }
    if manifest.get("version").is_none() {
        manifest["version"] = json!("0.1.0");
    }
    if manifest.get("createdAt").is_none() {
        manifest["createdAt"] = json!(ui_now_timestamp());
    }
    let mut files = manifest
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for required_file in required_storylock_package_files() {
        if !files
            .iter()
            .any(|item| item.as_str() == Some(required_file))
        {
            files.push(json!(required_file));
        }
    }
    manifest["files"] = Value::Array(files);
    fs::write(path, serde_json::to_vec_pretty(&manifest)?)?;
    Ok(())
}

pub(crate) fn write_json_if_missing(path: &Path, value: &Value) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
    }
    Ok(())
}

pub(crate) fn ui_now_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
        .to_string()
}

pub(crate) fn read_json_or_default(path: &Path, fallback: Value) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .unwrap_or(fallback)
}
