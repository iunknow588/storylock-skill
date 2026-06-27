use super::*;

pub(crate) fn known_authorization_modes() -> Vec<AuthorizationChannelPolicy> {
    vec![
        AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: "medium",
            allowed_action: "password_fill_or_signature",
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        },
        AuthorizationChannelPolicy {
            channel: "batch_read",
            required_strength: "high",
            allowed_action: "batch_read",
            grid_size: 12,
            required_cells: 12,
            remote_allowed: true,
        },
        AuthorizationChannelPolicy {
            channel: "story_edit",
            required_strength: "story_edit",
            allowed_action: "story_edit",
            grid_size: 24,
            required_cells: 22,
            remote_allowed: false,
        },
    ]
}

pub(crate) fn management_authorization_modes_json() -> Value {
    Value::Array(
        known_authorization_modes()
            .into_iter()
            .map(|policy| {
                json!({
                    "channel": policy.channel,
                    "requiredStrength": policy.required_strength,
                    "allowedAction": policy.allowed_action,
                    "gridSize": policy.grid_size,
                    "requiredCells": policy.required_cells,
                    "remoteAllowed": policy.remote_allowed
                })
            })
            .collect(),
    )
}

pub(crate) fn required_strength_for(capability: &str) -> &'static str {
    if capability == "requestSignature" {
        "high"
    } else {
        "medium"
    }
}

pub(crate) fn allowed_action_for(capability: &str) -> &'static str {
    if capability == "requestSignature" {
        "signature"
    } else {
        "password_fill"
    }
}

pub(crate) fn authorization_channel_for_request(capability: &str, request: &Value) -> String {
    request
        .get("authorizationChannel")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            if request
                .get("requestedAction")
                .and_then(Value::as_str)
                .is_some_and(|action| action == "story_edit")
            {
                "story_edit".to_string()
            } else if request
                .get("requestedAction")
                .and_then(Value::as_str)
                .is_some_and(|action| action == "batch_read")
            {
                "batch_read".to_string()
            } else {
                match capability {
                    "requestSignature" | "requestPasswordFill" => "single_read".to_string(),
                    _ => "single_read".to_string(),
                }
            }
        })
}

pub(crate) fn channel_policy_for_request(
    capability: &str,
    request: &Value,
) -> Result<AuthorizationChannelPolicy> {
    let channel = authorization_channel_for_request(capability, request);
    let policy = match channel.as_str() {
        "single_read" => AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: "medium",
            allowed_action: allowed_action_for(capability),
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        },
        "batch_read" => AuthorizationChannelPolicy {
            channel: "batch_read",
            required_strength: "high",
            allowed_action: "batch_read",
            grid_size: 12,
            required_cells: 12,
            remote_allowed: true,
        },
        "story_edit" => AuthorizationChannelPolicy {
            channel: "story_edit",
            required_strength: "story_edit",
            allowed_action: "story_edit",
            grid_size: 24,
            required_cells: 22,
            remote_allowed: false,
        },
        _ => {
            return Err(anyhow!(
                "authorizationChannel must be single_read, batch_read, or story_edit"
            ))
        }
    };
    if request
        .get("remoteRequest")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && !policy.remote_allowed
    {
        return Err(anyhow!(
            "story_edit is local-only and cannot be triggered by the remote gateway"
        ));
    }
    Ok(policy)
}
