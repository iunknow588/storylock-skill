use super::*;
use std::time::Instant;

const RELAY_LONG_POLL_WAIT_MS: u64 = 25_000;
const RELAY_CLIENT_TIMEOUT_SECS: u64 = 35;
const RELAY_IDLE_RECONNECT_DELAY_MS: u64 = 25;
const RELAY_REREGISTER_AFTER_MS: u128 = 60_000;

pub(crate) fn post_json(
    client: &Client,
    config: &WindowsHostConfig,
    url: &str,
    body: Value,
) -> Result<Value> {
    let mut request = client.post(url).json(&body);
    if !config.shared_secret.is_empty() {
        request = request.header("x-storylock-shared-secret", &config.shared_secret);
    }
    let response = request
        .send()
        .with_context(|| format!("request failed: {url}"))?;
    let status = response.status();
    let value = response.json::<Value>().unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        anyhow::bail!("gateway returned {status}: {value}");
    }
    Ok(value)
}

pub(crate) fn register_host(
    client: &Client,
    config: &WindowsHostConfig,
) -> Result<RegistrationResponse> {
    let payload = json!({
        "identityId": config.identity_id,
        "deviceId": config.device_id,
        "appInstanceId": config.app_instance_id,
        "preferredMode": config.preferred_mode,
        "host": {
            "healthUrl": config.health_url,
            "executeUrl": config.execute_url
        },
        "install": {
            "versionName": config.version,
            "versionCode": 1,
            "packageKind": "windows-rust-prototype"
        },
        "device": {
            "platform": "windows",
            "implementation": "rust",
            "computerName": env_or("COMPUTERNAME", "unknown")
        },
        "reachability": {
            "localHttp": true,
            "relayPolling": true,
            "healthStatus": config.health_json()
        }
    });
    let value = post_json(
        client,
        config,
        &config.gateway_url(&config.register_path),
        payload,
    )?;
    Ok(serde_json::from_value(value)?)
}

fn apply_registration(
    registration: RegistrationResponse,
    poll_url: &mut String,
    respond_url: &mut String,
    long_poll_wait_ms: &mut u64,
    client_timeout_ms: &mut u64,
) {
    if let Some(relay) = registration.relay {
        if let Some(next_poll_url) = relay.poll_url {
            *poll_url = next_poll_url;
        }
        if let Some(next_respond_url) = relay.respond_url {
            *respond_url = next_respond_url;
        }
        if let Some(policy) = relay.poll_policy {
            if let Some(wait_ms) = policy.wait_ms {
                *long_poll_wait_ms = wait_ms.clamp(1_000, 55_000);
            }
            if let Some(timeout_ms) = policy.client_timeout_ms {
                *client_timeout_ms = timeout_ms.clamp(*long_poll_wait_ms + 1_000, 60_000);
            }
        }
    }
}

pub(crate) fn run_relay_loop(runtime: WindowsHostRuntime) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(RELAY_CLIENT_TIMEOUT_SECS))
        .build()?;
    let mut poll_url = runtime.config.gateway_url(&runtime.config.relay_poll_path);
    let mut respond_url = runtime
        .config
        .gateway_url(&runtime.config.relay_respond_path);
    let mut long_poll_wait_ms = RELAY_LONG_POLL_WAIT_MS;
    let mut client_timeout_ms = RELAY_CLIENT_TIMEOUT_SECS * 1000;
    let mut last_registration_at: Instant;

    loop {
        match register_host(&client, &runtime.config) {
            Ok(registration) => {
                apply_registration(
                    registration,
                    &mut poll_url,
                    &mut respond_url,
                    &mut long_poll_wait_ms,
                    &mut client_timeout_ms,
                );
                println!("registered; polling relay at {poll_url}");
                runtime.set_relay_status("online", None);
                last_registration_at = Instant::now();
                break;
            }
            Err(error) => {
                eprintln!("registration failed: {error}");
                runtime.set_relay_status("registration_error", Some(error.to_string()));
                thread::sleep(Duration::from_secs(3));
            }
        }
    }

    loop {
        if last_registration_at.elapsed().as_millis() >= RELAY_REREGISTER_AFTER_MS {
            match register_host(&client, &runtime.config) {
                Ok(registration) => {
                    apply_registration(
                        registration,
                        &mut poll_url,
                        &mut respond_url,
                        &mut long_poll_wait_ms,
                        &mut client_timeout_ms,
                    );
                    runtime.set_relay_status("online", None);
                    last_registration_at = Instant::now();
                }
                Err(error) => {
                    eprintln!("relay re-registration failed: {error}");
                    runtime.set_relay_status("registration_refresh_error", Some(error.to_string()));
                }
            }
        }

        let poll = post_json(
            &client,
            &runtime.config,
            &poll_url,
            json!({
                "transport": "long_poll",
                "deviceId": runtime.config.device_id,
                "appInstanceId": runtime.config.app_instance_id,
                "waitMs": long_poll_wait_ms,
                "clientTimeoutMs": client_timeout_ms
            }),
        );
        match poll {
            Ok(value) if value.get("status").and_then(Value::as_str) == Some("ok") => {
                let relay_request_id = value
                    .get("relayRequestId")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let mut remote_request = value.get("request").cloned().unwrap_or_else(|| json!({}));
                if let Some(request) = remote_request.as_object_mut() {
                    request.insert("remoteRequest".to_string(), Value::Bool(true));
                    request.insert(
                        "remoteInterface".to_string(),
                        Value::String("relay_gateway".to_string()),
                    );
                    request
                        .entry("requester".to_string())
                        .or_insert_with(|| Value::String("relay_gateway".to_string()));
                }
                let response = execute_request(&runtime, remote_request);
                runtime.record_execution_summary(&response);
                runtime.set_relay_status("handled_request", None);
                let _ = post_json(
                    &client,
                    &runtime.config,
                    &respond_url,
                    json!({
                        "relayRequestId": relay_request_id,
                        "response": response
                    }),
                );
            }
            Ok(_) => {
                runtime.set_relay_status("idle", None);
                thread::sleep(Duration::from_millis(RELAY_IDLE_RECONNECT_DELAY_MS));
            }
            Err(error) => {
                eprintln!("relay poll failed: {error}");
                runtime.set_relay_status("poll_error", Some(error.to_string()));
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
