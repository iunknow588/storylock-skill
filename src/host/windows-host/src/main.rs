use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
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
            "capabilities": ["health", "verify", "authorize", "revoke", "execute", "relay_poll"],
            "status": "local_core_prototype",
            "core": {
                "name": "StoryLock Local Core",
                "boundary": "windows_dpapi_local_only",
                "callChain": ["verify", "authorize", "execute", "revoke"]
            },
            "approvalMode": self.approval_mode,
        "storage": {
                "provider": "dpapi",
                "path": self.data_dir.display().to_string()
            },
            "questionBank": {
                "path": question_bank_path(&self.data_dir).display().to_string()
            }
        })
    }
}

#[derive(Clone)]
struct WindowsHostRuntime {
    config: WindowsHostConfig,
    secret_store: SecretStore,
    question_bank: Arc<Mutex<QuestionBankFile>>,
}

impl WindowsHostRuntime {
    fn new(config: WindowsHostConfig) -> Result<Self> {
        let secret_store = SecretStore::new(config.data_dir.clone())?;
        let question_bank = load_or_init_question_bank(&config.data_dir)?;
        Ok(Self {
            config,
            secret_store,
            question_bank: Arc::new(Mutex::new(question_bank)),
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
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
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
    let parsed: QuestionBankFile = serde_json::from_str(&content)
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

fn request_summary(request: &Value, capability: &str, object_ref: &str) -> String {
    let requester = request
        .get("origin")
        .or_else(|| request.get("requester"))
        .and_then(Value::as_str)
        .unwrap_or("unknown requester");
    format!(
        "Approve local execution?\n\nCapability: {capability}\nObject: {object_ref}\nRequester: {requester}\n\nChoose Yes to allow this request on the Windows host."
    )
}

fn is_confirmation_approved(config: &WindowsHostConfig, request: &Value, object_ref: &str) -> bool {
    match config.approval_mode.as_str() {
        "auto_approve" => true,
        "auto_deny" => false,
        _ => show_windows_confirmation_dialog(
            "Yian Windows Host Confirmation",
            &request_summary(request, &capability_from(request), object_ref),
        ),
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

fn grid_policy_for(required_strength: &str) -> (u32, u32) {
    match required_strength {
        "high" => (9, 3),
        "medium" => (6, 2),
        _ => (3, 1),
    }
}

fn build_verification_cells(
    question_bank: &QuestionBankFile,
    required_strength: &str,
    object_ref: &str,
) -> Vec<VerificationCell> {
    let (_, required_cells) = grid_policy_for(required_strength);
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

fn verification_record_from_request(
    config: &WindowsHostConfig,
    question_bank: &QuestionBankFile,
    capability: &str,
    object_ref: &str,
) -> StoredVerificationRecord {
    let required_strength = required_strength_for(capability).to_string();
    let (grid_size, required_cells) = grid_policy_for(&required_strength);
    let cells = build_verification_cells(question_bank, &required_strength, object_ref);
    StoredVerificationRecord {
        verification_id: format!("ver-{}", Uuid::new_v4()),
        identity_id: config.identity_id.clone(),
        object_ref: object_ref.to_string(),
        capability: capability.to_string(),
        allowed_action: allowed_action_for(capability).to_string(),
        required_strength: required_strength.clone(),
        grid_size,
        required_cells,
        cells,
        created_at: now_timestamp(),
        expires_at: expires_at_after(180),
        status: "pending".to_string(),
    }
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
    let verification =
        verification_record_from_request(config, &question_bank, &capability, object_ref);
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
            return error_response(
                config,
                &request_id,
                "revokeLocalAuthorization",
                "SLG-003",
                "authorization_failed",
                &format!("authorization record was not found: {error}"),
                "Create a new authorization session if the old one is no longer available.",
            )
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

    if capability == "requestSignature" {
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

    let resolved_authorization = if let Some(authorization_id) = request
        .get("authorizationId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        match runtime.secret_store.read_authorization_record(authorization_id) {
            Ok(record) => {
                if let Err(error) = validate_authorization_for_core(&record, &capability, &object_ref) {
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
                return error_response(
                    config,
                    &request_id,
                    &capability,
                    "SLG-003",
                    "authorization_failed",
                    &format!("authorizationId could not be resolved: {error}"),
                    "Create a fresh verification and authorization session, then retry the execute call.",
                )
            }
        }
    } else {
        if !is_confirmation_approved(config, &request, &object_ref) {
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
            allowed_action: allowed_action_for(&capability).to_string(),
            required_strength: required_strength_for(&capability).to_string(),
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
        Ok(result) => json!({
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
        }),
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
                (&Method::Post, "/execute") => {
                    let mut body = String::new();
                    let payload = match request.as_reader().read_to_string(&mut body) {
                        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
                        Err(_) => json!({}),
                    };
                    Response::from_string(execute_request(&runtime, payload).to_string())
                        .with_header(content_type_json())
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
                break;
            }
            Err(error) => {
                eprintln!("registration failed: {error}");
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
                let remote_request = value.get("request").cloned().unwrap_or_else(|| json!({}));
                let response = execute_request(&runtime, remote_request);
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
            Ok(_) => thread::sleep(Duration::from_millis(750)),
            Err(error) => {
                eprintln!("relay poll failed: {error}");
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

    let runtime = WindowsHostRuntime::new(config)?;
    println!("{}", serde_json::to_string_pretty(&runtime.config)?);
    let server_runtime = runtime.clone();
    let _server = start_local_server(server_runtime)?;
    run_relay_loop(runtime)
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
            data_dir: std::env::temp_dir()
                .join(format!("yian_windows_host_test_{}", Uuid::new_v4())),
        }
    }

    fn test_runtime() -> WindowsHostRuntime {
        WindowsHostRuntime::new(test_config()).expect("test runtime")
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

        let authorization = authorize_local_action(
            &runtime,
            &json!({
                "requestId": "req-auth-1",
                "verificationId": verification_id,
                "answers": build_verification_cells(&runtime.current_question_bank().expect("question bank"), "high", "wallet-flow")
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
        let authorization_id = authorization
            .get("result")
            .and_then(|value| value.get("authorizationId"))
            .and_then(Value::as_str)
            .expect("authorization id");

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
            Some(authorization_id)
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
}
