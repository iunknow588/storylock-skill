use super::*;

impl WindowsHostConfig {
    pub(crate) fn from_env() -> Self {
        let data_dir = resolve_data_dir();
        let stored_remote = load_host_remote_config(&data_dir).unwrap_or_default();
        let stored_gateway = stored_remote
            .gateway_base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("https://yian.cdao.online");
        let gateway_base_url = env_or("STORYLOCK_GATEWAY_URL", stored_gateway);
        let identity_id = env_or("STORYLOCK_IDENTITY_ID", "windows-demo-001");
        let device_id = env_or(
            "STORYLOCK_DEVICE_ID",
            &format!("windows-{}", Uuid::new_v4()),
        );
        let app_instance_id = env_or("STORYLOCK_APP_INSTANCE_ID", &Uuid::new_v4().to_string());
        let shared_secret = env_or(
            "STORYLOCK_ANDROID_SHARED_SECRET",
            &env_or("STORYLOCK_SHARED_SECRET", ""),
        );
        let host_port = env_or("STORYLOCK_WINDOWS_HOST_PORT", "4510")
            .parse::<u16>()
            .unwrap_or(4510);
        let approval_mode = env_or("STORYLOCK_WINDOWS_APPROVAL_MODE", "windows_dialog");
        let remote_enabled = if std::env::var("STORYLOCK_WINDOWS_REMOTE_ENABLED").is_ok() {
            truthy_env("STORYLOCK_WINDOWS_REMOTE_ENABLED", false)
        } else {
            stored_remote.remote_enabled.unwrap_or(false)
        };

        Self {
            product: "Yian Windows Host".to_string(),
            implementation: "rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            gateway_base_url,
            identity_id,
            device_id,
            app_instance_id,
            shared_secret,
            preferred_mode: "relay_url".to_string(),
            host_port,
            health_url: format!("http://127.0.0.1:{host_port}/health"),
            execute_url: format!("http://127.0.0.1:{host_port}/execute"),
            register_path: "/local-host/register".to_string(),
            relay_poll_path: "/local-host/relay/poll".to_string(),
            relay_respond_path: "/local-host/relay/respond".to_string(),
            approval_mode,
            remote_enabled,
            data_dir,
        }
    }

    pub(crate) fn gateway_url(&self, path: &str) -> String {
        format!("{}{}", self.gateway_base_url.trim_end_matches('/'), path)
    }

    pub(crate) fn health_json(&self) -> Value {
        json!({
            "schemaVersion": "windows-host-health-v1",
            "product": self.product,
            "implementation": self.implementation,
            "version": self.version,
            "deviceId": self.device_id,
            "appInstanceId": self.app_instance_id,
            "identityId": self.identity_id,
            "preferredMode": self.preferred_mode,
            "hostPort": self.host_port,
            "serverRunning": true,
            "remoteEnabled": self.remote_enabled,
            "capabilities": if self.remote_enabled {
                json!(["health", "verify", "authorize", "revoke", "execute", "relay_poll"])
            } else {
                json!(["health", "verify", "authorize", "revoke", "execute"])
            },
            "status": "local_core_prototype",
            "core": {
                "name": "StoryLock Local Core",
                "boundary": "windows_dpapi_local_only",
                "callChain": ["verify", "authorize", "execute", "revoke"]
            },
            "approvalMode": self.approval_mode,
            "storage": {
                "provider": "dpapi",
                "visibility": "host_internal_only"
            },
            "questionBank": {
                "visibility": "host_internal_only"
            }
        })
    }
}

pub(crate) fn host_remote_config_path(data_dir: &Path) -> PathBuf {
    data_dir.join("host-config.json")
}

pub(crate) fn load_host_remote_config(data_dir: &Path) -> Result<StoredHostRemoteConfig> {
    let path = host_remote_config_path(data_dir);
    if !path.exists() {
        return Ok(StoredHostRemoteConfig::default());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config =
        serde_json::from_str::<StoredHostRemoteConfig>(content.trim_start_matches('\u{feff}'))
            .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

pub(crate) fn save_host_remote_config(
    data_dir: &Path,
    remote_enabled: bool,
    gateway_base_url: &str,
) -> Result<()> {
    let gateway_base_url = gateway_base_url.trim().trim_end_matches('/').to_string();
    if gateway_base_url.is_empty() {
        anyhow::bail!("Gateway URL cannot be empty");
    }
    if !gateway_base_url.starts_with("https://") && !gateway_base_url.starts_with("http://") {
        anyhow::bail!("Gateway URL must start with http:// or https://");
    }
    fs::create_dir_all(data_dir)
        .with_context(|| format!("failed to create {}", data_dir.display()))?;
    let config = StoredHostRemoteConfig {
        schema_version: "windows-host-remote-config-v1".to_string(),
        remote_enabled: Some(remote_enabled),
        gateway_base_url: Some(gateway_base_url),
    };
    let path = host_remote_config_path(data_dir);
    fs::write(&path, serde_json::to_string_pretty(&config)?)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
