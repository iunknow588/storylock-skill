use super::*;

pub(crate) fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn show_windows_confirmation_dialog(title: &str, body: &str) -> bool {
    let title_w = wide_null(title);
    let body_w = wide_null(body);
    let response = unsafe {
        MessageBoxW(
            HWND::default(),
            body_w.as_ptr(),
            title_w.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };
    response == IDYES
}

pub(crate) fn risk_description_for(capability: &str) -> &'static str {
    if capability == "requestSignature" {
        "High sensitivity: approval signs with a local DPAPI-protected key. Verify the requester and object before approving."
    } else {
        "Medium-high sensitivity: approval allows local credential fill. Verify the target origin before approving."
    }
}

pub(crate) fn confirmation_summary_for(
    config: &WindowsHostConfig,
    request: &Value,
    capability: &str,
    object_ref: &str,
    status: &str,
) -> Value {
    let expires_at = expires_at_after(300);
    let policy = channel_policy_for_request(capability, request).unwrap_or_else(|_| {
        AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: required_strength_for(capability),
            allowed_action: allowed_action_for(capability),
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        }
    });
    json!({
        "requestId": request_id_from(request),
        "status": status,
        "capability": capability,
        "objectRef": object_ref,
        "requester": requester_from(request),
        "origin": origin_from(request),
        "requiredStrength": policy.required_strength,
        "allowedAction": policy.allowed_action,
        "authorizationChannel": policy.channel,
        "expiry": expires_at,
        "risk": risk_description_for(capability),
        "approvalMode": config.approval_mode,
        "redactionLevel": "audit_meta_only",
        "hiddenFromUi": ["answers", "password", "privateKey", "signingKeyBytes", "storyRawText"],
        "timestamp": now_timestamp()
    })
}

pub(crate) fn request_summary(request: &Value, capability: &str, object_ref: &str) -> String {
    let requester = request
        .get("requester")
        .or_else(|| request.get("origin"))
        .and_then(Value::as_str)
        .unwrap_or("unknown requester");
    let origin = origin_from(request);
    let policy = channel_policy_for_request(capability, request).unwrap_or_else(|_| {
        AuthorizationChannelPolicy {
            channel: "single_read",
            required_strength: required_strength_for(capability),
            allowed_action: allowed_action_for(capability),
            grid_size: 9,
            required_cells: 6,
            remote_allowed: true,
        }
    });
    let required_strength = policy.required_strength;
    let allowed_action = policy.allowed_action;
    let expiry = expires_at_after(300);
    let risk = risk_description_for(capability);
    format!(
        "Approve local execution?\n\nCapability: {capability}\nObject: {object_ref}\nRequester: {requester}\nOrigin: {origin}\nRequired strength: {required_strength}\nAllowed action: {allowed_action}\nAuthorization channel: {}\nExpires at: {expiry}\n\nRisk:\n{risk}\n\nChoose Yes to allow this request on the Windows host.",
        policy.channel
    )
}

#[cfg(feature = "ui-slint")]
pub(crate) fn show_slint_confirmation_dialog(summary: &Value) -> bool {
    match slint_ui::confirm_request(summary) {
        Ok(approved) => approved,
        Err(error) => {
            eprintln!("Slint confirmation failed; falling back to Windows dialog: {error}");
            show_windows_confirmation_dialog(
                "Yian Windows Host Confirmation",
                &format!(
                    "Approve local execution?\n\n{}\n\nChoose Yes to allow this request on the Windows host.",
                    serde_json::to_string_pretty(summary)
                        .unwrap_or_else(|_| "request details unavailable".to_string())
                ),
            )
        }
    }
}

#[cfg(not(feature = "ui-slint"))]
pub(crate) fn show_slint_confirmation_dialog(summary: &Value) -> bool {
    eprintln!(
        "STORYLOCK_WINDOWS_APPROVAL_MODE=slint_dialog requires the ui-slint feature; falling back to Windows dialog."
    );
    show_windows_confirmation_dialog(
        "Yian Windows Host Confirmation",
        &format!(
            "Approve local execution?\n\n{}\n\nChoose Yes to allow this request on the Windows host.",
            serde_json::to_string_pretty(summary)
                .unwrap_or_else(|_| "request details unavailable".to_string())
        ),
    )
}

pub(crate) fn is_confirmation_approved(
    runtime: &WindowsHostRuntime,
    request: &Value,
    object_ref: &str,
) -> bool {
    let config = &runtime.config;
    let capability = capability_from(request);
    let pending_summary =
        confirmation_summary_for(config, request, &capability, object_ref, "pending");
    runtime.record_confirmation_summary(pending_summary.clone());
    match config.approval_mode.as_str() {
        "auto_approve" => {
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                "approved",
            ));
            true
        }
        "auto_deny" => {
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                "denied",
            ));
            false
        }
        "slint_dialog" => {
            let approved = show_slint_confirmation_dialog(&pending_summary);
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                if approved { "approved" } else { "denied" },
            ));
            approved
        }
        _ => {
            let approved = show_windows_confirmation_dialog(
                "Yian Windows Host Confirmation",
                &request_summary(request, &capability, object_ref),
            );
            runtime.record_confirmation_summary(confirmation_summary_for(
                config,
                request,
                &capability,
                object_ref,
                if approved { "approved" } else { "denied" },
            ));
            approved
        }
    }
}
