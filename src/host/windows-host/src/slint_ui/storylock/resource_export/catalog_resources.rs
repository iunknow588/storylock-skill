use super::*;
use crate::host_runtime::{now_timestamp, resolve_data_dir, sanitize_ref, StoredCredential};
use anyhow::anyhow;

const DEFAULT_REQUIRED_GRID_COUNT: u64 = 12;
const DEFAULT_AUTHORIZATION_FREQUENCY: &str = "Every high-risk request";

pub(crate) fn normalize_resource_group(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" | "low" | "public" => "normal".to_string(),
        "private" | "medium" => "private".to_string(),
        "secret" | "high" | "top-secret" => "secret".to_string(),
        _ => "normal".to_string(),
    }
}

fn resource_group_from_binding_sensitivity(resource: &Value) -> Option<String> {
    let bindings = resource.get("bindings").and_then(Value::as_array)?;
    let mut has_private = false;
    for binding in bindings {
        let group = binding
            .get("objectMeta")
            .and_then(|meta| meta.get("sensitivity"))
            .and_then(Value::as_str)
            .map(normalize_resource_group)
            .unwrap_or_else(|| "normal".to_string());
        if group == "secret" {
            return Some(group);
        }
        if group == "private" {
            has_private = true;
        }
    }
    Some(if has_private { "private" } else { "normal" }.to_string())
}

pub(crate) fn resource_group_from_catalog_resource(resource: &Value) -> SharedString {
    let group = resource
        .get("resourceGroup")
        .and_then(Value::as_str)
        .map(normalize_resource_group)
        .or_else(|| resource_group_from_binding_sensitivity(resource))
        .unwrap_or_else(|| "normal".to_string());
    SharedString::from(group.as_str())
}

pub(crate) fn normalize_legacy_resource_catalog_groups(catalog: &mut Value) -> bool {
    let Some(resources) = catalog.get_mut("resources").and_then(Value::as_array_mut) else {
        return false;
    };
    let mut changed = false;
    for resource in resources {
        let normalized = resource_group_from_catalog_resource(resource);
        if resource.get("resourceGroup").and_then(Value::as_str) != Some(normalized.as_str()) {
            resource["resourceGroup"] = json!(normalized.to_string());
            changed = true;
        }
    }
    changed
}

pub(crate) fn normalize_legacy_resource_catalog_file(package_dir: &Path) -> Result<()> {
    let mut catalog = read_protected_resources(package_dir);
    if normalize_legacy_resource_catalog_groups(&mut catalog) {
        save_protected_resources(package_dir, catalog)?;
    }
    Ok(())
}

pub(crate) fn format_protected_object_list(catalog: &Value, selected_group: &str) -> String {
    let rows = protected_object_rows(catalog, selected_group);
    if rows.is_empty() {
        "No protected objects in this level yet.".to_string()
    } else {
        format!("{} managed object(s). Click a row to edit.", rows.len())
    }
}

pub(crate) fn set_protected_object_rows_into_window(
    core: &StoryLockCoreApp,
    catalog: &Value,
    selected_group: &str,
) {
    let rows = protected_object_rows(catalog, selected_group);
    let empty = ProtectedObjectRow::default();
    for index in 0..8 {
        let row = rows.get(index).unwrap_or(&empty);
        match index {
            0 => set_row_1(core, row),
            1 => set_row_2(core, row),
            2 => set_row_3(core, row),
            3 => set_row_4(core, row),
            4 => set_row_5(core, row),
            5 => set_row_6(core, row),
            6 => set_row_7(core, row),
            7 => set_row_8(core, row),
            _ => {}
        }
    }
}

#[derive(Default)]
pub(crate) struct ProtectedObjectRow {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) secret: String,
    pub(crate) usage: String,
    pub(crate) level: String,
}

pub(crate) fn protected_object_rows(
    catalog: &Value,
    selected_group: &str,
) -> Vec<ProtectedObjectRow> {
    let selected_group = normalize_resource_group(selected_group);
    let mut rows = Vec::new();
    if let Some(resources) = catalog.get("resources").and_then(Value::as_array) {
        for resource in resources {
            let resource_id = resource
                .get("resourceId")
                .and_then(Value::as_str)
                .unwrap_or("resource");
            let display_name = resource
                .get("displayName")
                .and_then(Value::as_str)
                .unwrap_or(resource_id);
            if resource.get("bindings").and_then(Value::as_array).is_none() {
                continue;
            }
            let group = resource_group_from_catalog_resource(resource);
            if group.as_str() != selected_group {
                continue;
            }
            rows.push(ProtectedObjectRow {
                id: resource_id.to_string(),
                name: display_name.to_string(),
                secret: read_username_for_resource(resource),
                usage: object_usage_label_for_resource(resource),
                level: group.to_string(),
            });
        }
    }
    rows
}

