#![cfg_attr(all(windows, feature = "ui-slint"), windows_subsystem = "windows")]

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
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

#[cfg(feature = "ui-slint")]
mod slint_ui;
#[derive(Clone, Debug, Serialize)]
struct WindowsHostConfig {
    product: String,
    implementation: String,
    version: String,
    gateway_base_url: String,
    identity_id: String,
    device_id: String,
    app_instance_id: String,
    shared_secret: String,
    preferred_mode: String,
    host_port: u16,
    health_url: String,
    execute_url: String,
    register_path: String,
    relay_poll_path: String,
    relay_respond_path: String,
    approval_mode: String,
    remote_enabled: bool,
    data_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RegistrationResponse {
    relay: Option<RelayEndpoints>,
}

#[derive(Debug, Deserialize)]
struct RelayEndpoints {
    #[serde(rename = "pollUrl")]
    poll_url: Option<String>,
    #[serde(rename = "respondUrl")]
    respond_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredCredential {
    username: String,
    password: String,
    target_origin: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredAuthorizationRecord {
    verification_id: String,
    authorization_id: String,
    capability: String,
    object_ref: String,
    identity_id: String,
    allowed_action: String,
    required_strength: String,
    confirmation_method: String,
    created_at: String,
    expires_at: String,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredVerificationRecord {
    verification_id: String,
    identity_id: String,
    object_ref: String,
    capability: String,
    allowed_action: String,
    required_strength: String,
    grid_size: u32,
    required_cells: u32,
    cells: Vec<VerificationCell>,
    created_at: String,
    expires_at: String,
    status: String,
}

#[derive(Clone, Debug)]
struct AuthorizationChannelPolicy {
    channel: &'static str,
    required_strength: &'static str,
    allowed_action: &'static str,
    grid_size: u32,
    required_cells: u32,
    remote_allowed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VerificationCell {
    cell_id: String,
    prompt_ref: String,
    question_id: String,
    version_tag: String,
    prompt_text: String,
    expected_answer: String,
    position: u32,
    question_set_version: String,
    normalization_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct QuestionBankFile {
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    #[serde(rename = "questionSetVersion")]
    question_set_version: String,
    #[serde(rename = "normalizationVersion")]
    normalization_version: String,
    questions: Vec<QuestionBankEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct QuestionBankEntry {
    #[serde(rename = "questionId")]
    question_id: String,
    #[serde(rename = "promptRef")]
    prompt_ref: String,
    #[serde(rename = "versionTag")]
    version_tag: String,
    #[serde(rename = "promptText")]
    prompt_text: String,
    answer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProtectedEnvelope {
    schema_version: String,
    protected_by: String,
    created_at: String,
    cipher_text: String,
}

#[derive(Clone, Debug, Serialize)]
struct RuntimeUiState {
    started_at: String,
    relay_status: String,
    last_relay_error: Option<String>,
    last_relay_poll_at: Option<String>,
    last_execution: Option<Value>,
    last_confirmation: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalAuditEvent {
    timestamp: String,
    event_type: String,
    request_id: String,
    capability: String,
    identity_id: String,
    device_id: String,
    object_ref: Option<String>,
    result: String,
    error_code: Option<String>,
    error_type: Option<String>,
    redaction_level: String,
    meta: Value,
}

impl WindowsHostConfig {
    fn from_env() -> Self {
        let gateway_base_url = env_or("STORYLOCK_GATEWAY_URL", "https://yian.cdao.online");
        let identity_id = env_or("STORYLOCK_IDENTITY_ID", "windows-demo-001");
        let device_id = env_or(
            "STORYLOCK_DEVICE_ID",
            &format!("windows-{}", Uuid::new_v4()),
        );
        let app_instance_id = env_or("STORYLOCK_APP_INSTANCE_ID", &Uuid::new_v4().to_string());
        let shared_secret = env_or(
            "STORYLOCK_ANDROID_SHARED_SECRET",
            &env_or("STORYLOCK_SHARED_SECRET", ""),
        );
        let host_port = env_or("STORYLOCK_WINDOWS_HOST_PORT", "4510")
            .parse::<u16>()
            .unwrap_or(4510);
        let approval_mode = env_or("STORYLOCK_WINDOWS_APPROVAL_MODE", "windows_dialog");
        let remote_enabled = truthy_env("STORYLOCK_WINDOWS_REMOTE_ENABLED", false);
        let data_dir = resolve_data_dir();

        Self {
            product: "Yian Windows Host".to_string(),
            implementation: "rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            gateway_base_url,
            identity_id,
            device_id,
            app_instance_id,
            shared_secret,
            preferred_mode: "relay_url".to_string(),
            host_port,
            health_url: format!("http://127.0.0.1:{host_port}/health"),
            execute_url: format!("http://127.0.0.1:{host_port}/execute"),
            register_path: "/local-host/register".to_string(),
            relay_poll_path: "/local-host/relay/poll".to_string(),
            relay_respond_path: "/local-host/relay/respond".to_string(),
            approval_mode,
            remote_enabled,
            data_dir,
        }
    }

    fn gateway_url(&self, path: &str) -> String {
        format!("{}{}", self.gateway_base_url.trim_end_matches('/'), path)
    }

    fn health_json(&self) -> Value {
        json!({
            "schemaVersion": "windows-host-health-v1",
            "product": self.product,
            "implementation": self.implementation,
            "version": self.version,
            "deviceId": self.device_id,
            "appInstanceId": self.app_instance_id,
            "identityId": self.identity_id,
            "preferredMode": self.preferred_mode,
            "hostPort": self.host_port,
            "serverRunning": true,
            "remoteEnabled": self.remote_enabled,
            "capabilities": if self.remote_enabled {
                json!(["health", "verify", "authorize", "revoke", "execute", "relay_poll"])
            } else {
                json!(["health", "verify", "authorize", "revoke", "execute"])
            },
            "status": "local_core_prototype",
            "core": {
                "name": "StoryLock Local Core",
                "boundary": "windows_dpapi_local_only",
                "callChain": ["verify", "authorize", "execute", "revoke"]
            },
            "approvalMode": self.approval_mode,
            "storage": {
                "provider": "dpapi",
                "visibility": "host_internal_only"
            },
            "questionBank": {
                "visibility": "host_internal_only"
            }
        })
    }
}

#[derive(Clone)]
struct WindowsHostRuntime {
    config: WindowsHostConfig,
    secret_store: SecretStore,
    question_bank: Arc<Mutex<QuestionBankFile>>,
    ui_state: Arc<Mutex<RuntimeUiState>>,
}

impl WindowsHostRuntime {
    fn new(config: WindowsHostConfig) -> Result<Self> {
        let secret_store = SecretStore::new(config.data_dir.clone())?;
        let question_bank = load_or_init_question_bank(&config.data_dir)?;
        Ok(Self {
            config,
            secret_store,
            question_bank: Arc::new(Mutex::new(question_bank)),
            ui_state: Arc::new(Mutex::new(RuntimeUiState {
                started_at: now_timestamp(),
                relay_status: "starting".to_string(),
                last_relay_error: None,
                last_relay_poll_at: None,
                last_execution: None,
                last_confirmation: None,
            })),
        })
    }

    fn current_question_bank(&self) -> Result<QuestionBankFile> {
        self.question_bank
            .lock()
            .map(|bank| bank.clone())
            .map_err(|_| anyhow!("question bank lock was poisoned"))
    }

    fn replace_question_bank(&self, next: QuestionBankFile) -> Result<()> {
        let mut bank = self
            .question_bank
            .lock()
            .map_err(|_| anyhow!("question bank lock was poisoned"))?;
        *bank = next;
        Ok(())
    }

    fn set_relay_status(&self, status: &str, error: Option<String>) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.relay_status = status.to_string();
            state.last_relay_error = error;
            state.last_relay_poll_at = Some(now_timestamp());
        }
    }

    fn record_execution_summary(&self, response: &Value) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.last_execution = Some(summarize_execution_for_ui(response));
        }
    }

    fn record_confirmation_summary(&self, summary: Value) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.last_confirmation = Some(summary);
        }
    }

    fn ui_state_snapshot(&self) -> RuntimeUiState {
        self.ui_state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| RuntimeUiState {
                started_at: now_timestamp(),
                relay_status: "local_only".to_string(),
                last_relay_error: Some("ui state lock was poisoned".to_string()),
                last_relay_poll_at: None,
                last_execution: None,
                last_confirmation: None,
            })
    }
}

#[derive(Clone)]
struct SecretStore {
    root: PathBuf,
}

impl SecretStore {
    fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(root.join("keys"))?;
        fs::create_dir_all(root.join("credentials"))?;
        fs::create_dir_all(root.join("authorizations"))?;
        fs::create_dir_all(root.join("audit"))?;
        fs::create_dir_all(root.join("story-template-requests"))?;
        Ok(Self { root })
    }

    fn signature_key_path(&self, key_id: &str) -> PathBuf {
        self.root
            .join("keys")
            .join(format!("{}.json", sanitize_ref(key_id)))
    }

    fn credential_path(&self, credential_ref: &str) -> PathBuf {
        self.root
            .join("credentials")
            .join(format!("{}.json", sanitize_ref(credential_ref)))
    }

    fn authorization_path(&self, authorization_id: &str) -> PathBuf {
        self.root
            .join("authorizations")
            .join(format!("{}.json", sanitize_ref(authorization_id)))
    }

    fn verification_path(&self, verification_id: &str) -> PathBuf {
        self.root.join("authorizations").join(format!(
            "verification-{}.json",
            sanitize_ref(verification_id)
        ))
    }

    fn audit_log_path(&self) -> PathBuf {
        self.root.join("audit").join("local-audit.jsonl")
    }

    fn story_template_candidates_path(&self) -> PathBuf {
        self.root
            .join("story-template-requests")
            .join("story-template-candidates.jsonl")
    }

    fn story_template_interface_manifest_path(&self) -> PathBuf {
        self.root
            .join("story-template-requests")
            .join("interface-manifest.json")
    }

    fn get_or_create_signature_key(&self, key_id: &str) -> Result<String> {
        let path = self.signature_key_path(key_id);
        if path.exists() {
            return self.read_secret_string(&path);
        }
        let material = format!("sigkey-{}-{}", key_id, Uuid::new_v4());
        self.write_secret_string(&path, &material)?;
        Ok(material)
    }

    fn get_or_create_credential(
        &self,
        credential_ref: &str,
        username_hint: Option<&str>,
        target_origin: Option<&str>,
    ) -> Result<StoredCredential> {
        let path = self.credential_path(credential_ref);
        if path.exists() {
            return self.read_secret_json(&path);
        }
        let credential = StoredCredential {
            username: username_hint.unwrap_or("windows-user").to_string(),
            password: format!("pw-{}-{}", sanitize_ref(credential_ref), short_id()),
            target_origin: target_origin.unwrap_or("https://example.test").to_string(),
        };
        self.write_secret_json(&path, &credential)?;
        Ok(credential)
    }

    fn write_secret_string(&self, path: &Path, secret: &str) -> Result<()> {
        let envelope = ProtectedEnvelope {
            schema_version: "dpapi-protected-v1".to_string(),
            protected_by: "windows-dpapi".to_string(),
            created_at: now_timestamp(),
            cipher_text: dpapi_protect_to_base64(secret.as_bytes())?,
        };
        let serialized = serde_json::to_vec_pretty(&envelope)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    fn read_secret_string(&self, path: &Path) -> Result<String> {
        let envelope: ProtectedEnvelope = serde_json::from_slice(&fs::read(path)?)?;
        let decrypted = dpapi_unprotect_from_base64(&envelope.cipher_text)?;
        String::from_utf8(decrypted).context("stored secret was not valid utf-8")
    }

    fn write_secret_json<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec(value)?;
        let envelope = ProtectedEnvelope {
            schema_version: "dpapi-protected-v1".to_string(),
            protected_by: "windows-dpapi".to_string(),
            created_at: now_timestamp(),
            cipher_text: dpapi_protect_to_base64(&bytes)?,
        };
        fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;
        Ok(())
    }

    fn read_secret_json<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T> {
        let envelope: ProtectedEnvelope = serde_json::from_slice(&fs::read(path)?)?;
        let decrypted = dpapi_unprotect_from_base64(&envelope.cipher_text)?;
        Ok(serde_json::from_slice(&decrypted)?)
    }

    fn write_authorization_record(&self, record: &StoredAuthorizationRecord) -> Result<()> {
        self.write_secret_json(&self.authorization_path(&record.authorization_id), record)
    }

    fn write_verification_record(&self, record: &StoredVerificationRecord) -> Result<()> {
        self.write_secret_json(&self.verification_path(&record.verification_id), record)
    }

    fn read_verification_record(&self, verification_id: &str) -> Result<StoredVerificationRecord> {
        self.read_secret_json(&self.verification_path(verification_id))
    }

    fn read_authorization_record(
        &self,
        authorization_id: &str,
    ) -> Result<StoredAuthorizationRecord> {
        self.read_secret_json(&self.authorization_path(authorization_id))
    }

    fn append_audit_event(&self, event: &LocalAuditEvent) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.audit_log_path())?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    fn append_story_template_candidate(&self, candidate: &Value) -> Result<()> {
        write_host_json_if_missing(
            &self.story_template_interface_manifest_path(),
            &story_template_interface_manifest(),
        )?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.story_template_candidates_path())?;
        writeln!(file, "{}", serde_json::to_string(candidate)?)?;
        Ok(())
    }

    fn read_story_template_candidates(&self, limit: usize) -> Vec<Value> {
        let Ok(content) = fs::read_to_string(self.story_template_candidates_path()) else {
            return Vec::new();
        };
        let mut items = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect::<Vec<_>>();
        items.reverse();
        items.truncate(limit);
        items
    }
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn truthy_env(name: &str, fallback: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => fallback,
    }
}

fn resolve_data_dir() -> PathBuf {
    if let Ok(configured) = std::env::var("STORYLOCK_WINDOWS_DATA_DIR") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(appdata).join("Yian").join("windows-host");
    }
    PathBuf::from(".").join(".windows-host-data")
}

fn question_bank_path(data_dir: &Path) -> PathBuf {
    data_dir.join("question-bank.json")
}

fn write_host_json_if_missing(path: &Path, value: &Value) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
    }
    Ok(())
}

fn default_question_bank_json() -> &'static str {
    include_str!("../assets/question-bank.json")
}

fn load_or_init_question_bank(data_dir: &Path) -> Result<QuestionBankFile> {
    let path = question_bank_path(data_dir);
    if !path.exists() {
        fs::create_dir_all(data_dir)?;
        fs::write(&path, default_question_bank_json())?;
    }
    read_and_validate_question_bank(&path)
}

