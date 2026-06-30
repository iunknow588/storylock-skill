use super::*;

#[derive(Clone, Copy)]
pub(crate) struct TemplateBindingSpec {
    pub(crate) field_name: &'static str,
    pub(crate) role: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct TemplateChildSpec {
    pub(crate) child_key: &'static str,
    pub(crate) bundle_key: &'static str,
    pub(crate) file_name: &'static str,
    pub(crate) suffix: &'static str,
    pub(crate) fallback_bundle: fn() -> Value,
    pub(crate) bindings: &'static [TemplateBindingSpec],
    pub(crate) enabled_for_usage: fn(&str) -> bool,
}

const LOGIN_SITE_BINDINGS: [TemplateBindingSpec; 2] = [
    TemplateBindingSpec {
        field_name: "username",
        role: "username",
    },
    TemplateBindingSpec {
        field_name: "password",
        role: "password",
    },
];

const SIGNING_ACTION_BINDINGS: [TemplateBindingSpec; 2] = [
    TemplateBindingSpec {
        field_name: "publicKey",
        role: "public_key",
    },
    TemplateBindingSpec {
        field_name: "privateKey",
        role: "private_key",
    },
];

const AGENT_TASK_BINDINGS: [TemplateBindingSpec; 1] = [TemplateBindingSpec {
    field_name: "username",
    role: "username",
}];

pub(crate) const TEMPLATE_CHILD_SPECS: [TemplateChildSpec; 3] = [
    TemplateChildSpec {
        child_key: "loginSite",
        bundle_key: "loginSites",
        file_name: "login-sites.json",
        suffix: "login",
        fallback_bundle: default_login_templates_json,
        bindings: &LOGIN_SITE_BINDINGS,
        enabled_for_usage: |usage| usage == "password_fill",
    },
    TemplateChildSpec {
        child_key: "signingAction",
        bundle_key: "signingActions",
        file_name: "signing-actions.json",
        suffix: "sign",
        fallback_bundle: default_signing_templates_json,
        bindings: &SIGNING_ACTION_BINDINGS,
        enabled_for_usage: |usage| usage == "sign",
    },
    TemplateChildSpec {
        child_key: "agentTask",
        bundle_key: "agentTasks",
        file_name: "agent-tasks.json",
        suffix: "agent",
        fallback_bundle: default_agent_templates_json,
        bindings: &AGENT_TASK_BINDINGS,
        enabled_for_usage: |usage| usage == "password_fill",
    },
];

pub(crate) fn template_child_spec_for_bundle(
    bundle_key: &str,
) -> Option<&'static TemplateChildSpec> {
    TEMPLATE_CHILD_SPECS
        .iter()
        .find(|spec| spec.bundle_key == bundle_key)
}

pub(crate) fn template_shell_id_for_resource(resource_id: &str) -> String {
    format!("template-shell/{}", sanitize_segment(resource_id))
}

pub(crate) fn template_child_template_id(resource_id: &str, spec: &TemplateChildSpec) -> String {
    format!("{}-{}", sanitize_segment(resource_id), spec.suffix)
}

pub(crate) fn template_shell_from_resource(
    resource_id: &str,
    display_name: &str,
    usage: &str,
) -> Value {
    let shell_id = template_shell_id_for_resource(resource_id);
    let children = TEMPLATE_CHILD_SPECS
        .iter()
        .map(|spec| {
            json!({
                "childKey": spec.child_key,
                "bundleKey": spec.bundle_key,
                "fileName": spec.file_name,
                "templateId": template_child_template_id(resource_id, spec),
                "resourceId": resource_id,
                "enabled": (spec.enabled_for_usage)(usage)
            })
        })
        .collect::<Vec<_>>();
    json!({
        "shellId": shell_id,
        "displayName": display_name,
        "children": children
    })
}

