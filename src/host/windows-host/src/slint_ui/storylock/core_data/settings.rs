use super::*;
use serde::{Deserialize, Serialize};
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
    read_storylock_ui_settings(&storylock_ui_settings_path()).unwrap_or_default()
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(settings)?)?;
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
        .map(PathBuf::from)
        .unwrap_or_else(storylock_core_package_dir)
}

pub(crate) fn merge_host_language_setting(
    current: &StoryLockUiSettings,
    language: &str,
) -> StoryLockUiSettings {
    let mut settings = current.clone();
    settings.language = Some(language.to_string());
    settings
}

pub(crate) fn settings_from_storylock_core(core: &StoryLockCoreApp) -> StoryLockUiSettings {
    StoryLockUiSettings {
        language: non_empty_setting(core.get_language().as_str()),
        core_data_dir: non_empty_setting(core.get_core_data_dir().as_str()),
        export_package_dir: non_empty_setting(core.get_export_package_dir().as_str()),
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

fn non_empty_setting(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
