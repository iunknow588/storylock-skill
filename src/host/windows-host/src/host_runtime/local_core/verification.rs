use super::*;

pub(crate) fn object_ref_for_request(capability: &str, request: &Value) -> String {
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

pub(crate) fn build_verification_cells(
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

pub(crate) fn verification_record_from_policy(
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

pub(crate) fn create_grid_verification(runtime: &WindowsHostRuntime, request: &Value) -> Value {
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
    if let Err(error) = runtime.secret_store.write_verification_record(&verification) {
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
