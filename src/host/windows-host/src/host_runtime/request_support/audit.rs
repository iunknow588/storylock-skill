use super::*;

pub(crate) fn record_local_audit(
    runtime: &WindowsHostRuntime,
    event_type: &str,
    request_id: &str,
    capability: &str,
    object_ref: Option<&str>,
    result: &str,
    error_code: Option<&str>,
    error_type: Option<&str>,
    meta: Value,
) {
    let event = LocalAuditEvent {
        timestamp: now_timestamp(),
        event_type: event_type.to_string(),
        request_id: request_id.to_string(),
        capability: capability.to_string(),
        identity_id: runtime.config.identity_id.clone(),
        device_id: runtime.config.device_id.clone(),
        object_ref: object_ref.map(ToOwned::to_owned),
        result: result.to_string(),
        error_code: error_code.map(ToOwned::to_owned),
        error_type: error_type.map(ToOwned::to_owned),
        redaction_level: "audit_meta_only".to_string(),
        meta,
    };
    let _ = runtime.secret_store.append_audit_event(&event);
}

pub(crate) fn summarize_execution_for_ui(response: &Value) -> Value {
    let result = response.get("result").unwrap_or(&Value::Null);
    let audit = response.get("auditMeta").unwrap_or(&Value::Null);
    json!({
        "requestId": response.get("requestId").and_then(Value::as_str).unwrap_or(""),
        "status": response.get("status").and_then(Value::as_str).unwrap_or("unknown"),
        "capability": response.get("capability").and_then(Value::as_str).unwrap_or("unknown"),
        "objectRef": audit.get("objectRef").and_then(Value::as_str)
            .or_else(|| result.get("keyId").and_then(Value::as_str))
            .or_else(|| result.get("credentialRef").and_then(Value::as_str))
            .unwrap_or(""),
        "verificationId": audit.get("verificationId").and_then(Value::as_str)
            .or_else(|| result.get("verificationId").and_then(Value::as_str))
            .unwrap_or(""),
        "authorizationId": audit.get("authorizationId").and_then(Value::as_str)
            .or_else(|| result.get("authorizationId").and_then(Value::as_str))
            .unwrap_or(""),
        "requiredStrength": audit.get("requiredStrength").and_then(Value::as_str)
            .or_else(|| result.get("requiredStrength").and_then(Value::as_str))
            .unwrap_or(""),
        "allowedAction": audit.get("allowedAction").and_then(Value::as_str)
            .or_else(|| result.get("allowedAction").and_then(Value::as_str))
            .unwrap_or(""),
        "redactionLevel": response.get("redactionLevel").and_then(Value::as_str).unwrap_or("full"),
        "timestamp": audit.get("timestamp").and_then(Value::as_str).unwrap_or(""),
        "errorType": response.get("error")
            .and_then(|error| error.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("")
    })
}

pub(crate) fn increment_counter(map: &mut BTreeMap<String, u64>, key: &str) {
    let normalized = key.trim();
    if !normalized.is_empty() {
        *map.entry(normalized.to_string()).or_insert(0) += 1;
    }
}

pub(crate) fn audit_meta_str<'a>(event: &'a LocalAuditEvent, key: &str) -> Option<&'a str> {
    event
        .meta
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn request_audit_context(request: &Value) -> Value {
    json!({
        "requester": requester_from(request),
        "origin": origin_from(request),
        "remoteRequest": request
            .get("remoteRequest")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "remoteInterface": request
            .get("remoteInterface")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("local_api")
    })
}

pub(crate) fn merge_audit_meta(mut base: Value, context: Value) -> Value {
    if let (Some(base), Some(context)) = (base.as_object_mut(), context.as_object()) {
        for (key, value) in context {
            base.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }
    base
}

pub(crate) fn read_local_audit_events(path: &Path) -> Vec<LocalAuditEvent> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<LocalAuditEvent>(line).ok())
        .collect()
}

pub(crate) fn counter_entries(map: BTreeMap<String, u64>) -> Value {
    Value::Array(
        map.into_iter()
            .map(|(name, calls)| json!({ "name": name, "calls": calls }))
            .collect(),
    )
}