macro_rules! set_row {
    ($fn_name:ident, $id:ident, $name:ident, $secret:ident, $usage:ident, $level:ident) => {
        fn $fn_name(core: &StoryLockCoreApp, row: &ProtectedObjectRow) {
            core.$id(SharedString::from(row.id.as_str()));
            core.$name(SharedString::from(row.name.as_str()));
            core.$secret(SharedString::from(row.secret.as_str()));
            core.$usage(SharedString::from(row.usage.as_str()));
            core.$level(SharedString::from(row.level.as_str()));
        }
    };
}

set_row!(
    set_row_1,
    set_object_1_id,
    set_object_1_name,
    set_object_1_secret,
    set_object_1_usage,
    set_object_1_level
);
set_row!(
    set_row_2,
    set_object_2_id,
    set_object_2_name,
    set_object_2_secret,
    set_object_2_usage,
    set_object_2_level
);
set_row!(
    set_row_3,
    set_object_3_id,
    set_object_3_name,
    set_object_3_secret,
    set_object_3_usage,
    set_object_3_level
);
set_row!(
    set_row_4,
    set_object_4_id,
    set_object_4_name,
    set_object_4_secret,
    set_object_4_usage,
    set_object_4_level
);
set_row!(
    set_row_5,
    set_object_5_id,
    set_object_5_name,
    set_object_5_secret,
    set_object_5_usage,
    set_object_5_level
);
set_row!(
    set_row_6,
    set_object_6_id,
    set_object_6_name,
    set_object_6_secret,
    set_object_6_usage,
    set_object_6_level
);
set_row!(
    set_row_7,
    set_object_7_id,
    set_object_7_name,
    set_object_7_secret,
    set_object_7_usage,
    set_object_7_level
);
set_row!(
    set_row_8,
    set_object_8_id,
    set_object_8_name,
    set_object_8_secret,
    set_object_8_usage,
    set_object_8_level
);

pub(crate) fn first_resource_for_group<'a>(
    catalog: &'a Value,
    selected_group: &str,
) -> Option<&'a Value> {
    let selected_group = normalize_resource_group(selected_group);
    catalog
        .get("resources")
        .and_then(Value::as_array)?
        .iter()
        .find(|resource| {
            let resource_group = resource_group_from_catalog_resource(resource);
            resource_group.as_str() == selected_group
        })
}

pub(crate) fn resource_by_id<'a>(catalog: &'a Value, resource_id: &str) -> Option<&'a Value> {
    catalog
        .get("resources")
        .and_then(Value::as_array)?
        .iter()
        .find(|resource| resource.get("resourceId").and_then(Value::as_str) == Some(resource_id))
}

pub(crate) fn load_resource_into_window(core: &StoryLockCoreApp, resource: &Value) {
    let username = binding_secret_ref(resource, "username")
        .and_then(|secret_ref| read_stored_credential_field(secret_ref.as_str(), "username"));
    let password = binding_secret_ref(resource, "password")
        .and_then(|secret_ref| read_stored_credential_field(secret_ref.as_str(), "password"));
    core.set_resource_id(json_string(resource, &["resourceId"]));
    core.set_resource_kind(json_string(resource, &["resourceKind"]));
    core.set_provider_id(SharedString::from(username.unwrap_or_default()));
    core.set_display_name(uri_from_resource(resource));
    core.set_object_id(object_id_prefix_for_resource(resource));
    core.set_secret_reference(SharedString::from(password.unwrap_or_default()));
    core.set_object_kind(object_type_for_resource(resource));
    let resource_group = resource_group_from_catalog_resource(resource);
    core.set_resource_group(resource_group.clone());
    core.set_editing_resource_group(resource_group);
    core.set_resource_bindings(SharedString::from(format_bindings(resource)));
    core.set_object_meta(SharedString::from(format_object_meta(resource)));
}

