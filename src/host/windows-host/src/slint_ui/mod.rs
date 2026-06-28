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

use self::storylock::{
    apply_storylock_ui_settings, ensure_storylock_core_package, host_learning_plan_status,
    initial_storylock_core_package_dir, initialize_storylock_core_window,
    load_storylock_ui_settings, merge_host_language_setting, save_storylock_ui_settings,
    set_storylock_start_page_to_questions, wire_storylock_core_callbacks,
};

mod confirmation;
mod dashboard;
mod storylock;

pub use confirmation::confirm_request;
pub use dashboard::run;

slint::slint! {
    import { HostDashboard, SettingsDialog } from "host_dashboard.slint";
    import { StoryLockCoreApp } from "storylock_core.slint";
    import { StoryLockCoreSettingsDialog, AnswerEditorDialog, ObjectEditorDialog, LearningTestDialog } from "storylock_core/dialogs.slint";
    import { RequestConfirmation } from "request_confirmation.slint";

    export {
        HostDashboard,
        SettingsDialog,
        StoryLockCoreApp,
        StoryLockCoreSettingsDialog,
        AnswerEditorDialog,
        ObjectEditorDialog,
        LearningTestDialog,
        RequestConfirmation
    }
}
