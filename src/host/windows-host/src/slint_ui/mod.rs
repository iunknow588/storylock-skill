use crate::dpapi_protect_to_base64;
use crate::dpapi_unprotect_from_base64;
use crate::ProtectedEnvelope;
use crate::WindowsHostConfig;
use anyhow::Result;
use reqwest::blocking::Client;
use serde_json::json;
use serde_json::Value;
use slint::SharedString;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
};

use self::storylock::{
    apply_storylock_ui_settings, cleanup_legacy_host_config_storylock_templates,
    ensure_storylock_core_package, host_learning_plan_status, host_ui_settings_path,
    initial_storylock_core_package_dir, initialize_storylock_core_empty_window,
    initialize_storylock_core_window, load_host_ui_settings, load_storylock_ui_settings,
    merge_host_settings, merge_storylock_package_settings, normalize_storylock_ui_settings,
    package_dir_status_report, pick_host_config_file_once, pick_storylock_core_package_path,
    read_effective_author_draft, resolve_storylock_core_package_path,
    resolve_storylock_core_package_with_conflict_prompt,
    retire_legacy_combined_ui_settings_if_split, save_host_ui_settings, save_storylock_ui_settings,
    set_storylock_start_page_to_questions, wire_storylock_core_callbacks, StoryLockUiSettings,
};

mod confirmation;
mod dashboard;
mod puzzle_adapter;
mod storylock;

pub use confirmation::confirm_request;
pub use dashboard::run;

slint::slint! {
    import { HostDashboard, SettingsDialog, StoryLockAuthorizationDialog } from "host_dashboard.slint";
    import { StoryLockCoreApp } from "storylock_core.slint";
    import { StoryLockCoreSettingsDialog, AnswerEditorDialog, ObjectEditorDialog, LearningTestDialog } from "storylock_core/dialogs.slint";
    import { RequestConfirmation } from "request_confirmation.slint";

    export {
        HostDashboard,
        SettingsDialog,
        StoryLockAuthorizationDialog,
        StoryLockCoreApp,
        StoryLockCoreSettingsDialog,
        AnswerEditorDialog,
        ObjectEditorDialog,
        LearningTestDialog,
        RequestConfirmation
    }
}