fn read_and_validate_question_bank(path: &Path) -> Result<QuestionBankFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read question bank file: {}", path.display()))?;
    let parsed: QuestionBankFile = serde_json::from_str(content.trim_start_matches('\u{feff}'))
        .with_context(|| format!("failed to parse question bank file: {}", path.display()))?;
    validate_question_bank(&parsed)?;
    Ok(parsed)
}

fn validate_question_bank(question_bank: &QuestionBankFile) -> Result<()> {
    if question_bank.schema_version.trim().is_empty() {
        return Err(anyhow!("question bank schemaVersion must be non-empty"));
    }
    if question_bank.question_set_version.trim().is_empty() {
        return Err(anyhow!(
            "question bank questionSetVersion must be non-empty"
        ));
    }
    if question_bank.normalization_version.trim().is_empty() {
        return Err(anyhow!(
            "question bank normalizationVersion must be non-empty"
        ));
    }
    if question_bank.questions.is_empty() {
        return Err(anyhow!("question bank file contains no questions"));
    }
    for (index, question) in question_bank.questions.iter().enumerate() {
        if question.question_id.trim().is_empty() {
            return Err(anyhow!("question {} has empty questionId", index + 1));
        }
        if question.prompt_ref.trim().is_empty() {
            return Err(anyhow!("question {} has empty promptRef", index + 1));
        }
        if question.version_tag.trim().is_empty() {
            return Err(anyhow!("question {} has empty versionTag", index + 1));
        }
        if question.prompt_text.trim().is_empty() {
            return Err(anyhow!("question {} has empty promptText", index + 1));
        }
        if question.answer.trim().is_empty() {
            return Err(anyhow!("question {} has empty answer", index + 1));
        }
    }
    Ok(())
}

fn import_question_bank(data_dir: &Path, source_path: &Path) -> Result<QuestionBankFile> {
    let imported = read_and_validate_question_bank(source_path)?;
    fs::create_dir_all(data_dir)?;
    fs::copy(source_path, question_bank_path(data_dir)).with_context(|| {
        format!(
            "failed to copy question bank from {} into {}",
            source_path.display(),
            data_dir.display()
        )
    })?;
    Ok(imported)
}

fn sanitize_ref(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn short_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

fn content_type_json() -> Header {
    Header::from_bytes(
        &b"content-type"[..],
        &b"application/json; charset=utf-8"[..],
    )
    .expect("static header is valid")
}

fn content_type_html() -> Header {
    Header::from_bytes(&b"content-type"[..], &b"text/html; charset=utf-8"[..])
        .expect("static header is valid")
}

fn request_id_from(request: &Value) -> String {
    request
        .get("requestId")
        .and_then(Value::as_str)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("req-{}", Uuid::new_v4()))
}

fn capability_from(request: &Value) -> String {
    request
        .get("capability")
        .and_then(Value::as_str)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "requestSignature".to_string())
}

fn now_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
        .to_string()
}

fn error_response(
    config: &WindowsHostConfig,
    request_id: &str,
    capability: &str,
    code: &str,
    error_type: &str,
    message: &str,
    suggested_action: &str,
) -> Value {
    json!({
        "requestId": request_id,
        "status": "error",
        "capability": capability,
        "executionLocation": "local",
        "result": Value::Null,
        "redactionLevel": "full",
        "retentionGranted": "audit_meta_only",
        "auditMeta": {
            "timestamp": now_timestamp(),
            "localHost": "windows-rust-prototype",
            "identityId": config.identity_id,
            "deviceId": config.device_id,
            "approvalMode": config.approval_mode
        },
        "error": {
            "code": code,
            "type": error_type,
            "message": message,
            "suggestedAction": suggested_action
        }
    })
}

fn record_local_audit(
    runtime: &WindowsHostRuntime,
    event_type: &str,
    request_id: &str,
    capability: &str,
    object_ref: Option<&str>,
    result: &str,
    error_code: Option<&str>,
    error_type: Option<&str>,
    meta: Value,
) {
    let event = LocalAuditEvent {
        timestamp: now_timestamp(),
        event_type: event_type.to_string(),
        request_id: request_id.to_string(),
        capability: capability.to_string(),
        identity_id: runtime.config.identity_id.clone(),
        device_id: runtime.config.device_id.clone(),
        object_ref: object_ref.map(ToOwned::to_owned),
        result: result.to_string(),
        error_code: error_code.map(ToOwned::to_owned),
        error_type: error_type.map(ToOwned::to_owned),
        redaction_level: "audit_meta_only".to_string(),
        meta,
    };
    let _ = runtime.secret_store.append_audit_event(&event);
}

fn summarize_execution_for_ui(response: &Value) -> Value {
    let result = response.get("result").unwrap_or(&Value::Null);
    let audit = response.get("auditMeta").unwrap_or(&Value::Null);
    json!({
        "requestId": response.get("requestId").and_then(Value::as_str).unwrap_or(""),
        "status": response.get("status").and_then(Value::as_str).unwrap_or("unknown"),
        "capability": response.get("capability").and_then(Value::as_str).unwrap_or("unknown"),
        "objectRef": audit.get("objectRef").and_then(Value::as_str)
            .or_else(|| result.get("keyId").and_then(Value::as_str))
            .or_else(|| result.get("credentialRef").and_then(Value::as_str))
            .unwrap_or(""),
        "verificationId": audit.get("verificationId").and_then(Value::as_str)
            .or_else(|| result.get("verificationId").and_then(Value::as_str))
            .unwrap_or(""),
        "authorizationId": audit.get("authorizationId").and_then(Value::as_str)
            .or_else(|| result.get("authorizationId").and_then(Value::as_str))
            .unwrap_or(""),
        "requiredStrength": audit.get("requiredStrength").and_then(Value::as_str)
            .or_else(|| result.get("requiredStrength").and_then(Value::as_str))
            .unwrap_or(""),
        "allowedAction": audit.get("allowedAction").and_then(Value::as_str)
            .or_else(|| result.get("allowedAction").and_then(Value::as_str))
            .unwrap_or(""),
        "redactionLevel": response.get("redactionLevel").and_then(Value::as_str).unwrap_or("full"),
        "timestamp": audit.get("timestamp").and_then(Value::as_str).unwrap_or(""),
        "errorType": response.get("error")
            .and_then(|error| error.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("")
    })
}

fn signature_of_request(key_material: &str, request: &Value) -> Result<String> {
    let canonical = serde_json::to_vec(request)?;
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    hasher.update(b":");
    hasher.update(canonical);
    Ok(format!("sha256:{}", hex_string(&hasher.finalize())))
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn show_windows_confirmation_dialog(title: &str, body: &str) -> bool {
    let title_w = wide_null(title);
    let body_w = wide_null(body);
    let response = unsafe {
        MessageBoxW(
            HWND::default(),
            body_w.as_ptr(),
            title_w.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };
    response == IDYES
}

fn requester_from(request: &Value) -> String {
    request
        .get("requester")
        .or_else(|| request.get("origin"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown requester")
        .to_string()
}

fn origin_from(request: &Value) -> String {
    request
        .get("origin")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown origin")
        .to_string()
}

fn risk_description_for(capability: &str) -> &'static str {
    if capability == "requestSignature" {
        "High sensitivity: approval signs with a local DPAPI-protected key. Verify the requester and object before approving."
    } else {
        "Medium-high sensitivity: approval allows local credential fill. Verify the target origin before approving."
    }
}

fn confirmation_summary_for(
    config: &WindowsHostConfig,
    request: &Value,
    capability: &str,
    object_ref: &str,
    status: &str,
) -> Value {
    let expires_at = expires_at_after(300);
    let policy = channel_policy_for_request(capability, request).unwrap_or_else(|_| {
        AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: required_strength_for(capability),
            allowed_action: allowed_action_for(capability),
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        }
    });
    json!({
        "requestId": request_id_from(request),
        "status": status,
        "capability": capability,
        "objectRef": object_ref,
        "requester": requester_from(request),
        "origin": origin_from(request),
        "requiredStrength": policy.required_strength,
        "allowedAction": policy.allowed_action,
        "authorizationChannel": policy.channel,
        "expiry": expires_at,
        "risk": risk_description_for(capability),
        "approvalMode": config.approval_mode,
        "redactionLevel": "audit_meta_only",
        "hiddenFromUi": ["answers", "password", "privateKey", "signingKeyBytes", "storyRawText"],
        "timestamp": now_timestamp()
    })
}

fn known_authorization_modes() -> Vec<AuthorizationChannelPolicy> {
    vec![
        AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: "medium",
            allowed_action: "password_fill_or_signature",
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        },
        AuthorizationChannelPolicy {
            channel: "batch_read",
            required_strength: "high",
            allowed_action: "batch_read",
            grid_size: 12,
            required_cells: 12,
            remote_allowed: true,
        },
        AuthorizationChannelPolicy {
            channel: "story_edit",
            required_strength: "story_edit",
            allowed_action: "story_edit",
            grid_size: 24,
            required_cells: 22,
            remote_allowed: false,
        },
    ]
}

fn management_authorization_modes_json() -> Value {
    Value::Array(
        known_authorization_modes()
            .into_iter()
            .map(|policy| {
                json!({
                    "channel": policy.channel,
                    "requiredStrength": policy.required_strength,
                    "allowedAction": policy.allowed_action,
                    "gridSize": policy.grid_size,
                    "requiredCells": policy.required_cells,
                    "remoteAllowed": policy.remote_allowed
                })
            })
            .collect(),
    )
}

fn increment_counter(map: &mut BTreeMap<String, u64>, key: &str) {
    let normalized = key.trim();
    if !normalized.is_empty() {
        *map.entry(normalized.to_string()).or_insert(0) += 1;
    }
}

fn audit_meta_str<'a>(event: &'a LocalAuditEvent, key: &str) -> Option<&'a str> {
    event
        .meta
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn request_audit_context(request: &Value) -> Value {
    json!({
        "requester": requester_from(request),
        "origin": origin_from(request),
        "remoteRequest": request
            .get("remoteRequest")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "remoteInterface": request
            .get("remoteInterface")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("local_api")
    })
}

fn merge_audit_meta(mut base: Value, context: Value) -> Value {
    if let (Some(base), Some(context)) = (base.as_object_mut(), context.as_object()) {
        for (key, value) in context {
            base.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }
    base
}

fn read_local_audit_events(path: &Path) -> Vec<LocalAuditEvent> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<LocalAuditEvent>(line).ok())
        .collect()
}

fn counter_entries(map: BTreeMap<String, u64>) -> Value {
    Value::Array(
        map.into_iter()
            .map(|(name, calls)| json!({ "name": name, "calls": calls }))
            .collect(),
    )
}

fn host_management_stats(runtime: &WindowsHostRuntime) -> Value {
    let events = read_local_audit_events(&runtime.secret_store.audit_log_path());
    let mut object_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut object_successes: BTreeMap<String, u64> = BTreeMap::new();
    let mut object_failures: BTreeMap<String, u64> = BTreeMap::new();
    let mut object_last_seen: BTreeMap<String, String> = BTreeMap::new();
    let mut object_capabilities: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    let mut agent_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut channel_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut remote_interfaces: BTreeMap<String, u64> = BTreeMap::new();
    let mut error_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut total_calls = 0_u64;
    let mut success_calls = 0_u64;
    let mut failed_calls = 0_u64;
    let mut remote_calls = 0_u64;

    for event in &events {
        total_calls += 1;
        let result = event.result.as_str();
        if matches!(result, "success" | "approved") {
            success_calls += 1;
        }
        if matches!(result, "error" | "failed" | "denied") || event.error_code.is_some() {
            failed_calls += 1;
        }

        if let Some(object_ref) = event.object_ref.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            increment_counter(&mut object_calls, object_ref);
            object_last_seen.insert(object_ref.to_string(), event.timestamp.clone());
            increment_counter(
                object_capabilities
                    .entry(object_ref.to_string())
                    .or_default(),
                &event.capability,
            );
            if matches!(result, "success" | "approved") {
                increment_counter(&mut object_successes, object_ref);
            }
            if matches!(result, "error" | "failed" | "denied") || event.error_code.is_some() {
                increment_counter(&mut object_failures, object_ref);
            }
        }

        let requester = audit_meta_str(event, "requester")
            .or_else(|| audit_meta_str(event, "agentId"))
            .or_else(|| audit_meta_str(event, "origin"))
            .unwrap_or("unknown agent");
        increment_counter(&mut agent_calls, requester);

        if let Some(channel) = audit_meta_str(event, "authorizationChannel") {
            increment_counter(&mut channel_calls, channel);
        } else if let Some(action) = audit_meta_str(event, "allowedAction") {
            increment_counter(&mut channel_calls, action);
        }

        let remote_interface = audit_meta_str(event, "remoteInterface")
            .or_else(|| audit_meta_str(event, "origin"))
            .unwrap_or(if event.meta.get("remoteRequest").and_then(Value::as_bool).unwrap_or(false) {
                "remote_gateway"
            } else {
                "local_api"
            });
        increment_counter(&mut remote_interfaces, remote_interface);
        if remote_interface != "local_api"
            || event
                .meta
                .get("remoteRequest")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        {
            remote_calls += 1;
        }

        if let Some(error_code) = event.error_code.as_deref() {
            let key = match event.error_type.as_deref() {
                Some(error_type) if !error_type.trim().is_empty() => {
                    format!("{error_code}:{error_type}")
                }
                _ => error_code.to_string(),
            };
            increment_counter(&mut error_calls, &key);
        }
    }

    let objects = Value::Array(
        object_calls
            .into_iter()
            .map(|(object_ref, calls)| {
                json!({
                    "objectRef": object_ref,
                    "calls": calls,
                    "successes": object_successes.get(&object_ref).copied().unwrap_or(0),
                    "failures": object_failures.get(&object_ref).copied().unwrap_or(0),
                    "lastSeenAt": object_last_seen.get(&object_ref).cloned().unwrap_or_default(),
                    "capabilities": counter_entries(object_capabilities.remove(&object_ref).unwrap_or_default())
                })
            })
            .collect(),
    );

    json!({
        "schemaVersion": "windows-host-management-stats-v1",
        "generatedAt": now_timestamp(),
        "authorizationModes": management_authorization_modes_json(),
        "summary": {
            "auditEvents": total_calls,
            "successes": success_calls,
            "failures": failed_calls,
            "managedObjects": objects.as_array().map(Vec::len).unwrap_or(0),
            "remoteInterfaceCalls": remote_calls
        },
        "objects": objects,
        "agents": counter_entries(agent_calls),
        "authorizationChannels": counter_entries(channel_calls),
        "remoteInterfaces": counter_entries(remote_interfaces),
        "errors": counter_entries(error_calls),
        "redaction": {
            "level": "audit_meta_only",
            "hidden": ["answers", "password", "privateKey", "signingKeyBytes", "storyRawText", "sharedSecret", "drafts", "vaultFiles", "packagePaths"]
        }
    })
}