pub(crate) fn prepare_new_resource_in_window(core: &StoryLockCoreApp, catalog: &Value) {
    let next_index = catalog
        .get("resources")
        .and_then(Value::as_array)
        .map(|resources| resources.len() + 1)
        .unwrap_or(1);
    let selected_group = normalize_resource_group(core.get_resource_group().as_str());
    let usage = "password_fill";
    let resource_id = format!("managed-object-{next_index}");
    core.set_resource_id(SharedString::from(resource_id.as_str()));
    core.set_display_name(SharedString::from(""));
    core.set_secret_reference(SharedString::from(format!(
        "credential/local/{resource_id}"
    )));
    core.set_object_id(core.get_secret_reference());
    core.set_object_kind(SharedString::from(usage));
    core.set_resource_kind(SharedString::from(resource_kind_for_usage(usage)));
    core.set_provider_id(SharedString::from(""));
    core.set_resource_group(SharedString::from(selected_group.clone()));
    core.set_editing_resource_group(SharedString::from(selected_group));
    core.set_required_correct_count(SharedString::from(DEFAULT_REQUIRED_GRID_COUNT.to_string()));
    core.set_authorization_frequency(SharedString::from(DEFAULT_AUTHORIZATION_FREQUENCY));
}

pub(crate) fn object_id_prefix_for_resource(resource: &Value) -> SharedString {
    let object_id = resource
        .get("bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| bindings.first())
        .and_then(|binding| binding.get("objectId"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let parts = object_id
        .split('/')
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>();
    if parts.len() >= 3 {
        SharedString::from(parts[..3].join("/"))
    } else {
        SharedString::from(object_id)
    }
}

pub(crate) fn object_type_for_resource(resource: &Value) -> SharedString {
    let roles = resource
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .filter_map(|binding| binding.get("role").and_then(Value::as_str))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if roles.iter().any(|role| *role == "password") {
        SharedString::from("password_fill")
    } else if roles
        .iter()
        .any(|role| *role == "private_key" || *role == "signing_key")
    {
        SharedString::from("sign")
    } else {
        SharedString::from(
            resource
                .get("bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| bindings.first())
                .and_then(|binding| binding.get("objectMeta"))
                .and_then(|meta| meta.get("objectKind"))
                .and_then(Value::as_str)
                .unwrap_or("secret"),
        )
    }
}

fn object_usage_label_for_resource(resource: &Value) -> String {
    match object_type_for_resource(resource).as_str() {
        "password_fill" => "website".to_string(),
        "sign" => "signing".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn save_resource_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    save_resource_from_window_with_validation(core, package_dir, false)
}

pub(crate) fn save_object_editor_resource_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    save_resource_from_window_with_validation(core, package_dir, true)
}

pub(crate) fn delete_object_editor_resource_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> Result<()> {
    let resource_id = core.get_resource_id().to_string();
    if resource_id.trim().is_empty() {
        return Err(anyhow!("No managed object is selected"));
    }

    let mut catalog = read_protected_resources(package_dir);
    let mut resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let deleted_resource = resources
        .iter()
        .find(|item| item.get("resourceId").and_then(Value::as_str) == Some(resource_id.as_str()))
        .cloned();
    let original_len = resources.len();
    resources.retain(|item| {
        item.get("resourceId").and_then(Value::as_str) != Some(resource_id.as_str())
    });
    if resources.len() == original_len {
        return Err(anyhow!("Managed object not found: {resource_id}"));
    }

    catalog["version"] = json!("1");
    catalog["resources"] = Value::Array(resources);
    save_protected_resources(package_dir, catalog.clone())?;
    if let Some(resource) = deleted_resource {
        delete_managed_object_credential(&resource);
        remove_template_child_records_for_resource(package_dir, &resource)?;
    }

    core.set_resource_id(SharedString::from(""));
    core.set_resource_kind(SharedString::from(""));
    core.set_provider_id(SharedString::from(""));
    core.set_display_name(SharedString::from(""));
    core.set_object_id(SharedString::from(""));
    core.set_secret_reference(SharedString::from(""));
    core.set_resource_bindings(SharedString::from(""));
    core.set_object_meta(SharedString::from(""));
    let selected_group = normalize_resource_group(core.get_resource_group().as_str());
    core.set_resource_group(SharedString::from(selected_group.as_str()));
    core.set_editing_resource_group(SharedString::from(selected_group.as_str()));
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        selected_group.as_str(),
    )));
    set_protected_object_rows_into_window(core, &catalog, selected_group.as_str());
    Ok(())
}

fn save_resource_from_window_with_validation(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    require_editor_fields: bool,
) -> Result<()> {
    let required_grid_count = core
        .get_required_correct_count()
        .parse::<u64>()
        .unwrap_or(DEFAULT_REQUIRED_GRID_COUNT)
        .clamp(1, 24);
    let sensitivity = normalize_resource_group(core.get_editing_resource_group().as_str());
    let uri = normalize_uri(core.get_display_name().as_str());
    if require_editor_fields && uri.is_empty() {
        return Err(anyhow!("URI is required"));
    }
    let username = core.get_provider_id().trim().to_string();
    if require_editor_fields && username.is_empty() {
        return Err(anyhow!("Username is required"));
    }
    let password = core.get_secret_reference().to_string();
    if require_editor_fields && password.trim().is_empty() {
        return Err(anyhow!("Password is required"));
    }
    let fallback_resource_id = core.get_resource_id().to_string();
    let mut catalog = read_protected_resources(package_dir);
    let mut resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let existing_resource_id = fallback_resource_id.trim().to_string();
    let existing_resource = resources
        .iter()
        .find(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some(existing_resource_id.as_str())
        })
        .cloned();
    let editing_existing_resource = !existing_resource_id.is_empty()
        && resources.iter().any(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some(existing_resource_id.as_str())
        });
    let resource_id = if editing_existing_resource {
        existing_resource_id
    } else if uri.is_empty() {
        if fallback_resource_id.trim().is_empty() {
            "managed-object".to_string()
        } else {
            fallback_resource_id
        }
    } else {
        resource_id_from_uri(uri.as_str())
    };
    let usage = normalize_usage(core.get_object_kind().as_str());
    let resource_kind = resource_kind_for_usage(&usage).to_string();
    let provider_slug = if uri.is_empty() {
        provider_id_from_resource_id(&resource_id)
    } else if editing_existing_resource {
        provider_id_from_resource_id(&resource_id)
    } else {
        provider_id_from_uri(uri.as_str())
    };
    let display_name = if uri.is_empty() {
        core.get_display_name().to_string()
    } else {
        uri.clone()
    };
    let credential_ref = format!(
        "credential/{}/{}",
        sanitize_segment(&provider_slug),
        sanitize_segment(&resource_id)
    );
    let previous_credential_ref = existing_resource.as_ref().and_then(resource_credential_ref);
    if !username.is_empty() && !password.trim().is_empty() && !display_name.trim().is_empty() {
        write_managed_object_credential(&credential_ref, &username, &password, &display_name)?;
    }
    core.set_resource_id(SharedString::from(resource_id.as_str()));
    core.set_resource_kind(SharedString::from(resource_kind.as_str()));
    core.set_provider_id(SharedString::from(username.as_str()));
    core.set_display_name(SharedString::from(display_name.as_str()));
    core.set_secret_reference(SharedString::from(password.as_str()));
    core.set_object_kind(SharedString::from(usage.as_str()));
    core.set_required_correct_count(SharedString::from(required_grid_count.to_string()));
    core.set_authorization_frequency(SharedString::from(DEFAULT_AUTHORIZATION_FREQUENCY));
    let bindings = resource_bindings_from_window(
        core,
        required_grid_count,
        &sensitivity,
        &usage,
        &credential_ref,
    );
    let template_shell = template_shell_from_resource(&resource_id, &display_name, &usage);
    let resource = json!({
            "resourceId": resource_id,
            "resourceKind": resource_kind,
            "providerId": provider_slug,
            "displayName": display_name,
            "resourceGroup": sensitivity.clone(),
            "bindings": bindings,
            "templateShell": template_shell
    });
    let resource_id = resource
        .get("resourceId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if let Some(index) = resources.iter().position(|item| {
        item.get("resourceId").and_then(Value::as_str) == Some(resource_id.as_str())
    }) {
        resources[index] = resource;
    } else {
        resources.push(resource);
    }
    catalog["version"] = json!("1");
    catalog["resources"] = Value::Array(resources);
    save_protected_resources(package_dir, catalog.clone())?;
    sync_templates_for_resource(core, package_dir)?;
    if let Some(saved_resource) = resource_by_id(&catalog, resource_id.as_str()) {
        core.set_resource_bindings(SharedString::from(format_bindings(saved_resource)));
        core.set_object_meta(SharedString::from(format_object_meta(saved_resource)));
    }
    core.set_resource_group(SharedString::from(sensitivity.as_str()));
    core.set_editing_resource_group(SharedString::from(sensitivity.as_str()));
    if let Some(previous_credential_ref) = previous_credential_ref {
        if previous_credential_ref != credential_ref {
            delete_credential_file(&previous_credential_ref);
        }
    }
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        sensitivity.as_str(),
    )));
    set_protected_object_rows_into_window(core, &catalog, sensitivity.as_str());
    Ok(())
}

