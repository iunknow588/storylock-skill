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

pub(crate) fn required_storylock_package_files() -> [&'static str; 11] {
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
    ensure_storylock_vault(package_dir)?;
    ensure_storylock_template_files(package_dir)?;
    ensure_story_draft_template_files(package_dir)?;
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
    let drafts = [
        ("shouzhudaitu-zh.json", shouzhudaitu_author_draft_json()),
        ("zhizi-yilin-zh.json", zhizi_yilin_author_draft_json()),
        (
            "emperor-new-clothes-en.json",
            emperor_new_clothes_author_draft_json(),
        ),
    ];
    for (file_name, draft) in drafts {
        fs::write(
            story_drafts_dir.join(file_name),
            serde_json::to_vec_pretty(&draft)?,
        )?;
    }
    fs::write(
        story_drafts_dir.join("manifest.json"),
        include_bytes!("../../../../assets/story-drafts/manifest.json"),
    )?;
    ensure_manifest_lists_required_files(package_dir)
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