fn request_summary(request: &Value, capability: &str, object_ref: &str) -> String {
    let requester = request
        .get("requester")
        .or_else(|| request.get("origin"))
        .and_then(Value::as_str)
        .unwrap_or("unknown requester");
    let origin = origin_from(request);
    let policy = channel_policy_for_request(capability, request).unwrap_or_else(|_| {
        AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: required_strength_for(capability),
            allowed_action: allowed_action_for(capability),
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        }
    });
    let required_strength = policy.required_strength;
    let allowed_action = policy.allowed_action;
    let expiry = expires_at_after(300);
    let risk = risk_description_for(capability);
    format!(
        "Approve local execution?\n\nCapability: {capability}\nObject: {object_ref}\nRequester: {requester}\nOrigin: {origin}\nRequired strength: {required_strength}\nAllowed action: {allowed_action}\nAuthorization channel: {}\nExpires at: {expiry}\n\nRisk:\n{risk}\n\nChoose Yes to allow this request on the Windows host.",
        policy.channel
    )
}

#[cfg(feature = "ui-slint")]
fn show_slint_confirmation_dialog(summary: &Value) -> bool {
    match slint_ui::confirm_request(summary) {
        Ok(approved) => approved,
        Err(error) => {
            eprintln!("Slint confirmation failed; falling back to Windows dialog: {error}");
            show_windows_confirmation_dialog(
                "Yian Windows Host Confirmation",
                &format!(
                    "Approve local execution?\n\n{}\n\nChoose Yes to allow this request on the Windows host.",
                    serde_json::to_string_pretty(summary)
                        .unwrap_or_else(|_| "request details unavailable".to_string())
                ),
            )
        }
    }
}

#[cfg(not(feature = "ui-slint"))]
fn show_slint_confirmation_dialog(summary: &Value) -> bool {
    eprintln!(
        "STORYLOCK_WINDOWS_APPROVAL_MODE=slint_dialog requires the ui-slint feature; falling back to Windows dialog."
    );
    show_windows_confirmation_dialog(
        "Yian Windows Host Confirmation",
        &format!(
            "Approve local execution?\n\n{}\n\nChoose Yes to allow this request on the Windows host.",
            serde_json::to_string_pretty(summary)
                .unwrap_or_else(|_| "request details unavailable".to_string())
        ),
    )
}

fn is_confirmation_approved(
    runtime: &WindowsHostRuntime,
    request: &Value,
    object_ref: &str,
) -> bool {
    let config = &runtime.config;
    let capability = capability_from(request);
    let pending_summary =
        confirmation_summary_for(config, request, &capability, object_ref, "pending");
    runtime.record_confirmation_summary(pending_summary.clone());
    match config.approval_mode.as_str() {
        "auto_approve" => {
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                "approved",
            ));
            true
        }
        "auto_deny" => {
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                "denied",
            ));
            false
        }
        "slint_dialog" => {
            let approved = show_slint_confirmation_dialog(&pending_summary);
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                if approved { "approved" } else { "denied" },
            ));
            approved
        }
        _ => {
            let approved = show_windows_confirmation_dialog(
                "Yian Windows Host Confirmation",
                &request_summary(request, &capability, object_ref),
            );
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                if approved { "approved" } else { "denied" },
            ));
            approved
        }
    }
}

fn required_strength_for(capability: &str) -> &'static str {
    if capability == "requestSignature" {
        "high"
    } else {
        "medium"
    }
}

fn allowed_action_for(capability: &str) -> &'static str {
    if capability == "requestSignature" {
        "signature"
    } else {
        "password_fill"
    }
}

fn authorization_channel_for_request(capability: &str, request: &Value) -> String {
    request
        .get("authorizationChannel")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            if request
                .get("requestedAction")
                .and_then(Value::as_str)
                .is_some_and(|action| action == "story_edit")
            {
                "story_edit".to_string()
            } else if request
                .get("requestedAction")
                .and_then(Value::as_str)
                .is_some_and(|action| action == "batch_read")
            {
                "batch_read".to_string()
            } else {
                match capability {
                    "requestSignature" | "requestPasswordFill" => "single_read".to_string(),
                    _ => "single_read".to_string(),
                }
            }
        })
}

fn channel_policy_for_request(
    capability: &str,
    request: &Value,
) -> Result<AuthorizationChannelPolicy> {
    let channel = authorization_channel_for_request(capability, request);
    let policy = match channel.as_str() {
        "single_read" => AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: "medium",
            allowed_action: allowed_action_for(capability),
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        },
        "batch_read" => AuthorizationChannelPolicy {
            channel: "batch_read",
            required_strength: "high",
            allowed_action: "batch_read",
            grid_size: 12,
            required_cells: 12,
            remote_allowed: true,
        },
        "story_edit" => AuthorizationChannelPolicy {
            channel: "story_edit",
            required_strength: "story_edit",
            allowed_action: "story_edit",
            grid_size: 24,
            required_cells: 22,
            remote_allowed: false,
        },
        _ => {
            return Err(anyhow!(
                "authorizationChannel must be single_read, batch_read, or story_edit"
            ))
        }
    };
    if request
        .get("remoteRequest")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && !policy.remote_allowed
    {
        return Err(anyhow!(
            "story_edit is local-only and cannot be triggered by the remote gateway"
        ));
    }
    Ok(policy)
}

fn expires_at_after(seconds: u64) -> String {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() + seconds)
        .unwrap_or(seconds))
    .to_string()
}

