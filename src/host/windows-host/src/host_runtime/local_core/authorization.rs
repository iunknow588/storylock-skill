use super::*;

pub(crate) fn validate_authorization_for_core(
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

pub(crate) fn authorize_local_action(runtime: &WindowsHostRuntime, request: &Value) -> Value {
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
    let verification = match runtime.secret_store.read_verification_record(verification_id) {
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
    if let Err(error) = runtime.secret_store.write_authorization_record(&authorization) {
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

pub(crate) fn revoke_local_authorization(runtime: &WindowsHostRuntime, request: &Value) -> Value {
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
    let mut record = match runtime.secret_store.read_authorization_record(&authorization_id) {
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
