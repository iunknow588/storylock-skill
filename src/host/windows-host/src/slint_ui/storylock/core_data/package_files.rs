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
    if package_dir.join("story-template.json").exists() {
        return package_dir.join("templates");
    }
    storylock_runtime_config_dir(package_dir)
}

pub(crate) fn storylock_core_story_template_directories_dir(
    package_dir: &Path,
) -> std::path::PathBuf {
    storylock_runtime_templates_dir(package_dir)
}

pub(crate) fn storylock_runtime_root_dir(package_dir: &Path) -> std::path::PathBuf {
    package_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| package_dir.to_path_buf())
}

pub(crate) fn storylock_runtime_templates_dir(package_dir: &Path) -> std::path::PathBuf {
    storylock_runtime_root_dir(package_dir).join("templates")
}

pub(crate) fn storylock_runtime_config_dir(package_dir: &Path) -> std::path::PathBuf {
    storylock_runtime_root_dir(package_dir).join("config")
}

pub(crate) fn required_storylock_package_files() -> [&'static str; 7] {
    [
        "package-manifest.json",
        "resource-catalog.json",
        "vault.stlk",
        "learning-policy.json",
        "../config/login-sites.json",
        "../config/signing-actions.json",
        "../config/agent-tasks.json",
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
    cleanup_redundant_storylock_template_files(package_dir)?;
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
    if package_dir.join("story-template.json").exists() {
        return ensure_manifest_lists_required_files(package_dir);
    }
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
    }
    fs::write(
        story_template_directories_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&default_story_template_directory_manifest_json())?,
    )?;
    fs::write(
        story_template_directories_dir.join("README.md"),
        "StoryLock story template directories. Each directory contains one story-template.json with 24 questions.\n",
    )?;
    ensure_manifest_lists_required_files(package_dir)
}

fn default_story_template_directory_manifest_json() -> Value {
    let drafts = [
        ("shouzhudaitu-zh.json", shouzhudaitu_author_draft_json()),
        ("zhizi-yilin-zh.json", zhizi_yilin_author_draft_json()),
        (
            "emperor-new-clothes-en.json",
            emperor_new_clothes_author_draft_json(),
        ),
    ];
    json!({
        "schemaVersion": "storylock-template-directory-manifest-v1",
        "description": "Standalone StoryLock story template directories for user download.",
        "items": drafts
            .into_iter()
            .map(|(file_name, draft)| {
                let template_id = draft
                    .get("templateId")
                    .and_then(Value::as_str)
                    .unwrap_or(file_name.trim_end_matches(".json"));
                json!({
                    "templateId": template_id,
                    "language": draft
                        .get("language")
                        .and_then(Value::as_str)
                        .unwrap_or("zh-CN"),
                    "storyTitle": draft
                        .get("storyTitle")
                        .and_then(Value::as_str)
                        .unwrap_or("Story template"),
                    "fileName": file_name,
                    "directoryName": template_id,
                    "templateFileName": "story-template.json",
                    "nodeCount": draft
                        .get("nodes")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0)
                })
            })
            .collect::<Vec<_>>()
    })
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

pub(crate) fn cleanup_redundant_storylock_template_files(package_dir: &Path) -> Result<()> {
    let is_standalone_story_template = package_dir.join("story-template.json").exists();
    let mut paths = vec![package_dir
        .join("templates")
        .join("story-draft-templates.json")];
    if !is_standalone_story_template {
        paths.push(package_dir.join("story-drafts"));
        paths.push(package_dir.join("story-template-directories"));
        paths.push(package_dir.join("templates"));
        paths.push(storylock_runtime_root_dir(package_dir).join("story-template-directories"));
        for file_name in [
            "login-sites.json",
            "signing-actions.json",
            "agent-tasks.json",
        ] {
            paths.push(storylock_runtime_templates_dir(package_dir).join(file_name));
        }
        paths.push(storylock_runtime_templates_dir(package_dir).join("manifest.json"));
        paths.push(storylock_runtime_templates_dir(package_dir).join("README.md"));
    }
    for path in paths {
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