fn normalize_answer(answer: &str, normalization_version: &str) -> String {
    match normalization_version {
        "upper-ascii-v1" => answer.trim().to_ascii_uppercase(),
        _ => answer.trim().to_string(),
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn is_unexpired(expires_at: &str) -> bool {
    expires_at
        .parse::<u64>()
        .map(|expiry| expiry >= now_seconds())
        .unwrap_or(false)
}

fn object_ref_for_request(capability: &str, request: &Value) -> String {
    if capability == "requestSignature" {
        request
            .get("keyId")
            .and_then(Value::as_str)
            .unwrap_or("windows-signature-key")
            .to_string()
    } else {
        request
            .get("credentialRef")
            .and_then(Value::as_str)
            .unwrap_or("windows-credential-ref")
            .to_string()
    }
}

fn validate_authorization_for_core(
    authorization: &StoredAuthorizationRecord,
    capability: &str,
    object_ref: &str,
) -> Result<()> {
    if authorization.status != "approved" {
        return Err(anyhow!("authorization session is not approved"));
    }
    if !is_unexpired(&authorization.expires_at) {
        return Err(anyhow!("authorization session expired"));
    }
    if authorization.capability != capability {
        return Err(anyhow!("authorization capability mismatch"));
    }
    if authorization.object_ref != object_ref {
        return Err(anyhow!("authorization object mismatch"));
    }
    Ok(())
}

fn local_core_call_envelope(
    config: &WindowsHostConfig,
    request_id: &str,
    authorization: &StoredAuthorizationRecord,
) -> Value {
    json!({
        "schemaVersion": "storylock-local-core-call-v1",
        "coreCallId": format!("core-{}", Uuid::new_v4()),
        "requestId": request_id,
        "identityId": config.identity_id,
        "deviceId": config.device_id,
        "authorizationId": authorization.authorization_id,
        "verificationId": authorization.verification_id,
        "capability": authorization.capability,
        "objectRef": authorization.object_ref,
        "allowedAction": authorization.allowed_action,
        "requiredStrength": authorization.required_strength,
        "expiresAt": authorization.expires_at,
        "storageProvider": "windows-dpapi",
        "secretBoundary": "local_only"
    })
}

fn build_verification_cells(
    question_bank: &QuestionBankFile,
    required_strength: &str,
    object_ref: &str,
    required_cells: u32,
) -> Vec<VerificationCell> {
    let question_set_version = &question_bank.question_set_version;
    let normalization_version = &question_bank.normalization_version;
    let bank = &question_bank.questions;
    let seed = object_ref
        .bytes()
        .fold(0usize, |acc, byte| acc.wrapping_add(byte as usize));
    (0..required_cells)
        .map(|index| {
            let entry = &bank[(seed + index as usize) % bank.len()];
            VerificationCell {
                cell_id: format!("cell-{}", index + 1),
                prompt_ref: entry.prompt_ref.clone(),
                question_id: entry.question_id.clone(),
                version_tag: entry.version_tag.clone(),
                prompt_text: format!(
                    "{} Object: {object_ref}. Strength: {required_strength}.",
                    entry.prompt_text
                ),
                expected_answer: entry.answer.clone(),
                position: index,
                question_set_version: question_set_version.to_string(),
                normalization_version: normalization_version.to_string(),
            }
        })
        .collect()
}

fn verification_record_from_policy(
    config: &WindowsHostConfig,
    question_bank: &QuestionBankFile,
    capability: &str,
    object_ref: &str,
    request: &Value,
) -> Result<StoredVerificationRecord> {
    let policy = channel_policy_for_request(capability, request)?;
    let required_strength = policy.required_strength.to_string();
    let (grid_size, required_cells) = (policy.grid_size, policy.required_cells);
    let cells = build_verification_cells(
        question_bank,
        &required_strength,
        object_ref,
        required_cells,
    );
    Ok(StoredVerificationRecord {
        verification_id: format!("ver-{}", Uuid::new_v4()),
        identity_id: config.identity_id.clone(),
        object_ref: object_ref.to_string(),
        capability: capability.to_string(),
        allowed_action: policy.allowed_action.to_string(),
        required_strength: required_strength.clone(),
        grid_size,
        required_cells,
        cells,
        created_at: now_timestamp(),
        expires_at: expires_at_after(180),
        status: "pending".to_string(),
    })
}

fn create_grid_verification(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let config = &runtime.config;
    let request_id = request_id_from(request);
    let capability = capability_from(request);
    let object_ref = if capability == "requestSignature" {
        request
            .get("keyId")
            .and_then(Value::as_str)
            .unwrap_or("windows-signature-key")
    } else {
        request
            .get("credentialRef")
            .and_then(Value::as_str)
            .unwrap_or("windows-credential-ref")
    };
    let question_bank = match runtime.current_question_bank() {
        Ok(bank) => bank,
        Err(error) => return error_response(
            config,
            &request_id,
            "createGridVerification",
            "SLG-005",
            "host_storage_error",
            &format!("failed to read active question bank: {error}"),
            "Validate or re-import the local question bank, then retry the verification request.",
        ),
    };
    let verification = match verification_record_from_policy(
        config,
        &question_bank,
        &capability,
        object_ref,
        request,
    ) {
        Ok(record) => record,
        Err(error) => {
            record_local_audit(
                runtime,
                "policy_denied",
                &request_id,
                "createGridVerification",
                Some(object_ref),
                "denied",
                Some("SLG-002"),
                Some("policy_denied"),
                json!({
                    "reason": error.to_string(),
                    "authorizationChannel": authorization_channel_for_request(&capability, request)
                }),
            );
            return error_response(
                config,
                &request_id,
                "createGridVerification",
                "SLG-002",
                "policy_denied",
                &error.to_string(),
                "Use a supported local authorization channel. Remote requests cannot trigger story_edit.",
            );
        }
    };
    if let Err(error) = runtime
        .secret_store
        .write_verification_record(&verification)
    {
        return error_response(
            config,
            &request_id,
            "createGridVerification",
            "SLG-005",
            "host_storage_error",
            &format!("failed to persist verification record: {error}"),
            "Check the Windows host data directory and DPAPI availability, then retry the request.",
        );
    }
    json!({
        "requestId": request_id,
        "status": "success",
        "capability": "createGridVerification",
        "executionLocation": "local",
        "result": {
            "verificationId": verification.verification_id,
            "identityId": verification.identity_id,
            "objectRef": verification.object_ref,
            "requiredStrength": verification.required_strength,
            "grid": {
                "gridSize": verification.grid_size,
                "requiredCells": verification.required_cells,
                "questionSetVersion": verification
                    .cells
                    .first()
                    .map(|cell| cell.question_set_version.clone())
                    .unwrap_or_else(|| "windows-local-v1".to_string()),
                "questionSetVersions": ["windows-local-v1"],
                "normalizationVersion": verification
                    .cells
                    .first()
                    .map(|cell| cell.normalization_version.clone())
                    .unwrap_or_else(|| "upper-ascii-v1".to_string()),
                "normalizationVersions": ["upper-ascii-v1"],
                "cells": verification.cells.iter().map(|cell| json!({
                    "cellId": cell.cell_id,
                    "promptRef": cell.prompt_ref,
                    "questionId": cell.question_id,
                    "versionTag": cell.version_tag,
                    "promptText": cell.prompt_text,
                    "position": cell.position,
                    "questionSetVersion": cell.question_set_version,
                    "normalizationVersion": cell.normalization_version
                })).collect::<Vec<_>>()
            },
            "expiresAt": verification.expires_at
        },
        "redactionLevel": "none",
        "retentionGranted": "audit_meta_only",
        "auditMeta": {
            "timestamp": verification.created_at,
            "verificationId": verification.verification_id
        },
        "error": Value::Null
    })
}

fn question_bank_status(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    match runtime.current_question_bank() {
        Ok(bank) => json!({
            "requestId": request_id,
            "status": "success",
            "capability": "questionBankStatus",
            "executionLocation": "local",
            "result": {
                "path": question_bank_path(&runtime.config.data_dir).display().to_string(),
                "questionSetVersion": bank.question_set_version,
                "normalizationVersion": bank.normalization_version,
                "questionCount": bank.questions.len()
            },
            "redactionLevel": "none",
            "retentionGranted": "audit_meta_only",
            "auditMeta": {
                "timestamp": now_timestamp()
            },
            "error": Value::Null
        }),
        Err(error) => error_response(
            &runtime.config,
            &request_id,
            "questionBankStatus",
            "SLG-005",
            "host_storage_error",
            &format!("failed to read active question bank: {error}"),
            "Check local Windows host storage and retry the question bank status request.",
        ),
    }
}

fn ui_status(runtime: &WindowsHostRuntime) -> Value {
    let health = runtime.config.health_json();
    let question_bank = question_bank_status(
        runtime,
        &json!({
            "requestId": format!("req-{}", Uuid::new_v4())
        }),
    );
    let state = runtime.ui_state_snapshot();
    let question_bank_result = question_bank.get("result").cloned().unwrap_or(Value::Null);
    let question_bank_summary = json!({
        "questionSetVersion": question_bank_result
            .get("questionSetVersion")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        "normalizationVersion": question_bank_result
            .get("normalizationVersion")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        "questionCount": question_bank_result
            .get("questionCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "visibility": "host_internal_only"
    });
    json!({
        "status": "success",
        "capability": "windowsHostUiStatus",
        "executionLocation": "local",
        "result": {
            "host": health,
            "relay": {
                "status": state.relay_status,
                "lastPollAt": state.last_relay_poll_at,
                "lastError": state.last_relay_error
            },
            "remote": {
                "enabled": runtime.config.remote_enabled,
                "mode": if runtime.config.remote_enabled { "relay_url" } else { "local_only" },
                "gatewayUrl": if runtime.config.remote_enabled {
                    Value::String(runtime.config.gateway_base_url.clone())
                } else {
                    Value::Null
                }
            },
            "questionBank": question_bank_summary,
            "managementStats": host_management_stats(runtime),
            "storyTemplateGenerator": story_template_generator_status(runtime),
            "lastConfirmation": state.last_confirmation,
            "lastExecution": state.last_execution,
            "ui": {
                "startedAt": state.started_at,
                "managementUrl": format!("http://127.0.0.1:{}/ui", runtime.config.host_port),
                "statusUrl": format!("http://127.0.0.1:{}/ui/status", runtime.config.host_port)
            },
            "boundaries": {
                "remoteCapabilities": ["requestSignature", "requestPasswordFill"],
                "hiddenFromUi": ["answers", "password", "privateKey", "signingKeyBytes", "storyRawText"],
                "localCoreCallChain": ["verify", "authorize", "execute", "revoke"]
            }
        },
        "redactionLevel": "audit_meta_only",
        "retentionGranted": "audit_meta_only",
        "auditMeta": {
            "timestamp": now_timestamp()
        },
        "error": Value::Null
    })
}

fn diagnostics_status(runtime: &WindowsHostRuntime) -> Value {
    let ui = ui_status(runtime);
    json!({
        "status": "success",
        "capability": "windowsHostDiagnostics",
        "executionLocation": "local",
        "result": {
            "generatedAt": now_timestamp(),
            "host": runtime.config.health_json(),
            "ui": ui.get("result").cloned().unwrap_or(Value::Null),
            "localEndpoints": {
                "management": format!("http://127.0.0.1:{}/ui", runtime.config.host_port),
                "diagnostics": format!("http://127.0.0.1:{}/diagnostics", runtime.config.host_port),
                "shutdown": format!("http://127.0.0.1:{}/shutdown", runtime.config.host_port)
            },
            "redaction": {
                "level": "audit_meta_only",
                "hidden": ["answers", "password", "privateKey", "signingKeyBytes", "storyRawText", "sharedSecret"]
            }
        },
        "redactionLevel": "audit_meta_only",
        "retentionGranted": "audit_meta_only",
        "auditMeta": {
            "timestamp": now_timestamp()
        },
        "error": Value::Null
    })
}

fn windows_host_management_ui_html() -> &'static str {
    r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Yian Windows Host</title>
    <style>
      :root { color-scheme: light; --bg:#f4f7f8; --surface:#fff; --line:#d5e0e5; --text:#12202b; --muted:#5c6d79; --accent:#0d6d77; }
      * { box-sizing: border-box; }
      body { margin: 0; background: var(--bg); color: var(--text); font-family: "Segoe UI", "Microsoft YaHei", sans-serif; }
      main { width: min(1180px, calc(100vw - 32px)); margin: 0 auto; padding: 28px 0 44px; }
      header { display: flex; align-items: end; justify-content: space-between; gap: 16px; margin-bottom: 20px; }
      h1, h2, p { margin: 0; }
      h1 { font-size: 28px; line-height: 1.2; }
      .muted { color: var(--muted); line-height: 1.65; }
      .grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; }
      .wide { grid-column: span 2; }
      section { min-width: 0; padding: 18px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); }
      h2 { margin-bottom: 12px; font-size: 18px; }
      dl { display: grid; gap: 10px; margin: 0; }
      dt { color: var(--muted); font-size: 12px; }
      dd { margin: 3px 0 0; overflow-wrap: anywhere; }
      .pill { display: inline-flex; padding: 3px 9px; border: 1px solid var(--line); border-radius: 999px; color: var(--accent); font-size: 12px; }
      .ok { color: #0a6a38; }
      .warn { color: #9b5a00; }
      pre { margin: 0; max-height: 260px; overflow: auto; padding: 12px; border-radius: 8px; background: #101820; color: #d9e6ee; white-space: pre-wrap; overflow-wrap: anywhere; }
      button, a { min-height: 38px; display: inline-flex; align-items: center; justify-content: center; padding: 0 12px; border: 1px solid var(--line); border-radius: 8px; background: #fff; color: var(--text); text-decoration: none; cursor: pointer; }
      .actions { display: flex; flex-wrap: wrap; gap: 10px; }
      @media (max-width: 900px) { header, .grid { grid-template-columns: 1fr; display: grid; } .wide { grid-column: auto; } }
    </style>
  </head>
  <body>
    <main>
      <header>
        <div>
          <p class="muted">Yian Host audit and authorization surface</p>
          <h1>Yian Windows Host Management</h1>
        </div>
        <div class="actions">
          <button type="button" id="refresh">Refresh</button>
          <a href="/health">Health JSON</a>
          <a href="/diagnostics">Diagnostics</a>
        </div>
      </header>
      <div class="grid">
        <section><h2>Host</h2><dl id="host"></dl></section>
        <section><h2>Relay</h2><dl id="relay"></dl></section>
        <section><h2>Authorization Modes</h2><dl id="authorization-modes"></dl></section>
        <section><h2>Managed Objects</h2><dl id="managed-objects"></dl></section>
        <section><h2>Agents</h2><dl id="agents"></dl></section>
        <section><h2>Remote Interfaces</h2><dl id="remote-interfaces"></dl></section>
        <section><h2>Error Calls</h2><dl id="error-calls"></dl></section>
        <section><h2>Story Template Queue</h2><dl id="story-template"></dl></section>
        <section class="wide"><h2>Last Confirmation</h2><dl id="last-confirmation"></dl></section>
        <section><h2>Last Execution</h2><dl id="last-execution"></dl></section>
        <section><h2>Boundaries</h2><dl id="boundaries"></dl></section>
        <section class="wide"><h2>Raw Redacted Status JSON</h2><pre id="raw">loading...</pre></section>
      </div>
    </main>
    <script>
      const fields = (target, rows) => {
        document.querySelector(target).innerHTML = rows.map(([k, v]) => `<div><dt>${k}</dt><dd>${v ?? "not configured"}</dd></div>`).join("");
      };
      const emptyRows = (label) => [[label, "No calls recorded yet"]];
      async function loadStatus() {
        const response = await fetch("/ui/status", { headers: { accept: "application/json" } });
        const payload = await response.json();
        const result = payload.result || {};
        const host = result.host || {};
        const relay = result.relay || {};
        const remote = result.remote || {};
        const management = result.managementStats || {};
        const templateGenerator = result.storyTemplateGenerator || {};
        const confirmation = result.lastConfirmation || {};
        const last = result.lastExecution || {};
        const boundaries = result.boundaries || {};
        fields("#host", [
          ["Product", `${host.product || ""} ${host.version || ""}`],
          ["Status", `<span class="pill">${host.status || "unknown"}</span>`],
          ["Identity", host.identityId],
          ["Device", host.deviceId],
          ["Local API", host.executeUrl],
          ["Storage Visibility", host.storage?.visibility || "host_internal_only"],
        ]);
        fields("#relay", [
          ["Remote", remote.enabled ? `enabled: ${remote.gatewayUrl || ""}` : "local only"],
          ["Status", `<span class="${relay.status === "online" ? "ok" : "warn"}">${relay.status || "unknown"}</span>`],
          ["Last Poll", relay.lastPollAt],
          ["Last Error", relay.lastError || "none"],
        ]);
        fields("#authorization-modes", (management.authorizationModes || []).map(mode => [
          mode.channel,
          `${mode.requiredCells}/${mode.gridSize} grid cells, ${mode.requiredStrength}, ${mode.remoteAllowed ? "remote allowed" : "local only"}`
        ]));
        fields("#managed-objects", (management.objects || []).length ? management.objects.map(item => [
          item.objectRef,
          `${item.calls} calls, ${item.successes} ok, ${item.failures} errors, last ${item.lastSeenAt || "never"}`
        ]) : emptyRows("Managed objects"));
        fields("#agents", (management.agents || []).length ? management.agents.map(item => [item.name, `${item.calls} calls`]) : emptyRows("Agents"));
        fields("#remote-interfaces", (management.remoteInterfaces || []).length ? management.remoteInterfaces.map(item => [item.name, `${item.calls} calls`]) : emptyRows("Remote interfaces"));
        fields("#error-calls", (management.errors || []).length ? management.errors.map(item => [item.name, `${item.calls} calls`]) : [["No errors", "0 calls"]]);
        fields("#story-template", [
          ["Mode", templateGenerator.mode || "local_template_fallback"],
          ["LLM Key", templateGenerator.llmKey || "missing"],
          ["Candidate Count", templateGenerator.candidateCount ?? 0],
          ["Pull Rule", "StoryLock must pull; Host never invokes StoryLock"],
        ]);
        fields("#last-confirmation", [
          ["Request", confirmation.requestId || "none"],
          ["Status", confirmation.status || "none"],
          ["Capability", confirmation.capability || "none"],
          ["Object", confirmation.objectRef || "none"],
          ["Requester", confirmation.requester || "none"],
          ["Origin", confirmation.origin || "none"],
          ["Strength", confirmation.requiredStrength || "none"],
          ["Expiry", confirmation.expiry || "none"],
          ["Risk", confirmation.risk || "none"],
        ]);
        fields("#last-execution", [
          ["Request", last.requestId || "none"],
          ["Status", last.status || "none"],
          ["Capability", last.capability || "none"],
          ["Object", last.objectRef || "none"],
          ["Authorization", last.authorizationId || "none"],
          ["Strength", last.requiredStrength || "none"],
          ["Redaction", last.redactionLevel || "audit_meta_only"],
        ]);
        fields("#boundaries", [
          ["Remote Capabilities", (boundaries.remoteCapabilities || []).join(", ")],
          ["Hidden From UI", (boundaries.hiddenFromUi || []).join(", ")],
          ["Local Call Chain", (boundaries.localCoreCallChain || []).join(" -> ")],
        ]);
        document.querySelector("#raw").textContent = JSON.stringify(payload, null, 2);
      }
      document.querySelector("#refresh").addEventListener("click", loadStatus);
      loadStatus();
      setInterval(loadStatus, 5000);
    </script>
  </body>
</html>"##
}

#[allow(dead_code)]
fn windows_host_ui_html() -> &'static str {
    r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Yian Windows Host</title>
    <style>
      :root { color-scheme: light; --bg:#f4f7f8; --surface:#fff; --line:#d5e0e5; --text:#12202b; --muted:#5c6d79; --accent:#0d6d77; }
      * { box-sizing: border-box; }
      body { margin: 0; background: var(--bg); color: var(--text); font-family: "Segoe UI", "Microsoft YaHei", sans-serif; }
      main { width: min(1120px, calc(100vw - 32px)); margin: 0 auto; padding: 28px 0 44px; }
      header { display: flex; align-items: end; justify-content: space-between; gap: 16px; margin-bottom: 20px; }
      h1, h2, p { margin: 0; }
      h1 { font-size: 28px; line-height: 1.2; }
      .muted { color: var(--muted); line-height: 1.65; }
      .grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; }
      .wide { grid-column: span 2; }
      section { min-width: 0; padding: 18px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); }
      h2 { margin-bottom: 12px; font-size: 18px; }
      dl { display: grid; gap: 10px; margin: 0; }
      dt { color: var(--muted); font-size: 12px; }
      dd { margin: 3px 0 0; overflow-wrap: anywhere; }
      .pill { display: inline-flex; padding: 3px 9px; border: 1px solid var(--line); border-radius: 999px; color: var(--accent); font-size: 12px; }
      .ok { color: #0a6a38; }
      .warn { color: #9b5a00; }
      pre { margin: 0; max-height: 260px; overflow: auto; padding: 12px; border-radius: 8px; background: #101820; color: #d9e6ee; white-space: pre-wrap; overflow-wrap: anywhere; }
      button, a { min-height: 38px; display: inline-flex; align-items: center; justify-content: center; padding: 0 12px; border: 1px solid var(--line); border-radius: 8px; background: #fff; color: var(--text); text-decoration: none; cursor: pointer; }
      .actions { display: flex; flex-wrap: wrap; gap: 10px; }
      @media (max-width: 840px) { header, .grid { grid-template-columns: 1fr; display: grid; } .wide { grid-column: auto; } }
    </style>
  </head>
  <body>
    <main>
      <header>
        <div>
          <p class="muted">StoryLock Local Core</p>
          <h1>Yian Windows Host 本地管理页</h1>
        </div>
        <div class="actions">
          <button type="button" id="refresh">刷新</button>
          <a href="/health">Health JSON</a>
          <a href="/diagnostics">诊断信息</a>
        </div>
      </header>
      <div class="grid">
        <section>
          <h2>宿主状态</h2>
          <dl id="host"></dl>
        </section>
        <section>
          <h2>Relay</h2>
          <dl id="relay"></dl>
        </section>
        <section>
          <h2>题库</h2>
          <dl id="question-bank"></dl>
        </section>
        <section class="wide">
          <h2>最近确认请求</h2>
          <dl id="last-confirmation"></dl>
        </section>
        <section>
          <h2>最近执行摘要</h2>
          <dl id="last-execution"></dl>
        </section>
        <section>
          <h2>能力边界</h2>
          <dl id="boundaries"></dl>
        </section>
        <section class="wide">
          <h2>原始状态 JSON</h2>
          <pre id="raw">loading...</pre>
        </section>
      </div>
    </main>
    <script>
      const fields = (target, rows) => {
        document.querySelector(target).innerHTML = rows.map(([k, v]) => `<div><dt>${k}</dt><dd>${v ?? "未配置"}</dd></div>`).join("");
      };
      async function loadStatus() {
        const response = await fetch("/ui/status", { headers: { accept: "application/json" } });
        const payload = await response.json();
        const result = payload.result || {};
        const host = result.host || {};
        const relay = result.relay || {};
        const remote = result.remote || {};
        const bank = result.questionBank || {};
        const confirmation = result.lastConfirmation || {};
        const last = result.lastExecution || {};
        const boundaries = result.boundaries || {};
        fields("#host", [
          ["产品", `${host.product || ""} ${host.version || ""}`],
          ["状态", `<span class="pill">${host.status || "unknown"}</span>`],
          ["identityId", host.identityId],
          ["deviceId", host.deviceId],
          ["本地 API", host.executeUrl],
          ["Storage", host.storage?.visibility || "host_internal_only"],
        ]);
        fields("#relay", [
          ["Remote", remote.enabled ? `enabled: ${remote.gatewayUrl || ""}` : "local only"],
          ["状态", `<span class="${relay.status === "online" ? "ok" : "warn"}">${relay.status || "unknown"}</span>`],
          ["最近轮询", relay.lastPollAt],
          ["最近错误", relay.lastError || "无"],
        ]);
        fields("#question-bank", [
          ["版本", bank.questionSetVersion],
          ["规范化", bank.normalizationVersion],
          ["题目数量", bank.questionCount],
          ["Visibility", bank.visibility || "host_internal_only"],
        ]);
        fields("#last-confirmation", [
          ["请求", confirmation.requestId || "暂无"],
          ["状态", confirmation.status || "暂无"],
          ["能力", confirmation.capability || "暂无"],
          ["对象", confirmation.objectRef || "暂无"],
          ["请求方", confirmation.requester || "暂无"],
          ["来源", confirmation.origin || "暂无"],
          ["强度", confirmation.requiredStrength || "暂无"],
          ["过期", confirmation.expiry || "暂无"],
          ["风险", confirmation.risk || "暂无"],
        ]);
        fields("#last-execution", [
          ["请求", last.requestId || "暂无"],
          ["状态", last.status || "暂无"],
          ["能力", last.capability || "暂无"],
          ["对象", last.objectRef || "暂无"],
          ["授权", last.authorizationId || "暂无"],
          ["强度", last.requiredStrength || "暂无"],
          ["脱敏等级", last.redactionLevel || "audit_meta_only"],
        ]);
        fields("#boundaries", [
          ["远程能力", (boundaries.remoteCapabilities || []).join(", ")],
          ["UI 隐藏字段", (boundaries.hiddenFromUi || []).join(", ")],
          ["本地调用链", (boundaries.localCoreCallChain || []).join(" -> ")],
        ]);
        document.querySelector("#raw").textContent = JSON.stringify(payload, null, 2);
      }
      document.querySelector("#refresh").addEventListener("click", loadStatus);
      loadStatus();
      setInterval(loadStatus, 5000);
    </script>
  </body>
</html>"##
}

fn question_bank_import(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    let source_path = match request.get("sourcePath").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => value.trim(),
        _ => {
            return error_response(
                &runtime.config,
                &request_id,
                "questionBankImport",
                "SLG-001",
                "validation_error",
                "sourcePath is required",
                "Provide the question bank JSON file path to import.",
            )
        }
    };
    match import_question_bank(&runtime.config.data_dir, Path::new(source_path)).and_then(|bank| {
        runtime.replace_question_bank(bank.clone())?;
        Ok(bank)
    }) {
        Ok(bank) => json!({
            "requestId": request_id,
            "status": "success",
            "capability": "questionBankImport",
            "executionLocation": "local",
            "result": {
                "path": question_bank_path(&runtime.config.data_dir).display().to_string(),
                "questionSetVersion": bank.question_set_version,
                "normalizationVersion": bank.normalization_version,
                "questionCount": bank.questions.len()
            },
            "redactionLevel": "none",
            "retentionGranted": "audit_meta_only",
            "auditMeta": {
                "timestamp": now_timestamp()
            },
            "error": Value::Null
        }),
        Err(error) => error_response(
            &runtime.config,
            &request_id,
            "questionBankImport",
            "SLG-005",
            "host_storage_error",
            &format!("failed to import question bank: {error}"),
            "Validate the source JSON file and retry the question bank import request.",
        ),
    }
}

fn story_template_interface_manifest() -> Value {
    json!({
        "schemaVersion": "story-template-interface-manifest-v1",
        "owner": "yian-windows-host",
        "direction": "storylock_pulls_candidates",
        "interfaces": {
            "candidateQueue": "story-template-candidates.jsonl",
            "localHttpGenerate": "/story-template/generate",
            "localHttpCandidates": "/story-template/candidates"
        },
        "boundary": {
            "hostMayGenerateCandidates": true,
            "hostMustNotInvokeStoryLock": true,
            "storyLockImportsOnlyAfterExplicitPull": true,
            "llmKeysAreDirectAccessConfig": true
        }
    })
}

fn story_template_generator_status(runtime: &WindowsHostRuntime) -> Value {
    let key_configured = std::env::var("STORYLOCK_STORY_LLM_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let endpoint_configured = std::env::var("STORYLOCK_STORY_LLM_ENDPOINT")
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    json!({
        "schemaVersion": "story-template-generator-status-v1",
        "mode": if key_configured && endpoint_configured { "llm_ready" } else { "local_template_fallback" },
        "llmKey": if key_configured { "configured_direct_access" } else { "missing" },
        "llmEndpoint": if endpoint_configured { "configured" } else { "missing" },
        "candidateCount": runtime.secret_store.read_story_template_candidates(1000).len(),
        "interfaces": {
            "generate": format!("http://127.0.0.1:{}/story-template/generate", runtime.config.host_port),
            "candidates": format!("http://127.0.0.1:{}/story-template/candidates", runtime.config.host_port)
        },
        "boundary": "Host generates and queues candidates only. StoryLock must pull; Host never invokes StoryLock."
    })
}

fn safe_story_slug(value: &str) -> String {
    let slug = sanitize_ref(value);
    if slug.is_empty() {
        short_id()
    } else {
        slug
    }
}

fn generate_local_story_framework(request: &Value) -> Value {
    let theme = request
        .get("theme")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("memory authorization story");
    let audience = request
        .get("audience")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("local StoryLock user");
    let tone = request
        .get("tone")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("clear and memorable");
    let slug = safe_story_slug(theme);
    json!({
        "title": format!("Story Framework: {theme}"),
        "summary": format!("A {tone} story framework for {audience}, prepared as a candidate template for StoryLock to pull later."),
        "memoryAnchors": [
            format!("{theme} origin"),
            "trusted local device",
            "nine-grid recall",
            "agent request",
            "remote interface check",
            "safe completion"
        ],
        "chapters": [
            { "id": format!("{slug}-setup"), "purpose": "Introduce the person, place, and object that make the memory easy to recall." },
            { "id": format!("{slug}-challenge"), "purpose": "Connect the managed object request to a concrete event and choice." },
            { "id": format!("{slug}-resolution"), "purpose": "Close with the correct authorization boundary and outcome." }
        ],
        "questionPlan": {
            "targetCount": request.get("questionCount").and_then(Value::as_u64).unwrap_or(24),
            "grid": "StoryLock decides final grid cells and answers after import.",
            "hostBoundary": "candidate_framework_only"
        }
    })
}

fn story_template_generate(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    let framework = generate_local_story_framework(request);
    let candidate_id = format!("story-template-{}", Uuid::new_v4());
    let candidate = json!({
        "schemaVersion": "story-template-candidate-v1",
        "candidateId": candidate_id,
        "requestId": request_id,
        "createdAt": now_timestamp(),
        "generator": {
            "owner": "yian-host",
            "mode": story_template_generator_status(runtime)
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("local_template_fallback"),
            "llmKey": story_template_generator_status(runtime)
                .get("llmKey")
                .and_then(Value::as_str)
                .unwrap_or("missing")
        },
        "framework": framework,
        "consumption": {
            "direction": "storylock_pulls_candidate",
            "hostInvokesStoryLock": false,
            "status": "queued"
        },
        "redactionLevel": "candidate_only"
    });
    match runtime.secret_store.append_story_template_candidate(&candidate) {
        Ok(()) => json!({
            "requestId": request_id,
            "status": "success",
            "capability": "generateStoryTemplateCandidate",
            "executionLocation": "local",
            "result": {
                "candidateId": candidate.get("candidateId").cloned().unwrap_or(Value::Null),
                "queued": true,
                "storyLockPullRequired": true,
                "framework": candidate.get("framework").cloned().unwrap_or(Value::Null)
            },
            "redactionLevel": "candidate_only",
            "retentionGranted": "candidate_queue_only",
            "auditMeta": {
                "timestamp": now_timestamp(),
                "interface": "story-template/generate",
                "llmKey": candidate
                    .get("generator")
                    .and_then(|value| value.get("llmKey"))
                    .cloned()
                    .unwrap_or(Value::String("missing".to_string()))
            },
            "error": Value::Null
        }),
        Err(error) => error_response(
            &runtime.config,
            &request_id,
            "generateStoryTemplateCandidate",
            "SLG-005",
            "host_storage_error",
            &format!("failed to queue story template candidate: {error}"),
            "Check the Windows host data directory and retry template generation.",
        ),
    }
}

fn story_template_candidates(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    let limit = request
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 100) as usize;
    json!({
        "requestId": request_id,
        "status": "success",
        "capability": "storyTemplateCandidates",
        "executionLocation": "local",
        "result": {
            "pullModel": "storylock_explicit_pull_only",
            "hostInvokesStoryLock": false,
            "interfaceManifest": story_template_interface_manifest(),
            "candidates": runtime.secret_store.read_story_template_candidates(limit)
        },
        "redactionLevel": "candidate_only",
        "retentionGranted": "candidate_queue_only",
        "auditMeta": {
            "timestamp": now_timestamp(),
            "interface": "story-template/candidates"
        },
        "error": Value::Null
    })
}

fn authorize_local_action(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let config = &runtime.config;
    let request_id = request_id_from(request);
    let verification_id = match request.get("verificationId").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => value.trim(),
        _ => {
            return error_response(
                config,
                &request_id,
                "authorizeLocalAction",
                "SLG-001",
                "validation_error",
                "verificationId is required",
                "Call /verify first and reuse the returned verificationId.",
            )
        }
    };
    let verification = match runtime
        .secret_store
        .read_verification_record(verification_id)
    {
        Ok(record) => record,
        Err(error) => {
            return error_response(
                config,
                &request_id,
                "authorizeLocalAction",
                "SLG-003",
                "authorization_failed",
                &format!("verification record was not found: {error}"),
                "Create a new verification challenge and try again.",
            )
        }
    };
    if !is_unexpired(&verification.expires_at) {
        record_local_audit(
            runtime,
            "challenge_failed",
            &request_id,
            "authorizeLocalAction",
            Some(&verification.object_ref),
            "failed",
            Some("SLG-003"),
            Some("authorization_failed"),
            json!({
                "reason": "verification_expired",
                "verificationId": verification.verification_id
            }),
        );
        return error_response(
            config,
            &request_id,
            "authorizeLocalAction",
            "SLG-003",
            "authorization_failed",
            "verification challenge expired",
            "Create a new verification challenge and try again.",
        );
    }
    let answers = request
        .get("answers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let matched_cells = verification
        .cells
        .iter()
        .filter(|cell| {
            answers.iter().any(|item| {
                let cell_id = item.get("cellId").and_then(Value::as_str).unwrap_or("");
                let answer = item
                    .get("answer")
                    .and_then(Value::as_str)
                    .or_else(|| item.as_str())
                    .unwrap_or("");
                cell_id == cell.cell_id
                    && normalize_answer(answer, &cell.normalization_version)
                        == normalize_answer(&cell.expected_answer, &cell.normalization_version)
            })
        })
        .count() as u32;
    if matched_cells < verification.required_cells {
        record_local_audit(
            runtime,
            "challenge_failed",
            &request_id,
            "authorizeLocalAction",
            Some(&verification.object_ref),
            "failed",
            Some("SLG-003"),
            Some("authorization_failed"),
            json!({
                "reason": "answer_mismatch",
                "verificationId": verification.verification_id,
                "matchedCells": matched_cells,
                "requiredCells": verification.required_cells
            }),
        );
        return error_response(
            config,
            &request_id,
            "authorizeLocalAction",
            "SLG-003",
            "authorization_failed",
            "verification answers did not satisfy the required challenge cells",
            "Respond with the exact answer for each required cell returned by the verification challenge.",
        );
    }
    let authorization = StoredAuthorizationRecord {
        verification_id: verification.verification_id.clone(),
        authorization_id: format!("ses-{}", Uuid::new_v4()),
        capability: verification.capability.clone(),
        object_ref: verification.object_ref.clone(),
        identity_id: verification.identity_id.clone(),
        allowed_action: verification.allowed_action.clone(),
        required_strength: verification.required_strength.clone(),
        confirmation_method: "challenge_answer".to_string(),
        created_at: now_timestamp(),
        expires_at: expires_at_after(300),
        status: "approved".to_string(),
    };
    if let Err(error) = runtime
        .secret_store
        .write_authorization_record(&authorization)
    {
        return error_response(
            config,
            &request_id,
            "authorizeLocalAction",
            "SLG-005",
            "host_storage_error",
            &format!("failed to persist authorization record: {error}"),
            "Check the Windows host data directory and DPAPI availability, then retry the request.",
        );
    }
    record_local_audit(
        runtime,
        "authorization_approved",
        &request_id,
        "authorizeLocalAction",
        Some(&authorization.object_ref),
        "success",
        None,
        None,
        json!({
            "verificationId": authorization.verification_id,
            "authorizationId": authorization.authorization_id,
            "allowedAction": authorization.allowed_action,
            "requiredStrength": authorization.required_strength
        }),
    );
    json!({
        "requestId": request_id,
        "status": "success",
        "capability": "authorizeLocalAction",
        "executionLocation": "local",
        "result": {
            "approved": true,
            "authorizationId": authorization.authorization_id,
            "identityId": authorization.identity_id,
            "objectRef": authorization.object_ref,
            "allowedAction": authorization.allowed_action,
            "expiresAt": authorization.expires_at
        },
        "redactionLevel": "none",
        "retentionGranted": "audit_meta_only",
        "auditMeta": {
            "timestamp": authorization.created_at,
            "verificationId": authorization.verification_id,
            "authorizationId": authorization.authorization_id
        },
        "error": Value::Null
    })
}

fn revoke_local_authorization(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let config = &runtime.config;
    let request_id = request_id_from(request);
    let authorization_id = request
        .get("authorizationId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if authorization_id.is_empty() {
        return error_response(
            config,
            &request_id,
            "revokeLocalAuthorization",
            "SLG-001",
            "validation_error",
            "authorizationId is required",
            "Provide the authorizationId that should be revoked.",
        );
    }
    let mut record = match runtime
        .secret_store
        .read_authorization_record(&authorization_id)
    {
        Ok(record) => record,
        Err(error) => {
            record_local_audit(
                runtime,
                "session_revoke_rejected",
                &request_id,
                "revokeLocalAuthorization",
                None,
                "error",
                Some("SLG-003"),
                Some("authorization_failed"),
                json!({
                    "authorizationId": authorization_id,
                    "reason": format!("authorization record was not found: {error}")
                }),
            );
            return error_response(
                config,
                &request_id,
                "revokeLocalAuthorization",
                "SLG-003",
                "authorization_failed",
                &format!("authorization record was not found: {error}"),
                "Create a new authorization session if the old one is no longer available.",
            );
        }
    };
    record.status = "revoked".to_string();
    record.expires_at = now_timestamp();
    if let Err(error) = runtime.secret_store.write_authorization_record(&record) {
        return error_response(
            config,
            &request_id,
            "revokeLocalAuthorization",
            "SLG-005",
            "host_storage_error",
            &format!("failed to persist revoked authorization record: {error}"),
            "Check the Windows host data directory and DPAPI availability, then retry the request.",
        );
    }
    record_local_audit(
        runtime,
        "session_revoked",
        &request_id,
        "revokeLocalAuthorization",
        Some(&record.object_ref),
        "success",
        None,
        None,
        json!({
            "authorizationId": record.authorization_id,
            "verificationId": record.verification_id,
            "allowedAction": record.allowed_action
        }),
    );
    json!({
        "requestId": request_id,
        "status": "success",
        "capability": "revokeLocalAuthorization",
        "executionLocation": "local",
        "result": {
            "identityId": record.identity_id,
            "authorizationId": record.authorization_id,
            "targetType": "authorization_session",
            "status": record.status
        },
        "redactionLevel": "none",
        "retentionGranted": "audit_meta_only",
        "auditMeta": {
            "timestamp": now_timestamp(),
            "targetType": "authorization_session"
        },
        "error": Value::Null
    })
}

fn execute_with_local_core(
    runtime: &WindowsHostRuntime,
    request_id: &str,
    capability: &str,
    object_ref: &str,
    request: &Value,
    authorization: &StoredAuthorizationRecord,
) -> Result<Value> {
    validate_authorization_for_core(authorization, capability, object_ref)?;
    let core_call = local_core_call_envelope(&runtime.config, request_id, authorization);
    let core_call_id = core_call
        .get("coreCallId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if authorization.allowed_action == "story_edit" {
        Ok(json!({
            "approved": true,
            "coreCallId": core_call_id,
            "coreBoundary": "storylock_local_core",
            "verificationId": authorization.verification_id,
            "authorizationId": authorization.authorization_id,
            "objectRef": object_ref,
            "storyEditAuthorized": true,
            "editableScope": "local_storylock_core_only",
            "requiredStrength": authorization.required_strength,
            "allowedAction": authorization.allowed_action,
            "expiresAt": authorization.expires_at,
            "hiddenFromResult": ["storyRawText", "canonicalAnswer", "acceptedAnswers", "password", "privateKey", "signingKeyBytes"],
            "localCore": core_call
        }))
    } else if authorization.allowed_action == "batch_read" {
        Ok(json!({
            "approved": true,
            "coreCallId": core_call_id,
            "coreBoundary": "storylock_local_core",
            "verificationId": authorization.verification_id,
            "authorizationId": authorization.authorization_id,
            "objectRef": object_ref,
            "batchReadAuthorized": true,
            "readScope": "permission_summary_only",
            "requiredStrength": authorization.required_strength,
            "allowedAction": authorization.allowed_action,
            "expiresAt": authorization.expires_at,
            "items": [{
                "objectRef": object_ref,
                "status": "authorized",
                "redaction": "secret_values_hidden"
            }],
            "hiddenFromResult": ["storyRawText", "canonicalAnswer", "acceptedAnswers", "password", "privateKey", "signingKeyBytes"],
            "localCore": core_call
        }))
    } else if capability == "requestSignature" {
        let key_material = runtime
            .secret_store
            .get_or_create_signature_key(object_ref)?;
        let signature = signature_of_request(&key_material, request)?;
        Ok(json!({
            "approved": true,
            "coreCallId": core_call_id,
            "coreBoundary": "storylock_local_core",
            "verificationId": authorization.verification_id,
            "authorizationId": authorization.authorization_id,
            "signature": signature,
            "keyId": object_ref,
            "algorithm": "sha256-hmac-prototype",
            "requiredStrength": authorization.required_strength,
            "allowedAction": authorization.allowed_action,
            "expiresAt": authorization.expires_at,
            "privateKey": "windows-dpapi-local-only",
            "signingKeyBytes": "[windows-dpapi-protected]",
            "localCore": core_call
        }))
    } else {
        let credential = runtime.secret_store.get_or_create_credential(
            object_ref,
            request.get("usernameHint").and_then(Value::as_str),
            request.get("targetOrigin").and_then(Value::as_str),
        )?;
        Ok(json!({
            "approved": true,
            "coreCallId": core_call_id,
            "coreBoundary": "storylock_local_core",
            "verificationId": authorization.verification_id,
            "authorizationId": authorization.authorization_id,
            "credentialRef": object_ref,
            "username": credential.username,
            "password": credential.password,
            "targetOrigin": credential.target_origin,
            "requiredStrength": authorization.required_strength,
            "allowedAction": authorization.allowed_action,
            "expiresAt": authorization.expires_at,
            "localCore": core_call
        }))
    }
}

fn execute_request(runtime: &WindowsHostRuntime, request: Value) -> Value {
    let config = &runtime.config;
    let request_id = request_id_from(&request);
    let capability = capability_from(&request);
    let supported = matches!(
        capability.as_str(),
        "requestSignature" | "requestPasswordFill"
    );
    if !supported {
        return error_response(
            config,
            &request_id,
            &capability,
            "SLG-001",
            "validation_error",
            "unsupported capability",
            "Use requestSignature or requestPasswordFill.",
        );
    }

    let object_ref = object_ref_for_request(&capability, &request);
    let audit_context = request_audit_context(&request);

    let resolved_authorization = if let Some(authorization_id) = request
        .get("authorizationId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        match runtime
            .secret_store
            .read_authorization_record(authorization_id)
        {
            Ok(record) => {
                if let Err(error) =
                    validate_authorization_for_core(&record, &capability, &object_ref)
                {
                    record_local_audit(
                        runtime,
                        "execution_rejected",
                        &request_id,
                        &capability,
                        Some(&object_ref),
                        "error",
                        Some("SLG-003"),
                        Some("authorization_failed"),
                        merge_audit_meta(json!({
                            "reason": error.to_string(),
                            "authorizationId": authorization_id,
                            "authorizationStatus": record.status,
                            "allowedAction": record.allowed_action
                        }), audit_context.clone()),
                    );
                    return error_response(
                        config,
                        &request_id,
                        &capability,
                        "SLG-003",
                        "authorization_failed",
                        &format!("authorizationId is invalid for this request: {error}"),
                        "Create a fresh verification and authorization session, then retry the execute call.",
                    );
                }
                record
            }
            Err(error) => {
                record_local_audit(
                    runtime,
                    "execution_rejected",
                    &request_id,
                    &capability,
                    Some(&object_ref),
                    "error",
                    Some("SLG-003"),
                    Some("authorization_failed"),
                    merge_audit_meta(json!({
                        "reason": format!("authorizationId could not be resolved: {error}"),
                        "authorizationId": authorization_id
                    }), audit_context.clone()),
                );
                return error_response(
                    config,
                    &request_id,
                    &capability,
                    "SLG-003",
                    "authorization_failed",
                    &format!("authorizationId could not be resolved: {error}"),
                    "Create a fresh verification and authorization session, then retry the execute call.",
                );
            }
        }
    } else {
        let policy = match channel_policy_for_request(&capability, &request) {
            Ok(policy) => policy,
            Err(error) => {
                record_local_audit(
                    runtime,
                    "policy_denied",
                    &request_id,
                    &capability,
                    Some(&object_ref),
                    "denied",
                    Some("SLG-002"),
                    Some("policy_denied"),
                    merge_audit_meta(json!({
                        "reason": error.to_string(),
                        "authorizationChannel": authorization_channel_for_request(&capability, &request)
                    }), audit_context.clone()),
                );
                return error_response(
                    config,
                    &request_id,
                    &capability,
                    "SLG-002",
                    "policy_denied",
                    &error.to_string(),
                    "Use a supported local authorization channel. Remote requests cannot trigger story_edit.",
                );
            }
        };
        if !is_confirmation_approved(runtime, &request, &object_ref) {
            record_local_audit(
                runtime,
                "authorization_denied",
                &request_id,
                &capability,
                Some(&object_ref),
                "denied",
                Some("SLG-003"),
                Some("authorization_failed"),
                merge_audit_meta(json!({
                    "reason": "local_confirmation_denied",
                    "approvalMode": config.approval_mode
                }), audit_context.clone()),
            );
            return error_response(
                config,
                &request_id,
                &capability,
                "SLG-003",
                "authorization_failed",
                "local confirmation denied from Windows host dialog",
                "Review the request details in the Windows confirmation dialog and choose Yes to approve.",
            );
        }

        let authorization_record = StoredAuthorizationRecord {
            verification_id: format!("ver-{}", Uuid::new_v4()),
            authorization_id: format!("ses-{}", Uuid::new_v4()),
            capability: capability.clone(),
            object_ref: object_ref.clone(),
            identity_id: config.identity_id.clone(),
            allowed_action: policy.allowed_action.to_string(),
            required_strength: policy.required_strength.to_string(),
            confirmation_method: "windows_dialog".to_string(),
            created_at: now_timestamp(),
            expires_at: expires_at_after(300),
            status: "approved".to_string(),
        };
        if let Err(error) = runtime
            .secret_store
            .write_authorization_record(&authorization_record)
        {
            return error_response(
                config,
                &request_id,
                &capability,
                "SLG-005",
                "host_storage_error",
                &format!("windows host failed to persist authorization record: {error}"),
                "Check the Windows host data directory and DPAPI availability, then retry the request.",
            );
        }
        authorization_record
    };

    let verification_id = resolved_authorization.verification_id.clone();
    let authorization_id = resolved_authorization.authorization_id.clone();
    let required_strength = resolved_authorization.required_strength.clone();
    let allowed_action = resolved_authorization.allowed_action.clone();
    let confirmation_method = resolved_authorization.confirmation_method.clone();

    let execution = execute_with_local_core(
        runtime,
        &request_id,
        &capability,
        &object_ref,
        &request,
        &resolved_authorization,
    );

    match execution {
        Ok(result) => {
            record_local_audit(
                runtime,
                "execution_completed",
                &request_id,
                &capability,
                Some(&object_ref),
                "success",
                None,
                None,
                merge_audit_meta(json!({
                    "verificationId": verification_id,
                    "authorizationId": authorization_id,
                    "allowedAction": allowed_action,
                    "requiredStrength": required_strength,
                    "authorizationChannel": authorization_channel_for_request(&capability, &request),
                    "confirmationMethod": confirmation_method
                }), audit_context.clone()),
            );
            json!({
            "requestId": request_id,
            "status": "success",
            "capability": capability,
            "executionLocation": "local",
            "result": result,
            "redactionLevel": "result_only",
            "retentionGranted": if capability == "requestSignature" { "result_only" } else { "audit_meta_only" },
            "auditMeta": {
                "timestamp": now_timestamp(),
                "localHost": "windows-rust-local-core",
                "identityId": config.identity_id,
                "deviceId": config.device_id,
                "objectRef": object_ref,
                "approvalMode": config.approval_mode,
                "storageProvider": "dpapi",
                "coreBoundary": "storylock_local_core",
                "verificationId": verification_id,
                "authorizationId": authorization_id,
                "requiredStrength": required_strength,
                "allowedAction": allowed_action,
                "confirmationMethod": confirmation_method
            },
            "error": Value::Null
            })
        }
        Err(error) => {
            let error_type = if error.to_string().contains("authorization") {
                "authorization_failed"
            } else {
                "host_storage_error"
            };
            let code = if error_type == "authorization_failed" {
                "SLG-003"
            } else {
                "SLG-005"
            };
            record_local_audit(
                runtime,
                "execution_rejected",
                &request_id,
                &capability,
                Some(&object_ref),
                "error",
                Some(code),
                Some(error_type),
                merge_audit_meta(json!({
                    "reason": error.to_string(),
                    "verificationId": verification_id,
                    "authorizationId": authorization_id,
                    "allowedAction": allowed_action,
                    "authorizationChannel": authorization_channel_for_request(&capability, &request)
                }), audit_context.clone()),
            );
            error_response(
                config,
                &request_id,
                &capability,
                code,
                error_type,
                &format!("StoryLock Local Core execution failed: {error}"),
                "Check the local authorization session, protected storage, and DPAPI availability, then retry the request.",
            )
        }
    }
}

fn start_local_server(runtime: WindowsHostRuntime) -> Result<thread::JoinHandle<()>> {
    let address = format!("127.0.0.1:{}", runtime.config.host_port);
    let server = Server::http(&address)
        .map_err(|error| anyhow!("failed to bind local server on {address}: {error}"))?;
    println!("local server listening on http://{address}");

    Ok(thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let path = request.url().split('?').next().unwrap_or("/");
            let response = match (request.method(), path) {
                (&Method::Get, "/health") => {
                    Response::from_string(runtime.config.health_json().to_string())
                        .with_header(content_type_json())
                }
                (&Method::Get, "/question-bank/status") => Response::from_string(
                    question_bank_status(
                        &runtime,
                        &json!({
                            "requestId": format!("req-{}", Uuid::new_v4())
                        }),
                    )
                    .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Get, "/ui") => {
                    Response::from_string(windows_host_management_ui_html())
                        .with_header(content_type_html())
                }
                (&Method::Get, "/ui/status") => {
                    Response::from_string(ui_status(&runtime).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Get, "/diagnostics") => {
                    Response::from_string(diagnostics_status(&runtime).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Get, "/story-template/candidates") => Response::from_string(
                    story_template_candidates(
                        &runtime,
                        &json!({
                            "requestId": format!("req-{}", Uuid::new_v4())
                        }),
                    )
                    .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/shutdown") => {
                    thread::spawn(|| {
                        thread::sleep(Duration::from_millis(150));
                        std::process::exit(0);
                    });
                    Response::from_string(
                        json!({
                            "status": "success",
                            "capability": "windowsHostShutdown",
                            "message": "shutdown scheduled"
                        })
                        .to_string(),
                    )
                    .with_header(content_type_json())
                }
                (&Method::Post, "/verify") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    Response::from_string(create_grid_verification(&runtime, &payload).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Post, "/authorize") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    Response::from_string(authorize_local_action(&runtime, &payload).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Post, "/revoke") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    Response::from_string(
                        revoke_local_authorization(&runtime, &payload).to_string(),
                    )
                    .with_header(content_type_json())
                }
                (&Method::Post, "/question-bank/import") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    Response::from_string(question_bank_import(&runtime, &payload).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Post, "/story-template/generate") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    Response::from_string(story_template_generate(&runtime, &payload).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Post, "/execute") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    let execution = execute_request(&runtime, payload);
                    runtime.record_execution_summary(&execution);
                    Response::from_string(execution.to_string()).with_header(content_type_json())
                }
                _ => Response::from_string("{\"status\":\"error\",\"message\":\"not found\"}")
                    .with_status_code(StatusCode(404))
                    .with_header(content_type_json()),
            };
            let _ = request.respond(response);
        }
    }))
}

fn post_json(client: &Client, config: &WindowsHostConfig, url: &str, body: Value) -> Result<Value> {
    let mut request = client.post(url).json(&body);
    if !config.shared_secret.is_empty() {
        request = request.header("x-storylock-shared-secret", &config.shared_secret);
    }
    let response = request
        .send()
        .with_context(|| format!("request failed: {url}"))?;
    let status = response.status();
    let value = response.json::<Value>().unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        anyhow::bail!("gateway returned {status}: {value}");
    }
    Ok(value)
}

fn register_host(client: &Client, config: &WindowsHostConfig) -> Result<RegistrationResponse> {
    let payload = json!({
        "identityId": config.identity_id,
        "deviceId": config.device_id,
        "appInstanceId": config.app_instance_id,
        "preferredMode": config.preferred_mode,
        "host": {
            "healthUrl": config.health_url,
            "executeUrl": config.execute_url
        },
        "install": {
            "versionName": config.version,
            "versionCode": 1,
            "packageKind": "windows-rust-prototype"
        },
        "device": {
            "platform": "windows",
            "implementation": "rust",
            "computerName": env_or("COMPUTERNAME", "unknown")
        },
        "reachability": {
            "localHttp": true,
            "relayPolling": true,
            "healthStatus": config.health_json()
        }
    });
    let value = post_json(
        client,
        config,
        &config.gateway_url(&config.register_path),
        payload,
    )?;
    Ok(serde_json::from_value(value)?)
}

fn run_relay_loop(runtime: WindowsHostRuntime) -> Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(25)).build()?;
    let mut poll_url = runtime.config.gateway_url(&runtime.config.relay_poll_path);
    let mut respond_url = runtime
        .config
        .gateway_url(&runtime.config.relay_respond_path);

    loop {
        match register_host(&client, &runtime.config) {
            Ok(registration) => {
                if let Some(relay) = registration.relay {
                    if let Some(next_poll_url) = relay.poll_url {
                        poll_url = next_poll_url;
                    }
                    if let Some(next_respond_url) = relay.respond_url {
                        respond_url = next_respond_url;
                    }
                }
                println!("registered; polling relay at {poll_url}");
                runtime.set_relay_status("online", None);
                break;
            }
            Err(error) => {
                eprintln!("registration failed: {error}");
                runtime.set_relay_status("registration_error", Some(error.to_string()));
                thread::sleep(Duration::from_secs(3));
            }
        }
    }

    loop {
        let poll = post_json(
            &client,
            &runtime.config,
            &poll_url,
            json!({
                "deviceId": runtime.config.device_id,
                "appInstanceId": runtime.config.app_instance_id
            }),
        );
        match poll {
            Ok(value) if value.get("status").and_then(Value::as_str) == Some("ok") => {
                let relay_request_id = value
                    .get("relayRequestId")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let mut remote_request = value.get("request").cloned().unwrap_or_else(|| json!({}));
                if let Some(request) = remote_request.as_object_mut() {
                    request.insert("remoteRequest".to_string(), Value::Bool(true));
                    request.insert(
                        "remoteInterface".to_string(),
                        Value::String("relay_gateway".to_string()),
                    );
                    request
                        .entry("requester".to_string())
                        .or_insert_with(|| Value::String("relay_gateway".to_string()));
                }
                let response = execute_request(&runtime, remote_request);
                runtime.record_execution_summary(&response);
                runtime.set_relay_status("handled_request", None);
                let _ = post_json(
                    &client,
                    &runtime.config,
                    &respond_url,
                    json!({
                        "relayRequestId": relay_request_id,
                        "response": response
                    }),
                );
            }
            Ok(_) => {
                runtime.set_relay_status("idle", None);
                thread::sleep(Duration::from_millis(750));
            }
            Err(error) => {
                eprintln!("relay poll failed: {error}");
                runtime.set_relay_status("poll_error", Some(error.to_string()));
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

fn dpapi_protect_to_base64(plain_text: &[u8]) -> Result<String> {
    if plain_text.is_empty() {
        return Ok(String::new());
    }

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: plain_text.len() as u32,
        pbData: plain_text.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let success = unsafe {
        CryptProtectData(
            &mut in_blob,
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
    };
    if success == 0 {
        return Err(anyhow!("CryptProtectData failed"));
    }

    let bytes = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize) };
    let encoded = BASE64.encode(bytes);
    unsafe {
        LocalFree(out_blob.pbData as _);
    }
    Ok(encoded)
}

fn dpapi_unprotect_from_base64(cipher_text: &str) -> Result<Vec<u8>> {
    if cipher_text.is_empty() {
        return Ok(Vec::new());
    }
    let mut cipher_bytes = BASE64.decode(cipher_text)?;
    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: cipher_bytes.len() as u32,
        pbData: cipher_bytes.as_mut_ptr(),
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let success = unsafe {
        CryptUnprotectData(
            &mut in_blob,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
    };
    if success == 0 {
        return Err(anyhow!("CryptUnprotectData failed"));
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec() };
    unsafe {
        LocalFree(out_blob.pbData as _);
    }
    Ok(bytes)
}

fn main() -> Result<()> {
    let config = WindowsHostConfig::from_env();
    let args: Vec<String> = std::env::args().collect();
    let start_mode = std::env::var("STORYLOCK_WINDOWS_START_MODE")
        .unwrap_or_default()
        .to_ascii_lowercase();

    if args.iter().any(|arg| arg == "--slint-ui") {
        return run_slint_ui_entry(config);
    }
    if args.iter().any(|arg| arg == "--print-config") {
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--print-question-bank-path") {
        println!("{}", question_bank_path(&config.data_dir).display());
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--validate-question-bank") {
        let bank = load_or_init_question_bank(&config.data_dir)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "path": question_bank_path(&config.data_dir).display().to_string(),
                "questionSetVersion": bank.question_set_version,
                "normalizationVersion": bank.normalization_version,
                "questionCount": bank.questions.len()
            }))?
        );
        return Ok(());
    }
    if let Some(index) = args.iter().position(|arg| arg == "--import-question-bank") {
        let source = args
            .get(index + 1)
            .ok_or_else(|| anyhow!("--import-question-bank requires a source file path"))?;
        let imported = import_question_bank(&config.data_dir, Path::new(source))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "path": question_bank_path(&config.data_dir).display().to_string(),
                "questionSetVersion": imported.question_set_version,
                "normalizationVersion": imported.normalization_version,
                "questionCount": imported.questions.len()
            }))?
        );
        return Ok(());
    }
    if matches!(start_mode.as_str(), "console" | "debug") {
        eprintln!(
            "STORYLOCK_WINDOWS_START_MODE={start_mode} is ignored by the default Windows Slint UI build."
        );
    }

    run_default_entry(config)
}

