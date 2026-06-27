use super::*;

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
