use super::*;

pub(crate) fn local_core_call_envelope(
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

pub(crate) fn execute_with_local_core(
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
        let key_material = runtime.secret_store.get_or_create_signature_key(object_ref)?;
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

pub(crate) fn execute_request(runtime: &WindowsHostRuntime, request: Value) -> Value {
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
        match runtime.secret_store.read_authorization_record(authorization_id) {
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
        if let Err(error) = runtime.secret_store.write_authorization_record(&authorization_record) {
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
