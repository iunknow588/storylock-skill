use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoryLockUiSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) core_data_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) export_package_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) managed_key_package_dir: Option<String>,
}

pub(crate) fn storylock_ui_settings_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("config")
        .join("config.json")
}

pub(crate) fn load_storylock_ui_settings() -> StoryLockUiSettings {
    let mut settings = read_storylock_ui_settings(&storylock_ui_settings_path()).unwrap_or_default();
    normalize_storylock_ui_settings(&mut settings);
    settings
}

pub(crate) fn read_storylock_ui_settings(path: &Path) -> Result<StoryLockUiSettings> {
    if !path.exists() {
        return Ok(StoryLockUiSettings::default());
    }
    let text = fs::read_to_string(path)?;
    let settings = serde_json::from_str::<StoryLockUiSettings>(&text)?;
    Ok(settings)
}

pub(crate) fn write_storylock_ui_settings(
    path: &Path,
    settings: &StoryLockUiSettings,
) -> Result<()> {
    let mut normalized = settings.clone();
    normalize_storylock_ui_settings(&mut normalized);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&normalized)?)?;
    Ok(())
}

pub(crate) fn save_storylock_ui_settings(settings: &StoryLockUiSettings) -> Result<()> {
    write_storylock_ui_settings(&storylock_ui_settings_path(), settings)
}

pub(crate) fn initial_storylock_core_package_dir(settings: &StoryLockUiSettings) -> PathBuf {
    settings
        .core_data_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(resolve_storylock_core_package_path)
        .unwrap_or_else(storylock_core_package_dir)
}

pub(crate) fn storylock_core_package_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let resolved = resolve_storylock_core_package_path(path);
    candidates.push(resolved.clone());
    if resolved.file_name().and_then(|value| value.to_str()) == Some("vault.stlk") {
        if let Some(parent) = resolved.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    let identity_package = resolved.join("identity-package");
    if identity_package.exists()
        || identity_package.join("vault.stlk").exists()
        || identity_package.join("package-manifest.json").exists()
    {
        candidates.push(identity_package);
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(crate) fn detect_storylock_vault_conflicts(
    package_dir: &Path,
    export_dir: Option<&Path>,
    auth_dir: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let mut vault_paths = Vec::new();
    for candidate in storylock_core_package_candidates(package_dir) {
        let vault_path = storylock_core_vault_path(&candidate);
        if vault_path.exists() {
            vault_paths.push(vault_path);
        }
    }
    if let Some(export_dir) = export_dir {
        let export_dir = resolve_storylock_core_package_path(export_dir);
        let vault_path = storylock_core_vault_path(&export_dir);
        if vault_path.exists() {
            vault_paths.push(vault_path);
        }
    }
    if let Some(auth_dir) = auth_dir {
        let auth_dir = resolve_storylock_core_package_path(auth_dir);
        let vault_path = storylock_core_vault_path(&auth_dir);
        if vault_path.exists() {
            vault_paths.push(vault_path);
        }
    }
    vault_paths.sort();
    vault_paths.dedup();
    if vault_paths.len() <= 1 {
        return Ok(vault_paths);
    }
    let mut fingerprints = Vec::new();
    for path in &vault_paths {
        let bytes = fs::read(path)?;
        let digest = Sha256::digest(&bytes);
        fingerprints.push((path.clone(), format!("{:x}", digest)));
    }
    let first = fingerprints
        .first()
        .map(|item| item.1.clone())
        .unwrap_or_default();
    if fingerprints.iter().all(|item| item.1 == first) {
        Ok(vault_paths)
    } else {
        Err(anyhow::anyhow!(
            "multiple vault.stlk files were found with different contents"
        ))
    }
}

pub(crate) fn pick_storylock_core_package_path(initial_dir: &Path) -> Option<PathBuf> {
    let mut file_dialog = rfd::FileDialog::new();
    if initial_dir.exists() {
        file_dialog = file_dialog.set_directory(initial_dir);
    }
    let selected_file = file_dialog.add_filter("StoryLock vault", &["stlk"]).pick_file();
    if selected_file.is_some() {
        return selected_file;
    }
    let mut folder_dialog = rfd::FileDialog::new();
    if initial_dir.exists() {
        folder_dialog = folder_dialog.set_directory(initial_dir);
    }
    folder_dialog.pick_folder()
}

pub(crate) fn resolve_storylock_core_package_with_conflict_prompt(
    initial_dir: &Path,
    export_dir: Option<&Path>,
    auth_dir: Option<&Path>,
) -> Result<PathBuf> {
    let package_dir = resolve_storylock_core_package_path(initial_dir);
    if detect_storylock_vault_conflicts(&package_dir, export_dir, auth_dir).is_ok() {
        return Ok(package_dir);
    }
    let prompt_dir = package_dir.parent().unwrap_or(&package_dir);
    if let Some(selected_path) = pick_storylock_core_package_path(prompt_dir) {
        let selected_dir = resolve_storylock_core_package_path(selected_path);
        detect_storylock_vault_conflicts(&selected_dir, export_dir, auth_dir)?;
        Ok(selected_dir)
    } else {
        anyhow::bail!("multiple vault.stlk files were found with different contents")
    }
}

pub(crate) fn resolve_storylock_core_package_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let candidate = if path.file_name().and_then(|value| value.to_str()) == Some("vault.stlk") {
        path.parent().map(Path::to_path_buf).unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };
    if candidate.join("story-template.json").exists()
        || candidate.join("vault.stlk").exists()
        || candidate.join("package-manifest.json").exists()
    {
        return candidate;
    }
    let identity_package = candidate.join("identity-package");
    if identity_package.exists()
        || identity_package.join("vault.stlk").exists()
        || identity_package.join("package-manifest.json").exists()
    {
        return identity_package;
    }
    candidate
}

pub(crate) fn merge_host_settings(
    current: &StoryLockUiSettings,
    language: &str,
    core_data_dir: Option<String>,
) -> StoryLockUiSettings {
    let mut settings = current.clone();
    settings.language = Some(language.to_string());
    if let Some(path) = core_data_dir {
        let resolved = resolve_storylock_core_package_path(path);
        settings.core_data_dir = non_empty_setting(resolved.display().to_string().as_str());
        settings.managed_key_package_dir = None;
        settings.export_package_dir = None;
    }
    normalize_storylock_ui_settings(&mut settings);
    settings
}

pub(crate) fn settings_from_storylock_core(core: &StoryLockCoreApp) -> StoryLockUiSettings {
    StoryLockUiSettings {
        language: non_empty_setting(core.get_language().as_str()),
        core_data_dir: non_empty_setting(core.get_core_data_dir().as_str()),
        export_package_dir: non_empty_setting(core.get_export_package_dir().as_str()),
        managed_key_package_dir: None,
    }
}

pub(crate) fn apply_storylock_ui_settings(core: &StoryLockCoreApp, settings: &StoryLockUiSettings) {
    if let Some(language) = settings
        .language
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        core.set_language(SharedString::from(language));
    }
    if let Some(path) = settings
        .export_package_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        core.set_export_package_dir(SharedString::from(path));
    }
}

