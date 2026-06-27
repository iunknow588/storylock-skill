use super::*;

#[derive(Clone)]
pub(crate) struct SecretStore {
    pub(crate) root: PathBuf,
}

impl SecretStore {
    pub(crate) fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(root.join("keys"))?;
        fs::create_dir_all(root.join("credentials"))?;
        fs::create_dir_all(root.join("authorizations"))?;
        fs::create_dir_all(root.join("audit"))?;
        fs::create_dir_all(root.join("story-template-requests"))?;
        Ok(Self { root })
    }

    pub(crate) fn signature_key_path(&self, key_id: &str) -> PathBuf {
        self.root
            .join("keys")
            .join(format!("{}.json", sanitize_ref(key_id)))
    }

    pub(crate) fn credential_path(&self, credential_ref: &str) -> PathBuf {
        self.root
            .join("credentials")
            .join(format!("{}.json", sanitize_ref(credential_ref)))
    }

    pub(crate) fn authorization_path(&self, authorization_id: &str) -> PathBuf {
        self.root
            .join("authorizations")
            .join(format!("{}.json", sanitize_ref(authorization_id)))
    }

    pub(crate) fn verification_path(&self, verification_id: &str) -> PathBuf {
        self.root.join("authorizations").join(format!(
            "verification-{}.json",
            sanitize_ref(verification_id)
        ))
    }

    pub(crate) fn audit_log_path(&self) -> PathBuf {
        self.root.join("audit").join("local-audit.jsonl")
    }

    pub(crate) fn story_template_candidates_path(&self) -> PathBuf {
        self.root
            .join("story-template-requests")
            .join("story-template-candidates.jsonl")
    }

    pub(crate) fn story_template_interface_manifest_path(&self) -> PathBuf {
        self.root
            .join("story-template-requests")
            .join("interface-manifest.json")
    }

    pub(crate) fn get_or_create_signature_key(&self, key_id: &str) -> Result<String> {
        let path = self.signature_key_path(key_id);
        if path.exists() {
            return self.read_secret_string(&path);
        }
        let material = format!("sigkey-{}-{}", key_id, Uuid::new_v4());
        self.write_secret_string(&path, &material)?;
        Ok(material)
    }

    pub(crate) fn get_or_create_credential(
        &self,
        credential_ref: &str,
        username_hint: Option<&str>,
        target_origin: Option<&str>,
    ) -> Result<StoredCredential> {
        let path = self.credential_path(credential_ref);
        if path.exists() {
            return self.read_secret_json(&path);
        }
        let credential = StoredCredential {
            username: username_hint.unwrap_or("windows-user").to_string(),
            password: format!("pw-{}-{}", sanitize_ref(credential_ref), short_id()),
            target_origin: target_origin.unwrap_or("https://example.test").to_string(),
        };
        self.write_secret_json(&path, &credential)?;
        Ok(credential)
    }

    pub(crate) fn write_secret_string(&self, path: &Path, secret: &str) -> Result<()> {
        let envelope = ProtectedEnvelope {
            schema_version: "dpapi-protected-v1".to_string(),
            protected_by: "windows-dpapi".to_string(),
            created_at: now_timestamp(),
            cipher_text: dpapi_protect_to_base64(secret.as_bytes())?,
        };
        let serialized = serde_json::to_vec_pretty(&envelope)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    pub(crate) fn read_secret_string(&self, path: &Path) -> Result<String> {
        let envelope: ProtectedEnvelope = serde_json::from_slice(&fs::read(path)?)?;
        let decrypted = dpapi_unprotect_from_base64(&envelope.cipher_text)?;
        String::from_utf8(decrypted).context("stored secret was not valid utf-8")
    }

    pub(crate) fn write_secret_json<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec(value)?;
        let envelope = ProtectedEnvelope {
            schema_version: "dpapi-protected-v1".to_string(),
            protected_by: "windows-dpapi".to_string(),
            created_at: now_timestamp(),
            cipher_text: dpapi_protect_to_base64(&bytes)?,
        };
        fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;
        Ok(())
    }

    pub(crate) fn read_secret_json<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T> {
        let envelope: ProtectedEnvelope = serde_json::from_slice(&fs::read(path)?)?;
        let decrypted = dpapi_unprotect_from_base64(&envelope.cipher_text)?;
        Ok(serde_json::from_slice(&decrypted)?)
    }

    pub(crate) fn write_authorization_record(
        &self,
        record: &StoredAuthorizationRecord,
    ) -> Result<()> {
        self.write_secret_json(&self.authorization_path(&record.authorization_id), record)
    }

    pub(crate) fn write_verification_record(
        &self,
        record: &StoredVerificationRecord,
    ) -> Result<()> {
        self.write_secret_json(&self.verification_path(&record.verification_id), record)
    }

    pub(crate) fn read_verification_record(
        &self,
        verification_id: &str,
    ) -> Result<StoredVerificationRecord> {
        self.read_secret_json(&self.verification_path(verification_id))
    }

    pub(crate) fn read_authorization_record(
        &self,
        authorization_id: &str,
    ) -> Result<StoredAuthorizationRecord> {
        self.read_secret_json(&self.authorization_path(authorization_id))
    }

    pub(crate) fn append_audit_event(&self, event: &LocalAuditEvent) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.audit_log_path())?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub(crate) fn append_story_template_candidate(&self, candidate: &Value) -> Result<()> {
        write_host_json_if_missing(
            &self.story_template_interface_manifest_path(),
            &story_template_interface_manifest(),
        )?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.story_template_candidates_path())?;
        writeln!(file, "{}", serde_json::to_string(candidate)?)?;
        Ok(())
    }

    pub(crate) fn read_story_template_candidates(&self, limit: usize) -> Vec<Value> {
        let Ok(content) = fs::read_to_string(self.story_template_candidates_path()) else {
            return Vec::new();
        };
        let mut items = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect::<Vec<_>>();
        items.reverse();
        items.truncate(limit);
        items
    }
}
