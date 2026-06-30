use super::*;

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
fn execute_requires_grid_authorization_before_storylock_call() {
    let runtime = test_runtime();
    let response = execute_request(
        &runtime,
        json!({
            "requestId": "req-needs-grid",
            "capability": "requestSignature",
            "keyId": "wallet-needs-grid"
        }),
    );
    assert_eq!(response.get("status").and_then(Value::as_str), Some("error"));
    assert_eq!(
        response
            .get("error")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str),
        Some("authorization_required")
    );
}

#[test]
fn approves_signature_and_persists_key() {
    let runtime = test_runtime();
    let authorization_id = grid_authorize_request(
        &runtime,
        json!({
            "requestId": "req-approved-verify",
            "capability": "requestSignature",
            "keyId": "wallet-main"
        }),
    );
    let response = execute_request(
        &runtime,
        json!({
            "requestId": "req-approved",
            "capability": "requestSignature",
            "keyId": "wallet-main",
            "authorizationId": authorization_id
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
    assert!(runtime
        .secret_store
        .signature_key_path("wallet-main")
        .exists());
    assert!(runtime
        .secret_store
        .authorization_path(&authorization_id)
        .exists());
}

#[test]
fn password_fill_uses_dpapi_backed_credential_store() {
    let runtime = test_runtime();
    let first_authorization_id = grid_authorize_request(
        &runtime,
        json!({
            "requestId": "req-password-verify-1",
            "capability": "requestPasswordFill",
            "credentialRef": "mailbox-primary"
        }),
    );
    let first = execute_request(
        &runtime,
        json!({
            "requestId": "req-password-1",
            "capability": "requestPasswordFill",
            "credentialRef": "mailbox-primary",
            "usernameHint": "alice",
            "targetOrigin": "https://mail.example.test",
            "authorizationId": first_authorization_id
        }),
    );
    let second_authorization_id = grid_authorize_request(
        &runtime,
        json!({
            "requestId": "req-password-verify-2",
            "capability": "requestPasswordFill",
            "credentialRef": "mailbox-primary"
        }),
    );
    let second = execute_request(
        &runtime,
        json!({
            "requestId": "req-password-2",
            "capability": "requestPasswordFill",
            "credentialRef": "mailbox-primary",
            "authorizationId": second_authorization_id
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
fn revoked_authorization_cannot_execute() {
    let runtime = test_runtime();
    let authorization_id = grid_authorize_request(
        &runtime,
        json!({
            "requestId": "req-revoke-seed-verify",
            "capability": "requestSignature",
            "keyId": "wallet-revoked"
        }),
    );

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
        .find(|event| event.get("event_type").and_then(Value::as_str) == Some("execution_rejected"))
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
