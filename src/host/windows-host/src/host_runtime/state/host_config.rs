use super::*;

impl WindowsHostConfig {
    pub(crate) fn from_env() -> Self {
        let gateway_base_url = env_or("STORYLOCK_GATEWAY_URL", "https://yian.cdao.online");
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
        let remote_enabled = truthy_env("STORYLOCK_WINDOWS_REMOTE_ENABLED", false);
        let data_dir = resolve_data_dir();

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