fn resource_bindings_from_window(
    core: &StoryLockCoreApp,
    required_grid_count: u64,
    sensitivity: &str,
    usage: &str,
    credential_ref: &str,
) -> Vec<Value> {
    let resource_id = core.get_resource_id().to_string();
    let provider_id = provider_id_from_resource_id(&resource_id);
    let object_id = object_prefix_from_window(core, usage);
    let base = object_id_base(&object_id, &provider_id, &resource_id);

    if usage == "password_fill" {
        return vec![
            binding_json(
                "username",
                &format!("{base}/username"),
                "username",
                "text",
                "private",
                credential_ref,
                core,
                required_grid_count,
            ),
            binding_json(
                "password",
                &format!("{base}/password"),
                "password",
                "secret",
                "secret",
                credential_ref,
                core,
                required_grid_count,
            ),
        ];
    }

    if usage == "sign" {
        return vec![
            binding_json(
                "public_key",
                &format!("{base}/public_key"),
                "public_key",
                "text",
                "private",
                credential_ref,
                core,
                required_grid_count,
            ),
            binding_json(
                "private_key",
                &format!("{base}/private_key"),
                "private_key",
                "secret",
                "secret",
                credential_ref,
                core,
                required_grid_count,
            ),
        ];
    }

    let object_id = if object_id.trim().is_empty() {
        format!("{base}/secret")
    } else {
        object_id
    };
    vec![binding_json(
        "protected_object",
        &object_id,
        "secret",
        "secret",
        sensitivity,
        credential_ref,
        core,
        required_grid_count,
    )]
}

