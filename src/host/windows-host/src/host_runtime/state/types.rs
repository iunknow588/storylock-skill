use super::*;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WindowsHostConfig {
    pub(crate) product: String,
    pub(crate) implementation: String,
    pub(crate) version: String,
    pub(crate) gateway_base_url: String,
    pub(crate) identity_id: String,
    pub(crate) device_id: String,
    pub(crate) app_instance_id: String,
    pub(crate) shared_secret: String,
    pub(crate) preferred_mode: String,
    pub(crate) host_port: u16,
    pub(crate) health_url: String,
    pub(crate) execute_url: String,
    pub(crate) register_path: String,
    pub(crate) relay_poll_path: String,
    pub(crate) relay_respond_path: String,
    pub(crate) approval_mode: String,
    pub(crate) remote_enabled: bool,
    pub(crate) data_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct StoryLlmConfig {
    pub(crate) provider: String,
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) model: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegistrationResponse {
    pub(crate) relay: Option<RelayEndpoints>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RelayEndpoints {
    #[serde(rename = "pollUrl")]
    pub(crate) poll_url: Option<String>,
    #[serde(rename = "respondUrl")]
    pub(crate) respond_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredCredential {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) target_origin: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredAuthorizationRecord {
    pub(crate) verification_id: String,
    pub(crate) authorization_id: String,
    pub(crate) capability: String,
    pub(crate) object_ref: String,
    pub(crate) identity_id: String,
    pub(crate) allowed_action: String,
    pub(crate) required_strength: String,
    pub(crate) confirmation_method: String,
    pub(crate) created_at: String,
    pub(crate) expires_at: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredVerificationRecord {
    pub(crate) verification_id: String,
    pub(crate) identity_id: String,
    pub(crate) object_ref: String,
    pub(crate) capability: String,
    pub(crate) allowed_action: String,
    pub(crate) required_strength: String,
    pub(crate) grid_size: u32,
    pub(crate) required_cells: u32,
    pub(crate) cells: Vec<VerificationCell>,
    pub(crate) created_at: String,
    pub(crate) expires_at: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug)]
pub(crate) struct AuthorizationChannelPolicy {
    pub(crate) channel: &'static str,
    pub(crate) required_strength: &'static str,
    pub(crate) allowed_action: &'static str,
    pub(crate) grid_size: u32,
    pub(crate) required_cells: u32,
    pub(crate) remote_allowed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct VerificationCell {
    pub(crate) cell_id: String,
    pub(crate) prompt_ref: String,
    pub(crate) question_id: String,
    pub(crate) version_tag: String,
    pub(crate) prompt_text: String,
    #[serde(default, rename = "answerOptions")]
    pub(crate) answer_options: Vec<String>,
    pub(crate) expected_answer: String,
    pub(crate) position: u32,
    pub(crate) question_set_version: String,
    pub(crate) normalization_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct QuestionBankFile {
    #[serde(rename = "schemaVersion")]
    pub(crate) schema_version: String,
    #[serde(rename = "questionSetVersion")]
    pub(crate) question_set_version: String,
    #[serde(rename = "normalizationVersion")]
    pub(crate) normalization_version: String,
    pub(crate) questions: Vec<QuestionBankEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct QuestionBankEntry {
    #[serde(rename = "questionId")]
    pub(crate) question_id: String,
    #[serde(rename = "promptRef")]
    pub(crate) prompt_ref: String,
    #[serde(rename = "versionTag")]
    pub(crate) version_tag: String,
    #[serde(rename = "promptText")]
    pub(crate) prompt_text: String,
    pub(crate) answer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProtectedEnvelope {
    pub(crate) schema_version: String,
    pub(crate) protected_by: String,
    pub(crate) created_at: String,
    pub(crate) cipher_text: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct RuntimeUiState {
    pub(crate) started_at: String,
    pub(crate) relay_status: String,
    pub(crate) last_relay_error: Option<String>,
    pub(crate) last_relay_poll_at: Option<String>,
    pub(crate) last_execution: Option<Value>,
    pub(crate) last_confirmation: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LocalAuditEvent {
    pub(crate) timestamp: String,
    pub(crate) event_type: String,
    pub(crate) request_id: String,
    pub(crate) capability: String,
    pub(crate) identity_id: String,
    pub(crate) device_id: String,
    pub(crate) object_ref: Option<String>,
    pub(crate) result: String,
    pub(crate) error_code: Option<String>,
    pub(crate) error_type: Option<String>,
    pub(crate) redaction_level: String,
    pub(crate) meta: Value,
}
