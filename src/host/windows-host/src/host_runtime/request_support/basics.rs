use super::*;

pub(crate) fn request_id_from(request: &Value) -> String {
    request
        .get("requestId")
        .and_then(Value::as_str)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("req-{}", Uuid::new_v4()))
}

pub(crate) fn capability_from(request: &Value) -> String {
    request
        .get("capability")
        .and_then(Value::as_str)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "requestSignature".to_string())
}

pub(crate) fn now_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
        .to_string()
}

pub(crate) fn error_response(
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

pub(crate) fn signature_of_request(key_material: &str, request: &Value) -> Result<String> {
    let canonical = serde_json::to_vec(request)?;
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    hasher.update(b":");
    hasher.update(canonical);
    Ok(format!("sha256:{}", hex_string(&hasher.finalize())))
}

pub(crate) fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn requester_from(request: &Value) -> String {
    request
        .get("requester")
        .or_else(|| request.get("origin"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown requester")
        .to_string()
}

pub(crate) fn origin_from(request: &Value) -> String {
    request
        .get("origin")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown origin")
        .to_string()
}

pub(crate) fn expires_at_after(seconds: u64) -> String {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() + seconds)
        .unwrap_or(seconds))
    .to_string()
}

pub(crate) fn normalize_answer(answer: &str, normalization_version: &str) -> String {
    match normalization_version {
        "upper-ascii-v1" => answer.trim().to_ascii_uppercase(),
        _ => answer.trim().to_string(),
    }
}

pub(crate) fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn is_unexpired(expires_at: &str) -> bool {
    expires_at
        .parse::<u64>()
        .map(|expiry| expiry >= now_seconds())
        .unwrap_or(false)
}