fn normalize_uri(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn provider_id_from_uri(uri: &str) -> String {
    let normalized = uri
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    normalized
        .split('/')
        .next()
        .filter(|part| !part.trim().is_empty())
        .map(sanitize_segment)
        .unwrap_or_else(|| "local".to_string())
}

fn resource_id_from_uri(uri: &str) -> String {
    let normalized = uri
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let mut segments = normalized
        .split(['/', '?', '#'])
        .filter(|segment| !segment.trim().is_empty())
        .map(sanitize_segment)
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return "managed-object".to_string();
    }
    if segments.len() == 1 {
        return segments.remove(0);
    }
    segments.join("-")
}

fn uri_from_resource(resource: &Value) -> SharedString {
    let uri = resource
        .get("displayName")
        .and_then(Value::as_str)
        .unwrap_or_default();
    SharedString::from(uri)
}

pub(crate) fn binding_secret_ref(resource: &Value, role: &str) -> Option<String> {
    resource
        .get("bindings")
        .and_then(Value::as_array)?
        .iter()
        .find(|binding| binding.get("role").and_then(Value::as_str) == Some(role))
        .and_then(|binding| binding.get("objectMeta"))
        .and_then(|meta| meta.get("secretRef"))
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

pub(crate) fn read_username_for_resource(resource: &Value) -> String {
    binding_secret_ref(resource, "username")
        .and_then(|secret_ref| read_stored_credential_field(secret_ref.as_str(), "username"))
        .unwrap_or_default()
}

pub(crate) fn read_stored_credential_field(credential_ref: &str, field: &str) -> Option<String> {
    let path = credential_store_path(credential_ref);
    let bytes = fs::read(path).ok()?;
    let envelope: ProtectedEnvelope = serde_json::from_slice(&bytes).ok()?;
    let decrypted = dpapi_unprotect_from_base64(&envelope.cipher_text).ok()?;
    let stored: StoredCredential = serde_json::from_slice(&decrypted).ok()?;
    match field {
        "username" => Some(stored.username),
        "password" => Some(stored.password),
        _ => None,
    }
}

fn resource_credential_ref(resource: &Value) -> Option<String> {
    resource
        .get("bindings")
        .and_then(Value::as_array)?
        .first()
        .and_then(|binding| binding.get("objectMeta"))
        .and_then(|meta| meta.get("secretRef"))
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn write_managed_object_credential(
    credential_ref: &str,
    username: &str,
    password: &str,
    target_origin: &str,
) -> Result<()> {
    let path = credential_store_path(credential_ref);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stored = StoredCredential {
        username: username.to_string(),
        password: password.to_string(),
        target_origin: target_origin.to_string(),
    };
    let bytes = serde_json::to_vec(&stored)?;
    let envelope = ProtectedEnvelope {
        schema_version: "dpapi-protected-v1".to_string(),
        protected_by: "windows-dpapi".to_string(),
        created_at: now_timestamp(),
        cipher_text: dpapi_protect_to_base64(&bytes)?,
    };
    fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;
    Ok(())
}

fn delete_managed_object_credential(resource: &Value) {
    if let Some(credential_ref) = resource_credential_ref(resource) {
        delete_credential_file(&credential_ref);
    }
}

fn delete_credential_file(credential_ref: &str) {
    let path = credential_store_path(credential_ref);
    let _ = fs::remove_file(path);
}

fn credential_store_path(credential_ref: &str) -> std::path::PathBuf {
    resolve_data_dir()
        .join("credentials")
        .join(format!("{}.json", sanitize_ref(credential_ref)))
}

fn normalize_usage(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    if value.contains("sign")
        || value.contains("private_key")
        || value.contains("key_pair")
        || value.contains("signature")
        || value.contains('签')
    {
        "sign".to_string()
    } else {
        "password_fill".to_string()
    }
}

fn resource_kind_for_usage(usage: &str) -> &'static str {
    if usage == "sign" {
        "key_pair"
    } else {
        "website_account"
    }
}

fn provider_id_from_resource_id(resource_id: &str) -> String {
    resource_id
        .split(['-', '/', '_'])
        .next()
        .filter(|part| !part.trim().is_empty())
        .map(sanitize_segment)
        .unwrap_or_else(|| "local".to_string())
}

fn object_prefix_from_window(core: &StoryLockCoreApp, usage: &str) -> String {
    let secret_reference = core.get_secret_reference().to_string();
    if secret_reference.contains('/') {
        return secret_reference;
    }
    let resource_id = core.get_resource_id().to_string();
    let provider_id = provider_id_from_resource_id(&resource_id);
    let prefix = if usage == "sign" {
        "keypair"
    } else {
        "credential"
    };
    format!(
        "{}/{}/{}",
        prefix,
        sanitize_segment(&provider_id),
        sanitize_segment(&resource_id)
    )
}

fn object_id_base(object_id: &str, provider_id: &str, resource_id: &str) -> String {
    let parts = object_id
        .split('/')
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>();
    if parts.len() >= 3 {
        return parts[..3].join("/");
    }
    format!(
        "credential/{}/{}",
        sanitize_segment(provider_id),
        sanitize_segment(resource_id)
    )
}

fn binding_json(
    role: &str,
    object_id: &str,
    object_kind: &str,
    encoding: &str,
    sensitivity: &str,
    credential_ref: &str,
    core: &StoryLockCoreApp,
    required_grid_count: u64,
) -> Value {
    json!({
        "role": role,
        "objectId": object_id,
        "objectMeta": {
            "objectKind": object_kind,
            "encoding": encoding,
            "sensitivity": sensitivity,
            "requiredGridCount": required_grid_count,
            "authorizationFrequency": core.get_authorization_frequency().to_string(),
            "secretRef": credential_ref
        }
    })
}

fn sync_templates_for_resource(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let resource_id = core.get_resource_id().to_string();
    let usage = normalize_usage(core.get_object_kind().as_str());
    let mut vault = read_storylock_vault_payload(package_dir);
    let mut templates = storylock_templates_from_vault(&vault);
    sync_template_children_for_resource(
        &mut templates,
        resource_id.as_str(),
        core.get_display_name().as_str(),
        &usage,
    );

    vault["templates"] = templates;
    save_storylock_vault_payload(package_dir, vault)?;
    ensure_storylock_template_files(package_dir)
}