pub(crate) fn host_management_stats(runtime: &WindowsHostRuntime) -> Value {
    let events = read_local_audit_events(&runtime.secret_store.audit_log_path());
    let mut object_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut object_successes: BTreeMap<String, u64> = BTreeMap::new();
    let mut object_failures: BTreeMap<String, u64> = BTreeMap::new();
    let mut object_last_seen: BTreeMap<String, String> = BTreeMap::new();
    let mut object_capabilities: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    let mut agent_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut channel_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut remote_interfaces: BTreeMap<String, u64> = BTreeMap::new();
    let mut error_calls: BTreeMap<String, u64> = BTreeMap::new();
    let mut total_calls = 0_u64;
    let mut success_calls = 0_u64;
    let mut failed_calls = 0_u64;
    let mut remote_calls = 0_u64;

    for event in &events {
        total_calls += 1;
        let result = event.result.as_str();
        if matches!(result, "success" | "approved") {
            success_calls += 1;
        }
        if matches!(result, "error" | "failed" | "denied") || event.error_code.is_some() {
            failed_calls += 1;
        }

        if let Some(object_ref) = event
            .object_ref
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            increment_counter(&mut object_calls, object_ref);
            object_last_seen.insert(object_ref.to_string(), event.timestamp.clone());
            increment_counter(
                object_capabilities
                    .entry(object_ref.to_string())
                    .or_default(),
                &event.capability,
            );
            if matches!(result, "success" | "approved") {
                increment_counter(&mut object_successes, object_ref);
            }
            if matches!(result, "error" | "failed" | "denied") || event.error_code.is_some() {
                increment_counter(&mut object_failures, object_ref);
            }
        }

        let requester = audit_meta_str(event, "requester")
            .or_else(|| audit_meta_str(event, "agentId"))
            .or_else(|| audit_meta_str(event, "origin"))
            .unwrap_or("unknown agent");
        increment_counter(&mut agent_calls, requester);

        if let Some(channel) = audit_meta_str(event, "authorizationChannel") {
            increment_counter(&mut channel_calls, channel);
        } else if let Some(action) = audit_meta_str(event, "allowedAction") {
            increment_counter(&mut channel_calls, action);
        }

        let remote_interface = audit_meta_str(event, "remoteInterface")
            .or_else(|| audit_meta_str(event, "origin"))
            .unwrap_or(
                if event
                    .meta
                    .get("remoteRequest")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    "remote_gateway"
                } else {
                    "local_api"
                },
            );
        increment_counter(&mut remote_interfaces, remote_interface);
        if remote_interface != "local_api"
            || event
                .meta
                .get("remoteRequest")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        {
            remote_calls += 1;
        }

        if let Some(error_code) = event.error_code.as_deref() {
            let key = match event.error_type.as_deref() {
                Some(error_type) if !error_type.trim().is_empty() => {
                    format!("{error_code}:{error_type}")
                }
                _ => error_code.to_string(),
            };
            increment_counter(&mut error_calls, &key);
        }
    }

    let objects = Value::Array(
        object_calls
            .into_iter()
            .map(|(object_ref, calls)| {
                json!({
                    "objectRef": object_ref,
                    "calls": calls,
                    "successes": object_successes.get(&object_ref).copied().unwrap_or(0),
                    "failures": object_failures.get(&object_ref).copied().unwrap_or(0),
                    "lastSeenAt": object_last_seen.get(&object_ref).cloned().unwrap_or_default(),
                    "capabilities": counter_entries(object_capabilities.remove(&object_ref).unwrap_or_default())
                })
            })
            .collect(),
    );

    json!({
        "schemaVersion": "windows-host-management-stats-v1",
        "generatedAt": now_timestamp(),
        "authorizationModes": management_authorization_modes_json(),
        "summary": {
            "auditEvents": total_calls,
            "successes": success_calls,
            "failures": failed_calls,
            "managedObjects": objects.as_array().map(Vec::len).unwrap_or(0),
            "remoteInterfaceCalls": remote_calls
        },
        "objects": objects,
        "agents": counter_entries(agent_calls),
        "authorizationChannels": counter_entries(channel_calls),
        "remoteInterfaces": counter_entries(remote_interfaces),
        "errors": counter_entries(error_calls),
        "redaction": {
            "level": "audit_meta_only",
            "hidden": ["answers", "password", "privateKey", "signingKeyBytes", "storyRawText", "sharedSecret", "drafts", "vaultFiles", "packagePaths"]
        }
    })
}