pub(crate) fn normalize_storylock_ui_settings(settings: &mut StoryLockUiSettings) {
    settings.core_data_dir = settings
        .core_data_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| resolve_storylock_core_package_path(value).display().to_string());
    settings.export_package_dir = settings
        .export_package_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .and_then(|value| {
            let resolved = resolve_storylock_core_package_path(&value);
            if resolved
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "storylock-managed-key-package")
            {
                None
            } else {
                Some(resolved.display().to_string())
            }
        });
    settings.managed_key_package_dir = None;
}

pub(crate) fn package_dir_status_report(package_dir: &Path) -> String {
    let vault_path = storylock_core_vault_path(package_dir);
    let vault_mtime = fs::metadata(&vault_path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|| String::from("missing"));
    let learning_policy = if package_dir.join("learning-policy.json").exists() {
        "present"
    } else {
        "missing"
    };
    let export_status = if package_dir.join("EXPORT_STATUS.txt").exists() {
        "present"
    } else {
        "missing"
    };
    format!(
        "Package: {}\nVault mtime (unix): {}\nLearning policy: {}\nExport status: {}\nPath boundary: current package root only\nLegacy scan:\n{}",
        package_dir.display(),
        vault_mtime,
        learning_policy,
        export_status,
        package_dir_legacy_scan_report(package_dir)
    )
}

pub(crate) fn package_dir_legacy_scan_report(package_dir: &Path) -> String {
    let current_vault = storylock_core_vault_path(package_dir);
    let current_hash = vault_fingerprint(&current_vault);
    let root = package_dir.parent().unwrap_or(package_dir);
    let mut rows = Vec::new();

    for name in ["identity-package", "storylock-managed-key-package"] {
        let candidate = root.join(name);
        if candidate == package_dir {
            continue;
        }
        if candidate.exists() {
            rows.push(format!(
                "- {name}: {}",
                compare_vault_to_current(&candidate, current_hash.as_deref())
            ));
        }
    }

    let templates_root = root.join("templates");
    if let Ok(entries) = fs::read_dir(&templates_root) {
        for entry in entries.flatten() {
            let candidate = entry.path();
            if !candidate.is_dir() {
                continue;
            }
            let has_story_template = candidate.join("story-template.json").exists();
            let has_vault = candidate.join("vault.stlk").exists();
            let has_manifest = candidate.join("package-manifest.json").exists();
            if !(has_story_template || has_vault || has_manifest) {
                continue;
            }
            let label = candidate
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            rows.push(format!(
                "- templates/{label}: {}",
                if has_vault {
                    compare_vault_to_current(&candidate, current_hash.as_deref())
                } else if has_story_template {
                    "template-only".to_string()
                } else {
                    "package-like without vault".to_string()
                }
            ));
        }
    }

    if rows.is_empty() {
        "none".to_string()
    } else {
        rows.join("\n")
    }
}

fn compare_vault_to_current(candidate: &Path, current_hash: Option<&str>) -> String {
    match vault_fingerprint(&storylock_core_vault_path(candidate)) {
        Some(candidate_hash) => match current_hash {
            Some(current_hash) if current_hash == candidate_hash => "vault same".to_string(),
            Some(_) => "vault different".to_string(),
            None => "vault present".to_string(),
        },
        None => "vault missing".to_string(),
    }
}

fn vault_fingerprint(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(bytes)))
}

fn non_empty_setting(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
