use super::*;

pub(crate) fn question_bank_status(runtime: &WindowsHostRuntime, request: &Value) -> Value {
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

pub(crate) fn ui_status(runtime: &WindowsHostRuntime) -> Value {
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

pub(crate) fn diagnostics_status(runtime: &WindowsHostRuntime) -> Value {
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
