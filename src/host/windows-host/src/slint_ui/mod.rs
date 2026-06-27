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
    ensure_storylock_core_package, host_learning_plan_status, initialize_storylock_core_window,
    set_storylock_start_page_to_questions, storylock_core_package_dir,
    wire_storylock_core_callbacks,
};

mod confirmation;
mod dashboard;
mod storylock;

pub use confirmation::confirm_request;
pub use dashboard::run;

slint::slint! {
    import { HostDashboard, SettingsDialog } from "host_dashboard.slint";
    import { StoryLockCoreApp } from "storylock_core.slint";
    import { StoryLockCoreSettingsDialog, AnswerEditorDialog } from "storylock_core/dialogs.slint";
    import { RequestConfirmation } from "request_confirmation.slint";

    export {
        HostDashboard,
        SettingsDialog,
        StoryLockCoreApp,
        StoryLockCoreSettingsDialog,
        AnswerEditorDialog,
        RequestConfirmation
    }
}