#[cfg(not(feature = "ui-slint"))]
fn run_console_entry(config: WindowsHostConfig) -> Result<()> {
    let runtime = WindowsHostRuntime::new(config)?;
    println!("{}", serde_json::to_string_pretty(&runtime.config)?);
    let server_runtime = runtime.clone();
    let _server = start_local_server(server_runtime)?;
    run_runtime_loop(runtime)
}

#[cfg(feature = "ui-slint")]
fn run_default_entry(config: WindowsHostConfig) -> Result<()> {
    run_desktop_ui_entry(config)
}

#[cfg(not(feature = "ui-slint"))]
fn run_default_entry(config: WindowsHostConfig) -> Result<()> {
    run_console_entry(config)
}

#[cfg(feature = "ui-slint")]
fn run_slint_ui_entry(config: WindowsHostConfig) -> Result<()> {
    run_desktop_ui_entry(config)
}

#[cfg(not(feature = "ui-slint"))]
fn run_slint_ui_entry(_config: WindowsHostConfig) -> Result<()> {
    Err(anyhow!(
        "Slint UI is not enabled. Run with: cargo run --features ui-slint -- --slint-ui"
    ))
}

#[cfg(feature = "ui-slint")]
fn run_desktop_ui_entry(config: WindowsHostConfig) -> Result<()> {
    let runtime = WindowsHostRuntime::new(config.clone())?;
    let server_runtime = runtime.clone();
    let _server = match start_local_server(server_runtime) {
        Ok(server) => server,
        Err(error) if error.to_string().contains("failed to bind local server") => {
            return slint_ui::run(config);
        }
        Err(error) => return Err(error),
    };
    thread::spawn(move || run_runtime_loop(runtime));
    slint_ui::run(config)
}

