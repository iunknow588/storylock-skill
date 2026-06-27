use super::*;

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

pub(crate) fn run_relay_loop(runtime: WindowsHostRuntime) -> Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(25)).build()?;
    let mut poll_url = runtime.config.gateway_url(&runtime.config.relay_poll_path);
    let mut respond_url = runtime
        .config
        .gateway_url(&runtime.config.relay_respond_path);

    loop {
        match register_host(&client, &runtime.config) {
            Ok(registration) => {
                if let Some(relay) = registration.relay {
                    if let Some(next_poll_url) = relay.poll_url {
                        poll_url = next_poll_url;
                    }
                    if let Some(next_respond_url) = relay.respond_url {
                        respond_url = next_respond_url;
                    }
                }
                println!("registered; polling relay at {poll_url}");
                runtime.set_relay_status("online", None);
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
        let poll = post_json(
            &client,
            &runtime.config,
            &poll_url,
            json!({
                "deviceId": runtime.config.device_id,
                "appInstanceId": runtime.config.app_instance_id
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
                thread::sleep(Duration::from_millis(750));
            }
            Err(error) => {
                eprintln!("relay poll failed: {error}");
                runtime.set_relay_status("poll_error", Some(error.to_string()));
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