pub(crate) fn default_template_item_id(
    resource_id: &str,
    spec: &TemplateChildSpec,
    templates: &Value,
) -> String {
    existing_bundle_items(templates, spec.bundle_key)
        .into_iter()
        .find(|item| item.get("resourceId").and_then(Value::as_str) == Some(resource_id))
        .and_then(|item| {
            item.get("templateId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| template_child_template_id(resource_id, spec))
}

pub(crate) fn existing_bundle_items(templates: &Value, bundle_key: &str) -> Vec<Value> {
    templates
        .get(bundle_key)
        .and_then(|bundle| bundle.get("items"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| {
            template_bundle_fallback(bundle_key)
                .get("items")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        })
}

pub(crate) fn template_bundle_fallback(bundle_key: &str) -> Value {
    template_child_spec_for_bundle(bundle_key)
        .map(|spec| (spec.fallback_bundle)())
        .unwrap_or_else(|| {
            json!({
                "version": "1",
                "templateType": "unknown",
                "items": []
            })
        })
}

pub(crate) fn upsert_template_bundle_item(
    templates: &mut Value,
    bundle_key: &str,
    item: Value,
    enabled: bool,
) {
    let fallback = template_bundle_fallback(bundle_key);
    let mut bundle = templates
        .get(bundle_key)
        .cloned()
        .unwrap_or_else(|| fallback.clone());
    if !bundle.is_object() {
        bundle = fallback.clone();
    }
    bundle["version"] = fallback
        .get("version")
        .cloned()
        .unwrap_or_else(|| json!("1"));
    bundle["templateType"] = fallback
        .get("templateType")
        .cloned()
        .unwrap_or_else(|| json!(bundle_key));

    let resource_id = item
        .get("resourceId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let template_id = item
        .get("templateId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let parent_shell_id = item
        .get("parentShellId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let mut items = bundle
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    items.retain(|existing| {
        existing.get("resourceId").and_then(Value::as_str) != Some(resource_id.as_str())
            && existing.get("templateId").and_then(Value::as_str) != Some(template_id.as_str())
            && existing.get("parentShellId").and_then(Value::as_str)
                != Some(parent_shell_id.as_str())
    });
    if enabled {
        items.push(item);
    }
    bundle["items"] = Value::Array(items);
    templates[bundle_key] = bundle;
}

pub(crate) fn template_bundle_item_from_resource(
    spec: &TemplateChildSpec,
    resource_id: &str,
    display_name: &str,
    parent_shell_id: &str,
) -> Value {
    let template_id = template_child_template_id(resource_id, spec);
    let bindings = spec
        .bindings
        .iter()
        .map(|binding| {
            json!({
                "fieldName": binding.field_name,
                "role": binding.role
            })
        })
        .collect::<Vec<_>>();
    json!({
        "templateId": template_id,
        "displayName": display_name,
        "resourceId": resource_id,
        "parentShellId": parent_shell_id,
        "bindings": bindings
    })
}

pub(crate) fn sync_template_children_for_resource(
    templates: &mut Value,
    resource_id: &str,
    display_name: &str,
    usage: &str,
) {
    let shell_id = template_shell_id_for_resource(resource_id);
    for spec in TEMPLATE_CHILD_SPECS {
        let template_id = default_template_item_id(resource_id, &spec, templates);
        let mut item =
            template_bundle_item_from_resource(&spec, resource_id, display_name, shell_id.as_str());
        item["templateId"] = json!(template_id);
        upsert_template_bundle_item(
            templates,
            spec.bundle_key,
            item,
            (spec.enabled_for_usage)(usage),
        );
    }
}

pub(crate) fn remove_template_child_records_for_resource(
    package_dir: &Path,
    resource: &Value,
) -> Result<()> {
    let resource_id = resource
        .get("resourceId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if resource_id.trim().is_empty() {
        return Ok(());
    }

    let shell_id = resource
        .get("templateShell")
        .and_then(|shell| shell.get("shellId"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| template_shell_id_for_resource(resource_id.as_str()));

    let mut vault = read_storylock_vault_payload(package_dir);
    let mut templates = storylock_templates_from_vault(&vault);
    for spec in TEMPLATE_CHILD_SPECS {
        let fallback = template_bundle_fallback(spec.bundle_key);
        let mut bundle = templates
            .get(spec.bundle_key)
            .cloned()
            .unwrap_or_else(|| fallback.clone());
        if !bundle.is_object() {
            bundle = fallback.clone();
        }
        let mut items = bundle
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        items.retain(|item| {
            item.get("resourceId").and_then(Value::as_str) != Some(resource_id.as_str())
                && item.get("parentShellId").and_then(Value::as_str) != Some(shell_id.as_str())
        });
        bundle["items"] = Value::Array(items);
        templates[spec.bundle_key] = bundle;
    }
    vault["templates"] = templates;
    save_storylock_vault_payload(package_dir, vault)?;
    ensure_storylock_template_files(package_dir)
}