fn run_runtime_loop(runtime: WindowsHostRuntime) -> Result<()> {
    if runtime.config.remote_enabled {
        run_relay_loop(runtime)
    } else {
        runtime.set_relay_status("local_only", None);
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> WindowsHostConfig {
        WindowsHostConfig {
            product: "Yian Windows Host".to_string(),
            implementation: "rust".to_string(),
            version: "0.1.0".to_string(),
            gateway_base_url: "https://example.test".to_string(),
            identity_id: "windows-demo-001".to_string(),
            device_id: "windows-device-001".to_string(),
            app_instance_id: "windows-app-001".to_string(),
            shared_secret: "shared-secret".to_string(),
            preferred_mode: "relay_url".to_string(),
            host_port: 4510,
            health_url: "http://127.0.0.1:4510/health".to_string(),
            execute_url: "http://127.0.0.1:4510/execute".to_string(),
            register_path: "/local-host/register".to_string(),
            relay_poll_path: "/local-host/relay/poll".to_string(),
            relay_respond_path: "/local-host/relay/respond".to_string(),
            approval_mode: "auto_approve".to_string(),
            remote_enabled: false,
            data_dir: std::env::temp_dir()
                .join(format!("yian_windows_host_test_{}", Uuid::new_v4())),
        }
    }

    fn test_runtime() -> WindowsHostRuntime {
        WindowsHostRuntime::new(test_config()).expect("test runtime")
    }

    fn authorize_all_cells(runtime: &WindowsHostRuntime, verification_id: &str) -> String {
        let authorization = authorize_local_action(
            runtime,
            &json!({
                "requestId": format!("req-auth-{}", Uuid::new_v4()),
                "verificationId": verification_id,
                "answers": runtime.secret_store
                    .read_verification_record(verification_id)
                    .expect("stored verification")
                    .cells
                    .iter()
                    .map(|cell| json!({
                        "cellId": cell.cell_id,
                        "answer": cell.expected_answer.to_ascii_lowercase()
                    }))
                    .collect::<Vec<_>>()
            }),
        );
        assert_eq!(
            authorization.get("status").and_then(Value::as_str),
            Some("success")
        );
        authorization
            .get("result")
            .and_then(|value| value.get("authorizationId"))
            .and_then(Value::as_str)
            .expect("authorization id")
            .to_string()
    }

    fn local_audit_events(runtime: &WindowsHostRuntime) -> Vec<Value> {
        let path = runtime.secret_store.audit_log_path();
        let content = fs::read_to_string(path).expect("audit jsonl");
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("audit line json"))
            .collect()
    }

    #[test]
    fn rejects_unsupported_capability() {
        let runtime = test_runtime();
        let response = execute_request(
            &runtime,
            json!({
                "requestId": "req-unsupported",
                "capability": "deleteEverything"
            }),
        );
        assert_eq!(
            response.get("status").and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            response.get("capability").and_then(Value::as_str),
            Some("deleteEverything")
        );
    }

    #[test]
    fn approves_signature_and_persists_key() {
        let runtime = test_runtime();
        let response = execute_request(
            &runtime,
            json!({
                "requestId": "req-approved",
                "capability": "requestSignature",
                "keyId": "wallet-main"
            }),
        );
        assert_eq!(
            response.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            response
                .get("result")
                .and_then(|value| value.get("approved"))
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(response
            .get("result")
            .and_then(|value| value.get("verificationId"))
            .and_then(Value::as_str)
            .is_some());
        let authorization_id = response
            .get("result")
            .and_then(|value| value.get("authorizationId"))
            .and_then(Value::as_str)
            .expect("authorization id");
        assert!(runtime
            .secret_store
            .signature_key_path("wallet-main")
            .exists());
        assert!(runtime
            .secret_store
            .authorization_path(authorization_id)
            .exists());
    }

    #[test]
    fn password_fill_uses_dpapi_backed_credential_store() {
        let runtime = test_runtime();
        let first = execute_request(
            &runtime,
            json!({
                "requestId": "req-password-1",
                "capability": "requestPasswordFill",
                "credentialRef": "mailbox-primary",
                "usernameHint": "alice",
                "targetOrigin": "https://mail.example.test"
            }),
        );
        let second = execute_request(
            &runtime,
            json!({
                "requestId": "req-password-2",
                "capability": "requestPasswordFill",
                "credentialRef": "mailbox-primary"
            }),
        );
        let first_password = first
            .get("result")
            .and_then(|value| value.get("password"))
            .and_then(Value::as_str)
            .expect("first password");
        let second_password = second
            .get("result")
            .and_then(|value| value.get("password"))
            .and_then(Value::as_str)
            .expect("second password");
        assert_eq!(first_password, second_password);
        assert!(runtime
            .secret_store
            .credential_path("mailbox-primary")
            .exists());
        assert!(second
            .get("result")
            .and_then(|value| value.get("authorizationId"))
            .and_then(Value::as_str)
            .is_some());
    }

    #[test]
    fn verify_authorize_execute_flow_reuses_authorization_session() {
        let runtime = test_runtime();
        let verification = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-verify-1",
                "capability": "requestSignature",
                "keyId": "wallet-flow"
            }),
        );
        assert_eq!(
            verification.get("status").and_then(Value::as_str),
            Some("success")
        );
        let verification_id = verification
            .get("result")
            .and_then(|value| value.get("verificationId"))
            .and_then(Value::as_str)
            .expect("verification id");

        let authorization_id = authorize_all_cells(&runtime, verification_id);

        let execution = execute_request(
            &runtime,
            json!({
                "requestId": "req-exec-1",
                "capability": "requestSignature",
                "keyId": "wallet-flow",
                "authorizationId": authorization_id
            }),
        );
        assert_eq!(
            execution.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            execution
                .get("result")
                .and_then(|value| value.get("authorizationId"))
                .and_then(Value::as_str),
            Some(authorization_id.as_str())
        );
        assert_eq!(
            execution
                .get("result")
                .and_then(|value| value.get("coreBoundary"))
                .and_then(Value::as_str),
            Some("storylock_local_core")
        );
        assert!(execution
            .get("result")
            .and_then(|value| value.get("localCore"))
            .and_then(|value| value.get("coreCallId"))
            .and_then(Value::as_str)
            .is_some());
    }

    #[test]
    fn ui_status_reports_redacted_management_stats() {
        let runtime = test_runtime();
        let success = execute_request(
            &runtime,
            json!({
                "requestId": "req-management-success",
                "capability": "requestPasswordFill",
                "credentialRef": "mailbox-management",
                "requester": "agent-alpha",
                "origin": "https://agent.example.test",
                "remoteRequest": true,
                "remoteInterface": "relay_gateway"
            }),
        );
        assert_eq!(success.get("status").and_then(Value::as_str), Some("success"));

        let denied = execute_request(
            &runtime,
            json!({
                "requestId": "req-management-denied",
                "capability": "requestPasswordFill",
                "credentialRef": "mailbox-management",
                "requester": "agent-alpha",
                "remoteRequest": true,
                "remoteInterface": "relay_gateway",
                "authorizationChannel": "story_edit"
            }),
        );
        assert_eq!(denied.get("status").and_then(Value::as_str), Some("error"));

        let status = ui_status(&runtime);
        let stats = status
            .get("result")
            .and_then(|value| value.get("managementStats"))
            .expect("management stats");

        assert!(stats
            .get("authorizationModes")
            .and_then(Value::as_array)
            .expect("authorization modes")
            .iter()
            .any(|mode| {
                mode.get("channel").and_then(Value::as_str) == Some("story_edit")
                    && mode.get("requiredCells").and_then(Value::as_u64) == Some(22)
                    && mode.get("remoteAllowed").and_then(Value::as_bool) == Some(false)
            }));

        let object = stats
            .get("objects")
            .and_then(Value::as_array)
            .expect("objects")
            .iter()
            .find(|item| item.get("objectRef").and_then(Value::as_str) == Some("mailbox-management"))
            .expect("managed object");
        assert_eq!(object.get("calls").and_then(Value::as_u64), Some(2));
        assert_eq!(object.get("successes").and_then(Value::as_u64), Some(1));
        assert_eq!(object.get("failures").and_then(Value::as_u64), Some(1));

        assert!(stats
            .get("agents")
            .and_then(Value::as_array)
            .expect("agents")
            .iter()
            .any(|item| {
                item.get("name").and_then(Value::as_str) == Some("agent-alpha")
                    && item.get("calls").and_then(Value::as_u64) == Some(2)
            }));
        assert!(stats
            .get("remoteInterfaces")
            .and_then(Value::as_array)
            .expect("remote interfaces")
            .iter()
            .any(|item| item.get("name").and_then(Value::as_str) == Some("relay_gateway")));
        assert!(stats
            .get("errors")
            .and_then(Value::as_array)
            .expect("errors")
            .iter()
            .any(|item| {
                item.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.contains("SLG-002"))
            }));

        assert_eq!(
            status
                .get("result")
                .and_then(|value| value.get("questionBank"))
                .and_then(|value| value.get("path")),
            None
        );
    }

    #[test]
    fn story_template_generation_queues_candidates_for_storylock_pull() {
        let runtime = test_runtime();
        std::env::set_var("STORYLOCK_STORY_LLM_API_KEY", "sk-test-secret-value");

        let generated = story_template_generate(
            &runtime,
            &json!({
                "requestId": "req-story-template",
                "theme": "train station memory",
                "audience": "desktop tester",
                "tone": "precise",
                "questionCount": 24
            }),
        );
        assert_eq!(
            generated.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            generated
                .get("result")
                .and_then(|value| value.get("storyLockPullRequired"))
                .and_then(Value::as_bool),
            Some(true)
        );

        let candidates = story_template_candidates(
            &runtime,
            &json!({
                "requestId": "req-story-template-candidates",
                "limit": 10
            }),
        );
        let result = candidates.get("result").expect("candidate result");
        assert_eq!(
            result
                .get("hostInvokesStoryLock")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            result
                .get("pullModel")
                .and_then(Value::as_str),
            Some("storylock_explicit_pull_only")
        );
        assert!(result
            .get("candidates")
            .and_then(Value::as_array)
            .expect("candidate array")
            .iter()
            .any(|candidate| {
                candidate
                    .get("framework")
                    .and_then(|framework| framework.get("title"))
                    .and_then(Value::as_str)
                    .is_some_and(|title| title.contains("train station memory"))
            }));

        let status = ui_status(&runtime);
        let template_status = status
            .get("result")
            .and_then(|value| value.get("storyTemplateGenerator"))
            .expect("template generator status");
        assert_eq!(
            template_status.get("llmKey").and_then(Value::as_str),
            Some("configured_direct_access")
        );
        assert!(!serde_json::to_string(&status)
            .expect("status json")
            .contains("sk-test-secret-value"));

        std::env::remove_var("STORYLOCK_STORY_LLM_API_KEY");
    }

    #[test]
    fn revoked_authorization_cannot_execute() {
        let runtime = test_runtime();
        let approved = execute_request(
            &runtime,
            json!({
                "requestId": "req-revoke-seed",
                "capability": "requestSignature",
                "keyId": "wallet-revoked"
            }),
        );
        let authorization_id = approved
            .get("result")
            .and_then(|value| value.get("authorizationId"))
            .and_then(Value::as_str)
            .expect("authorization id")
            .to_string();

        let revoke = revoke_local_authorization(
            &runtime,
            &json!({
                "requestId": "req-revoke",
                "authorizationId": authorization_id
            }),
        );
        assert_eq!(
            revoke.get("status").and_then(Value::as_str),
            Some("success")
        );

        let denied = execute_request(
            &runtime,
            json!({
                "requestId": "req-revoked-exec",
                "capability": "requestSignature",
                "keyId": "wallet-revoked",
                "authorizationId": authorization_id
            }),
        );
        assert_eq!(denied.get("status").and_then(Value::as_str), Some("error"));
        assert_eq!(
            denied
                .get("error")
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str),
            Some("authorization_failed")
        );
        let events = local_audit_events(&runtime);
        let rejected = events
            .iter()
            .rev()
            .find(|event| {
                event.get("event_type").and_then(Value::as_str) == Some("execution_rejected")
            })
            .expect("execution rejection audit");
        assert_eq!(
            rejected.get("request_id").and_then(Value::as_str),
            Some("req-revoked-exec")
        );
        assert_eq!(
            rejected.get("error_code").and_then(Value::as_str),
            Some("SLG-003")
        );
        assert_eq!(
            rejected
                .get("meta")
                .and_then(|meta| meta.get("authorizationStatus"))
                .and_then(Value::as_str),
            Some("revoked")
        );
    }

    #[test]
    fn question_bank_can_be_loaded_and_validated() {
        let runtime = test_runtime();
        let loaded =
            load_or_init_question_bank(&runtime.config.data_dir).expect("load question bank");
        assert_eq!(loaded.question_set_version, "windows-local-v1");
        assert!(!loaded.questions.is_empty());
    }

    #[test]
    fn question_bank_import_replaces_runtime_state() {
        let runtime = test_runtime();
        let import_path = runtime.config.data_dir.join("import-bank.json");
        fs::write(
            &import_path,
            serde_json::to_vec_pretty(&json!({
                "schemaVersion": "windows-local-question-bank-v1",
                "questionSetVersion": "windows-local-v2",
                "normalizationVersion": "upper-ascii-v1",
                "questions": [{
                    "questionId": "story-q-99",
                    "promptRef": "prompt-99",
                    "versionTag": "v2",
                    "promptText": "Imported question.",
                    "answer": "HORIZON"
                }]
            }))
            .expect("serialize import bank"),
        )
        .expect("write import bank");

        let response = question_bank_import(
            &runtime,
            &json!({
                "requestId": "req-import-1",
                "sourcePath": import_path.display().to_string()
            }),
        );
        assert_eq!(
            response.get("status").and_then(Value::as_str),
            Some("success")
        );
        let current = runtime
            .current_question_bank()
            .expect("current question bank");
        assert_eq!(current.question_set_version, "windows-local-v2");
        assert_eq!(current.questions.len(), 1);
        assert_eq!(current.questions[0].answer, "HORIZON");
    }

    #[test]
    fn question_bank_import_accepts_utf8_bom() {
        let runtime = test_runtime();
        let import_path = runtime.config.data_dir.join("import-bank-with-bom.json");
        let mut bytes = vec![0xef, 0xbb, 0xbf];
        bytes.extend(
            serde_json::to_vec_pretty(&json!({
                "schemaVersion": "windows-local-question-bank-v1",
                "questionSetVersion": "windows-local-bom-v1",
                "normalizationVersion": "upper-ascii-v1",
                "questions": [{
                    "questionId": "story-q-bom",
                    "promptRef": "prompt-bom",
                    "versionTag": "v1",
                    "promptText": "BOM encoded question.",
                    "answer": "ANCHOR"
                }]
            }))
            .expect("serialize import bank"),
        );
        fs::write(&import_path, bytes).expect("write bom import bank");

        let response = question_bank_import(
            &runtime,
            &json!({
                "requestId": "req-import-bom",
                "sourcePath": import_path.display().to_string()
            }),
        );
        assert_eq!(
            response.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            response
                .get("result")
                .and_then(|value| value.get("questionSetVersion"))
                .and_then(Value::as_str),
            Some("windows-local-bom-v1")
        );
    }

    #[test]
    fn authorization_channels_map_to_windows_grid_policy() {
        let runtime = test_runtime();
        let single = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-channel-single",
                "capability": "requestPasswordFill",
                "credentialRef": "mailbox-single",
                "authorizationChannel": "single_read"
            }),
        );
        assert_eq!(
            single
                .get("result")
                .and_then(|value| value.get("requiredStrength"))
                .and_then(Value::as_str),
            Some("medium")
        );
        assert_eq!(
            single
                .get("result")
                .and_then(|value| value.get("grid"))
                .and_then(|value| value.get("requiredCells"))
                .and_then(Value::as_u64),
            Some(6)
        );

        let batch = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-channel-batch",
                "capability": "requestPasswordFill",
                "credentialRef": "mailbox-batch",
                "authorizationChannel": "batch_read"
            }),
        );
        assert_eq!(
            batch
                .get("result")
                .and_then(|value| value.get("requiredStrength"))
                .and_then(Value::as_str),
            Some("high")
        );
        assert_eq!(
            batch
                .get("result")
                .and_then(|value| value.get("grid"))
                .and_then(|value| value.get("requiredCells"))
                .and_then(Value::as_u64),
            Some(12)
        );

        let story_edit = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-channel-story-edit",
                "capability": "requestPasswordFill",
                "credentialRef": "story-local",
                "authorizationChannel": "story_edit"
            }),
        );
        assert_eq!(
            story_edit
                .get("result")
                .and_then(|value| value.get("requiredStrength"))
                .and_then(Value::as_str),
            Some("story_edit")
        );
        assert_eq!(
            story_edit
                .get("result")
                .and_then(|value| value.get("grid"))
                .and_then(|value| value.get("requiredCells"))
                .and_then(Value::as_u64),
            Some(22)
        );

        let denied_remote = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-channel-remote-story-edit",
                "capability": "requestPasswordFill",
                "credentialRef": "story-local",
                "authorizationChannel": "story_edit",
                "remoteRequest": true
            }),
        );
        assert_eq!(
            denied_remote.get("status").and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            denied_remote
                .get("error")
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str),
            Some("policy_denied")
        );
        let events = local_audit_events(&runtime);
        let policy_denied = events
            .iter()
            .rev()
            .find(|event| {
                event.get("event_type").and_then(Value::as_str) == Some("policy_denied")
                    && event.get("request_id").and_then(Value::as_str)
                        == Some("req-channel-remote-story-edit")
            })
            .expect("remote story_edit policy audit");
        assert_eq!(
            policy_denied.get("result").and_then(Value::as_str),
            Some("denied")
        );
        assert_eq!(
            policy_denied.get("error_type").and_then(Value::as_str),
            Some("policy_denied")
        );
        assert_eq!(
            policy_denied
                .get("meta")
                .and_then(|meta| meta.get("authorizationChannel"))
                .and_then(Value::as_str),
            Some("story_edit")
        );
    }

    #[test]
    fn batch_read_channel_executes_with_redacted_summary_result() {
        let runtime = test_runtime();
        let verification = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-batch-flow-verify",
                "capability": "requestPasswordFill",
                "credentialRef": "batch-resource",
                "authorizationChannel": "batch_read"
            }),
        );
        assert_eq!(
            verification.get("status").and_then(Value::as_str),
            Some("success")
        );
        let verification_id = verification
            .get("result")
            .and_then(|value| value.get("verificationId"))
            .and_then(Value::as_str)
            .expect("verification id");
        let authorization_id = authorize_all_cells(&runtime, verification_id);
        let execution = execute_request(
            &runtime,
            json!({
                "requestId": "req-batch-flow-exec",
                "capability": "requestPasswordFill",
                "credentialRef": "batch-resource",
                "authorizationChannel": "batch_read",
                "authorizationId": authorization_id
            }),
        );
        assert_eq!(
            execution.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            execution
                .get("result")
                .and_then(|value| value.get("allowedAction"))
                .and_then(Value::as_str),
            Some("batch_read")
        );
        assert_eq!(
            execution
                .get("result")
                .and_then(|value| value.get("batchReadAuthorized"))
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(execution
            .get("result")
            .and_then(|value| value.get("password"))
            .is_none());
    }

    #[test]
    fn story_edit_channel_executes_only_as_local_core_authorization() {
        let runtime = test_runtime();
        let verification = create_grid_verification(
            &runtime,
            &json!({
                "requestId": "req-story-edit-verify",
                "capability": "requestPasswordFill",
                "credentialRef": "story-local",
                "authorizationChannel": "story_edit"
            }),
        );
        assert_eq!(
            verification.get("status").and_then(Value::as_str),
            Some("success")
        );
        let verification_id = verification
            .get("result")
            .and_then(|value| value.get("verificationId"))
            .and_then(Value::as_str)
            .expect("verification id");
        let authorization_id = authorize_all_cells(&runtime, verification_id);
        let execution = execute_request(
            &runtime,
            json!({
                "requestId": "req-story-edit-exec",
                "capability": "requestPasswordFill",
                "credentialRef": "story-local",
                "authorizationChannel": "story_edit",
                "authorizationId": authorization_id
            }),
        );
        assert_eq!(
            execution.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            execution
                .get("result")
                .and_then(|value| value.get("allowedAction"))
                .and_then(Value::as_str),
            Some("story_edit")
        );
        assert_eq!(
            execution
                .get("result")
                .and_then(|value| value.get("storyEditAuthorized"))
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(execution
            .get("result")
            .and_then(|value| value.get("storyRawText"))
            .is_none());
    }
}
