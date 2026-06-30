use super::*;

#[test]
fn ui_status_reports_redacted_management_stats() {
    let runtime = test_runtime();
    let authorization_id = grid_authorize_request(
        &runtime,
        json!({
            "requestId": "req-management-success-verify",
            "capability": "requestPasswordFill",
            "credentialRef": "mailbox-management",
            "requester": "agent-alpha",
            "origin": "https://agent.example.test",
            "remoteRequest": true,
            "remoteInterface": "relay_gateway"
        }),
    );
    let success = execute_request(
        &runtime,
        json!({
            "requestId": "req-management-success",
            "capability": "requestPasswordFill",
            "credentialRef": "mailbox-management",
            "requester": "agent-alpha",
            "origin": "https://agent.example.test",
            "remoteRequest": true,
            "remoteInterface": "relay_gateway",
            "authorizationId": authorization_id
        }),
    );
    assert_eq!(
        success.get("status").and_then(Value::as_str),
        Some("success")
    );

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
    assert_eq!(object.get("calls").and_then(Value::as_u64), Some(3));
    assert_eq!(object.get("successes").and_then(Value::as_u64), Some(2));
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
