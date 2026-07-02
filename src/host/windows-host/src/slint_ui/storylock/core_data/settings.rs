use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static STORYLOCK_PATH_DIALOG_OPEN: AtomicBool = AtomicBool::new(false);
static STORYLOCK_PATH_DIALOG_LAST_CLOSED: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
const STORYLOCK_PATH_DIALOG_SUPPRESS_AFTER_CLOSE: Duration = Duration::from_millis(200);

struct StoryLockPathDialogGuard;

impl Drop for StoryLockPathDialogGuard {
    fn drop(&mut self) {
        STORYLOCK_PATH_DIALOG_OPEN.store(false, Ordering::Release);
        if let Ok(mut closed_at) = storylock_path_dialog_last_closed().lock() {
            *closed_at = Some(Instant::now());
        }
    }
}

fn storylock_path_dialog_last_closed() -> &'static Mutex<Option<Instant>> {
    STORYLOCK_PATH_DIALOG_LAST_CLOSED.get_or_init(|| Mutex::new(None))
}

fn begin_storylock_path_dialog_once() -> Option<StoryLockPathDialogGuard> {
    if let Ok(closed_at) = storylock_path_dialog_last_closed().lock() {
        if closed_at
            .as_ref()
            .is_some_and(|instant| instant.elapsed() < STORYLOCK_PATH_DIALOG_SUPPRESS_AFTER_CLOSE)
        {
            return None;
        }
    }

    if STORYLOCK_PATH_DIALOG_OPEN
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return None;
    }

    Some(StoryLockPathDialogGuard)
}

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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HostUiSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) language: Option<String>,
}

fn exe_config_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("config")
}

pub(crate) fn host_ui_settings_path() -> PathBuf {
    exe_config_dir().join("host-config.json")
}

pub(crate) fn legacy_combined_ui_settings_path() -> PathBuf {
    exe_config_dir().join("config.json")
}

pub(crate) fn storylock_ui_settings_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("storylock")
        .join("config")
        .join("storylock-config.json")
}

pub(crate) fn load_host_ui_settings() -> HostUiSettings {
    let path = host_ui_settings_path();
    let mut settings = if path.exists() {
        read_host_ui_settings(&path).unwrap_or_default()
    } else {
        read_storylock_ui_settings(&legacy_combined_ui_settings_path())
            .map(|legacy| HostUiSettings {
                language: legacy.language,
            })
            .unwrap_or_default()
    };
    normalize_host_ui_settings(&mut settings);
    settings
}

pub(crate) fn read_host_ui_settings(path: &Path) -> Result<HostUiSettings> {
    if !path.exists() {
        return Ok(HostUiSettings::default());
    }
    let text = fs::read_to_string(path)?;
    let settings = serde_json::from_str::<HostUiSettings>(&text)?;
    Ok(settings)
}

pub(crate) fn write_host_ui_settings(path: &Path, settings: &HostUiSettings) -> Result<()> {
    let mut normalized = settings.clone();
    normalize_host_ui_settings(&mut normalized);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&normalized)?)?;
    Ok(())
}

pub(crate) fn save_host_ui_settings(settings: &HostUiSettings) -> Result<()> {
    write_host_ui_settings(&host_ui_settings_path(), settings)
}

pub(crate) fn merge_host_settings(current: &HostUiSettings, language: &str) -> HostUiSettings {
    let mut settings = current.clone();
    settings.language = non_empty_setting(language);
    normalize_host_ui_settings(&mut settings);
    settings
}

fn normalize_host_ui_settings(settings: &mut HostUiSettings) {
    settings.language = settings
        .language
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
}

pub(crate) fn retire_legacy_combined_ui_settings_if_split() -> Result<()> {
    let legacy_path = legacy_combined_ui_settings_path();
    if legacy_path.exists()
        && host_ui_settings_path().exists()
        && storylock_ui_settings_path().exists()
    {
        fs::remove_file(legacy_path)?;
    }
    Ok(())
}

pub(crate) fn cleanup_legacy_host_config_storylock_templates() -> Result<()> {
    for file_name in [
        "login-sites.json",
        "signing-actions.json",
        "agent-tasks.json",
    ] {
        let path = exe_config_dir().join(file_name);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub(crate) fn load_storylock_ui_settings() -> StoryLockUiSettings {
    let path = storylock_ui_settings_path();
    let mut settings = if path.exists() {
        read_storylock_ui_settings(&path).unwrap_or_default()
    } else {
        read_storylock_ui_settings(&legacy_combined_ui_settings_path()).unwrap_or_default()
    };
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
    let _guard = begin_storylock_path_dialog_once()?;
    let mut folder_dialog = rfd::FileDialog::new();
    if initial_dir.exists() {
        folder_dialog = folder_dialog.set_directory(initial_dir);
    }
    folder_dialog.pick_folder()
}

pub(crate) fn pick_storylock_folder_once(
    initial_dir: &Path,
    configure: impl FnOnce(rfd::FileDialog) -> rfd::FileDialog,
) -> Option<PathBuf> {
    let _guard = begin_storylock_path_dialog_once()?;
    let mut dialog = rfd::FileDialog::new();
    if initial_dir.exists() {
        dialog = dialog.set_directory(initial_dir);
    }
    configure(dialog).pick_folder()
}

pub(crate) fn pick_host_config_file_once(initial_file: &Path) -> Option<PathBuf> {
    let _guard = begin_storylock_path_dialog_once()?;
    let mut dialog = rfd::FileDialog::new().add_filter("Host config", &["json"]);
    if let Some(parent) = initial_file.parent().filter(|path| path.exists()) {
        dialog = dialog.set_directory(parent);
    }
    dialog.pick_file()
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
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
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

pub(crate) fn merge_storylock_package_settings(
    current: &StoryLockUiSettings,
    core_data_dir: Option<String>,
) -> StoryLockUiSettings {
    let mut settings = current.clone();
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
        .map(|value| {
            resolve_storylock_core_package_path(value)
                .display()
                .to_string()
        });
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
