use super::*;

pub(crate) fn build_export_preview(package_dir: &Path) -> String {
    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let permission_objects = catalog
        .get("resources")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let preflight = preflight_storylock_core_package(package_dir);
    let status = if preflight.errors.is_empty() {
        "OK"
    } else {
        "FAILED"
    };
    let pending_state = if read_storylock_vault(package_dir)
        .get("pendingAuthorDraft")
        .is_some_and(|value| !value.is_null())
    {
        "pending temporary draft exists; export will promote it inside vault.stlk"
    } else {
        "no pending temporary draft"
    };
    let errors = if preflight.errors.is_empty() {
        "none".to_string()
    } else {
        preflight
            .errors
            .iter()
            .map(|issue| format!("{} {} {}", issue.code, issue.path, issue.message))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "identity-package/\n  vault.stlk\n  package-manifest.json\n  resource-catalog.json\n  learning-policy.json\n\nLocal path: {}\ntemporaryDraft={pending_state}\nresources={resources}\npermissionObjects={permission_objects}\npreflight={status}\nerrors:\n{errors}\n\nStoryLock UI internal export preview only; Yian Host reads learning-policy.json for retention scheduling, but does not read drafts, vault files, raw story, answers, passwords, private keys, or signingKeyBytes.",
        package_dir.display()
    )
}

pub(crate) fn validate_learning_test_inputs(package_dir: &Path) -> Result<String> {
    let draft = read_effective_author_draft(package_dir);
    let nodes = draft
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("author draft nodes must be an array"))?;
    if nodes.len() != 24 {
        anyhow::bail!(
            "author draft must contain exactly 24 questions, got {}",
            nodes.len()
        );
    }
    let mut total_correct = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        let question = node.get("question").and_then(Value::as_str).unwrap_or("");
        if question.trim().is_empty() {
            anyhow::bail!("question {} is empty", index + 1);
        }
        let options = node_answer_options(node);
        if options.len() != 9 {
            anyhow::bail!("question {} must contain 9 answer options", index + 1);
        }
        let correct_count = options
            .iter()
            .filter(|option| {
                option
                    .get("isCorrect")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .count();
        if correct_count == 0 {
            anyhow::bail!(
                "question {} must contain at least one correct answer",
                index + 1
            );
        }
        total_correct += correct_count;
    }
    let preflight = preflight_storylock_core_package(package_dir);
    if !preflight.errors.is_empty() {
        anyhow::bail!(
            "package preflight failed: {}",
            preflight
                .errors
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(format!(
        "Pre-export test passed. StoryLock questions and related package data are ready for encrypted export; verified {total_correct} correct answer markers."
    ))
}

pub(crate) fn default_storylock_export_dir(package_dir: &Path) -> std::path::PathBuf {
    package_dir
        .parent()
        .map(|parent| parent.join("storylock-managed-key-package"))
        .unwrap_or_else(|| std::path::PathBuf::from("storylock-managed-key-package"))
}

pub(crate) fn storylock_export_dir_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> std::path::PathBuf {
    let configured = core.get_export_package_dir().trim().to_string();
    if configured.is_empty() {
        default_storylock_export_dir(package_dir)
    } else {
        std::path::PathBuf::from(configured)
    }
}

#[cfg(test)]
pub(crate) fn export_storylock_package(package_dir: &Path) -> Result<std::path::PathBuf> {
    let export_dir = default_storylock_export_dir(package_dir);
    export_storylock_package_to(package_dir, &export_dir)
}

pub(crate) fn export_storylock_package_to(
    package_dir: &Path,
    export_dir: &Path,
) -> Result<std::path::PathBuf> {
    let preflight = preflight_storylock_core_package(package_dir);
    if !preflight.errors.is_empty() {
        anyhow::bail!(
            "package preflight failed: {}",
            preflight
                .errors
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    promote_pending_author_draft(package_dir)?;
    if export_dir.exists() {
        fs::remove_dir_all(export_dir)?;
    }
    copy_dir_recursive(package_dir, export_dir)?;
    fs::write(
        export_dir.join("EXPORT_STATUS.txt"),
        format!(
            "Exported from StoryLock Core after learning test.\nSource: {}\nExportedAt: {}\nTemporaryDraftCleared: true\n",
            package_dir.display(),
            ui_now_timestamp()
        ),
    )?;
    remove_pending_author_draft(package_dir)?;
    Ok(export_dir.to_path_buf())
}

pub(crate) fn promote_pending_author_draft(package_dir: &Path) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    if let Some(pending) = vault.get("pendingAuthorDraft").cloned() {
        if !pending.is_null() {
            vault["authorDraft"] = pending;
            vault["pendingAuthorDraft"] = Value::Null;
            save_storylock_vault_payload(package_dir, vault)?;
        }
    }
    Ok(())
}

pub(crate) fn remove_pending_author_draft(package_dir: &Path) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    if vault
        .get("pendingAuthorDraft")
        .is_some_and(|pending| !pending.is_null())
    {
        vault["pendingAuthorDraft"] = Value::Null;
        save_storylock_vault_payload(package_dir, vault)?;
    }
    Ok(())
}

pub(crate) fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        if source_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == ".tmp")
        {
            continue;
        }
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}
