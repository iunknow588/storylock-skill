use super::*;

pub(crate) fn normalize_resource_group(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => "normal".to_string(),
        "private" => "private".to_string(),
        "secret" => "secret".to_string(),
        _ => "normal".to_string(),
    }
}

pub(crate) fn resource_group_from_catalog_resource(resource: &Value) -> SharedString {
    let group = resource
        .get("resourceGroup")
        .and_then(Value::as_str)
        .or_else(|| {
            resource
                .get("bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| bindings.first())
                .and_then(|binding| binding.get("objectMeta"))
                .and_then(|meta| meta.get("sensitivity"))
                .and_then(Value::as_str)
        })
        .unwrap_or("normal");
    SharedString::from(normalize_resource_group(group))
}

pub(crate) fn format_protected_object_list(catalog: &Value, selected_group: &str) -> String {
    let selected_group = normalize_resource_group(selected_group);
    let mut items = Vec::new();
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
            let resource_group = resource
                .get("resourceGroup")
                .and_then(Value::as_str)
                .unwrap_or("normal");
            let Some(bindings) = resource.get("bindings").and_then(Value::as_array) else {
                continue;
            };
            for binding in bindings {
                let meta = binding.get("objectMeta").unwrap_or(&Value::Null);
                let group = normalize_resource_group(
                    meta.get("sensitivity")
                        .and_then(Value::as_str)
                        .unwrap_or(resource_group),
                );
                if group != selected_group {
                    continue;
                }
                items.push(format!(
                    "{}. {} | resource={} | object={} | kind={} | level={}",
                    items.len() + 1,
                    display_name,
                    resource_id,
                    binding.get("objectId").and_then(Value::as_str).unwrap_or(""),
                    meta.get("objectKind").and_then(Value::as_str).unwrap_or("secret"),
                    group
                ));
            }
        }
    }
    if items.is_empty() {
        "No protected objects in this level yet.".to_string()
    } else {
        items.join("\n")
    }
}

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

pub(crate) fn save_resource_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let object_id = if core.get_object_id().trim().is_empty() {
        format!(
            "credential/{}/main/secret",
            sanitize_segment(core.get_provider_id().as_str())
        )
    } else {
        core.get_object_id().to_string()
    };
    let object_kind = if core.get_object_kind().trim().is_empty() {
        "secret".to_string()
    } else {
        core.get_object_kind().to_string()
    };
    let required_grid_count = core
        .get_required_correct_count()
        .parse::<u64>()
        .unwrap_or(12)
        .clamp(1, 24);
    let sensitivity = normalize_resource_group(core.get_resource_group().as_str());
    let catalog = json!({
        "version": "1",
        "resources": [{
            "resourceId": core.get_resource_id().to_string(),
            "resourceKind": core.get_resource_kind().to_string(),
            "providerId": core.get_provider_id().to_string(),
            "displayName": core.get_display_name().to_string(),
            "resourceGroup": sensitivity.clone(),
            "bindings": [
                {
                    "role": "protected_object",
                    "objectId": object_id,
                    "objectMeta": {
                        "objectKind": object_kind,
                        "encoding": "secret",
                        "sensitivity": sensitivity.clone(),
                        "requiredGridCount": required_grid_count,
                        "authorizationFrequency": core.get_authorization_frequency().to_string(),
                        "secretRef": core.get_secret_reference().to_string()
                    }
                }
            ]
        }]
    });
    fs::write(
        storylock_core_catalog_path(package_dir),
        serde_json::to_vec_pretty(&catalog)?,
    )?;
    core.set_resource_bindings(SharedString::from(format_bindings(
        catalog
            .get("resources")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .unwrap_or(&Value::Null),
    )));
    core.set_object_meta(SharedString::from(format_object_meta(
        catalog
            .get("resources")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .unwrap_or(&Value::Null),
    )));
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        core.get_resource_group().as_str(),
    )));
    Ok(())
}
