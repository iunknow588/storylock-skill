#![cfg_attr(windows, windows_subsystem = "windows")]

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Response, Server, StatusCode};
use uuid::Uuid;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONQUESTION, MB_YESNO};

mod host_runtime;
#[cfg(feature = "ui-slint")]
mod slint_ui;
#[cfg(test)]
use host_runtime::io::{question_bank_import, ui_status};
#[cfg(test)]
use host_runtime::local_core::{
    authorize_local_action, create_grid_verification, execute_request, revoke_local_authorization,
};
#[cfg(test)]
use host_runtime::state::{load_or_init_question_bank, WindowsHostRuntime};
pub(crate) use host_runtime::state::{ProtectedEnvelope, WindowsHostConfig};
#[cfg(test)]
use host_runtime::story_templates::{story_template_candidates, story_template_generate};
use host_runtime::ui::{
    dpapi_protect_to_base64 as runtime_dpapi_protect_to_base64,
    dpapi_unprotect_from_base64 as runtime_dpapi_unprotect_from_base64, main as host_runtime_main,
};

fn main() -> Result<()> {
    host_runtime_main()
}

pub(crate) fn dpapi_protect_to_base64(plain_text: &[u8]) -> Result<String> {
    runtime_dpapi_protect_to_base64(plain_text)
}

pub(crate) fn dpapi_unprotect_from_base64(cipher_text: &str) -> Result<Vec<u8>> {
    runtime_dpapi_unprotect_from_base64(cipher_text)
}

#[cfg(test)]
mod tests;
