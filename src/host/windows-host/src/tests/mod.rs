use super::*;

mod authorization;
mod execution;
mod question_bank;
mod story_templates;
mod ui_status;

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
        remote_enabled: false,
        data_dir: std::env::temp_dir()
            .join(format!("yian_windows_host_test_{}", Uuid::new_v4())),
    }
}

fn test_runtime() -> WindowsHostRuntime {
    WindowsHostRuntime::new(test_config()).expect("test runtime")
}

fn authorize_all_cells(runtime: &WindowsHostRuntime, verification_id: &str) -> String {
    let authorization = authorize_local_action(
        runtime,
        &json!({
            "requestId": format!("req-auth-{}", Uuid::new_v4()),
            "verificationId": verification_id,
            "answers": runtime.secret_store
                .read_verification_record(verification_id)
                .expect("stored verification")
                .cells
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
    authorization
        .get("result")
        .and_then(|value| value.get("authorizationId"))
        .and_then(Value::as_str)
        .expect("authorization id")
        .to_string()
}

fn local_audit_events(runtime: &WindowsHostRuntime) -> Vec<Value> {
    let path = runtime.secret_store.audit_log_path();
    let content = fs::read_to_string(path).expect("audit jsonl");
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("audit line json"))
        .collect()
}
