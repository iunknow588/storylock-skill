use super::*;

pub(crate) fn initialize_storylock_core_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let vault = read_storylock_vault_payload(package_dir);
    let draft = storylock_author_draft_from_vault(&vault);
    let templates = storylock_templates_from_vault(&vault);
    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    core.set_core_data_dir(SharedString::from(package_dir.display().to_string()));
    core.set_draft_file_path(SharedString::from("vault.stlk"));
    core.set_manifest_file_path(SharedString::from("package-manifest.json"));
    core.set_encrypted_vault_path(SharedString::from("vault.stlk"));
    core.set_resource_catalog_path(SharedString::from("resource-catalog.json"));
    core.set_learning_policy_path(SharedString::from("learning-policy.json"));
    core.set_export_package_dir(SharedString::from(
        default_storylock_export_dir(package_dir)
            .display()
            .to_string(),
    ));
    core.set_story_title(json_string(&draft, &["storyTitle"]));
    core.set_story_summary(json_string(&draft, &["summary"]));
    core.set_story_plot(json_string(&draft, &["storyPlot"]));
    core.set_memory_anchors(SharedString::from(
        draft
            .get("memoryAnchors")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .unwrap_or_default(),
    ));
    core.set_element_group(SharedString::from(
        draft
            .get("elementGroups")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default(),
    ));
    load_node_into_window(core, package_dir, core.get_node_index());
    if let Some(resource) = catalog
        .get("resources")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
    {
        core.set_resource_id(json_string(resource, &["resourceId"]));
        core.set_resource_kind(json_string(resource, &["resourceKind"]));
        core.set_provider_id(json_string(resource, &["providerId"]));
        core.set_display_name(json_string(resource, &["displayName"]));
        core.set_resource_group(resource_group_from_catalog_resource(resource));
        core.set_resource_bindings(SharedString::from(format_bindings(resource)));
        core.set_object_meta(SharedString::from(format_object_meta(resource)));
    }
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        core.get_resource_group().as_str(),
    )));
    core.set_template_display_name(json_string(&draft, &["storyTitle"]));
    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(
        package_dir,
    )));
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    core.set_candidate_template_status(SharedString::from(
        "Host can queue candidates; StoryLock must pull them explicitly.",
    ));
    core.set_learning_result(SharedString::from(
        "Run the pre-export test after finishing edits and policy changes.",
    ));
    core.set_learning_status(SharedString::from(
        "Pre-export test idle. Export stays blocked until the test passes.",
    ));
    core.set_export_ready(false);
    load_learning_policy_into_window(core, package_dir);
    core.set_config_status(SharedString::from(format!(
        "StoryLock Core package ready at {}",
        package_dir.display()
    )));
    if templates
        .get("agentTasks")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
        .is_some_and(|items| !items.is_empty())
    {
        core.set_active_page(0);
    }
}
