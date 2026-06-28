use super::*;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use uuid::Uuid;

fn temp_core_dir() -> PathBuf {
    std::env::temp_dir().join(format!("storylock_core_ui_test_{}", Uuid::new_v4()))
}

fn temp_identity_package_dir() -> PathBuf {
    temp_core_dir().join("identity-package")
}

fn ui_test_guard() -> MutexGuard<'static, ()> {
    static UI_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    UI_TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn initializes_storylock_core_package_files() {
    let root = temp_core_dir();
    let dir = root.join("identity-package");
    ensure_storylock_core_package(&dir).expect("init package");
    assert!(storylock_core_manifest_path(&dir).exists());
    assert!(storylock_core_catalog_path(&dir).exists());
    assert!(storylock_core_vault_path(&dir).exists());
    assert!(storylock_core_learning_policy_path(&dir).exists());
    let manifest = read_json_or_default(&storylock_core_manifest_path(&dir), Value::Null);
    assert!(manifest
        .get("files")
        .and_then(Value::as_array)
        .is_some_and(|files| files
            .iter()
            .any(|item| item.as_str() == Some("learning-policy.json"))));
    for required in [
        "../config/login-sites.json",
        "../config/signing-actions.json",
        "../config/agent-tasks.json",
    ] {
        assert!(manifest
            .get("files")
            .and_then(Value::as_array)
            .is_some_and(|files| files.iter().any(|item| item.as_str() == Some(required))));
        assert!(dir.join(required).exists(), "{required} should exist");
    }
    for template_id in [
        "shouzhudaitu-zh",
        "zhizi-yilin-zh",
        "emperor-new-clothes-en",
    ] {
        assert!(
            root.join("templates")
                .join(template_id)
                .join("story-template.json")
                .exists(),
            "{template_id} should exist"
        );
    }
}

#[test]
fn default_resource_catalog_covers_login_credentials_and_key_pairs() {
    let catalog = default_resource_catalog_json();
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .expect("default resources");
    assert!(resources.iter().any(|resource| {
        resource.get("resourceId").and_then(Value::as_str) == Some("github-main")
            && resource
                .get("bindings")
                .and_then(Value::as_array)
                .is_some_and(|bindings| {
                    bindings.iter().any(|binding| {
                        binding.get("role").and_then(Value::as_str) == Some("username")
                    }) && bindings.iter().any(|binding| {
                        binding.get("role").and_then(Value::as_str) == Some("password")
                            && binding
                                .get("objectMeta")
                                .and_then(|meta| meta.get("objectKind"))
                                .and_then(Value::as_str)
                                == Some("password")
                    })
                })
    }));
    assert!(resources.iter().any(|resource| {
        resource.get("resourceId").and_then(Value::as_str) == Some("wallet-main")
            && resource
                .get("bindings")
                .and_then(Value::as_array)
                .is_some_and(|bindings| {
                    bindings.iter().any(|binding| {
                        binding.get("role").and_then(Value::as_str) == Some("public_key")
                    }) && bindings.iter().any(|binding| {
                        binding.get("role").and_then(Value::as_str) == Some("private_key")
                            && binding
                                .get("objectMeta")
                                .and_then(|meta| meta.get("objectKind"))
                                .and_then(Value::as_str)
                                == Some("private_key")
                    })
                })
    }));

    let signing = default_signing_templates_json();
    let bindings = signing
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("bindings"))
        .and_then(Value::as_array)
        .expect("signing bindings");
    assert!(bindings.iter().any(|binding| {
        binding.get("fieldName").and_then(Value::as_str) == Some("publicKey")
            && binding.get("role").and_then(Value::as_str) == Some("public_key")
    }));
    assert!(bindings.iter().any(|binding| {
        binding.get("fieldName").and_then(Value::as_str) == Some("privateKey")
            && binding.get("role").and_then(Value::as_str) == Some("private_key")
    }));
}

#[test]
fn managed_object_editor_save_persists_uri_username_and_password() {
    let _guard = ui_test_guard();
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let core = StoryLockCoreApp::new().expect("core app");
    initialize_storylock_core_window(&core, &dir);

    core.set_display_name(SharedString::from("www.aliyun.com"));
    core.set_provider_id(SharedString::from("alice"));
    core.set_secret_reference(SharedString::from("secret-pass"));
    core.set_object_kind(SharedString::from("password_fill"));

    save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

    let catalog = read_json_or_default(&storylock_core_catalog_path(&dir), Value::Null);
    let resource = resource_by_id(&catalog, "www_aliyun_com").expect("saved resource");
    assert_eq!(
        resource.get("displayName").and_then(Value::as_str),
        Some("www.aliyun.com")
    );
    assert_eq!(read_username_for_resource(resource), "alice");
    assert_eq!(
        binding_secret_ref(resource, "password")
            .and_then(|secret_ref| read_stored_credential_field(secret_ref.as_str(), "password"))
            .as_deref(),
        Some("secret-pass")
    );
}

#[test]
fn protected_object_rows_show_username_for_password_fill_resources() {
    let _guard = ui_test_guard();
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let core = StoryLockCoreApp::new().expect("core app");

    core.set_display_name(SharedString::from("www.huawei.com"));
    core.set_provider_id(SharedString::from("bob"));
    core.set_secret_reference(SharedString::from("pw-123"));
    core.set_object_kind(SharedString::from("password_fill"));
    save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

    let catalog = read_json_or_default(&storylock_core_catalog_path(&dir), Value::Null);
    let rows = protected_object_rows(&catalog, "secret");
    let row = rows
        .iter()
        .find(|item| item.name == "www.huawei.com")
        .expect("row for saved site");
    assert_eq!(row.secret, "bob");
    assert_eq!(row.usage, "website");
}

#[test]
fn package_templates_dir_contains_three_story_template_directories() {
    let root = temp_core_dir();
    let dir = root.join("identity-package");
    ensure_storylock_core_package(&dir).expect("init package");
    for template_id in [
        "shouzhudaitu-zh",
        "zhizi-yilin-zh",
        "emperor-new-clothes-en",
    ] {
        let template = read_json_or_default(
            &root
                .join("templates")
                .join(template_id)
                .join("story-template.json"),
            Value::Null,
        );
        assert_eq!(
            template.get("templateId").and_then(Value::as_str),
            Some(template_id)
        );
        assert_eq!(
            template
                .get("nodes")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(24)
        );
        assert!(root
            .join("templates")
            .join(template_id)
            .join("README.md")
            .exists());
        assert!(!root
            .join("templates")
            .join(template_id)
            .join("vault.stlk")
            .exists());
    }
}

#[test]
fn story_template_summary_scans_user_added_template_directories() {
    let root = temp_core_dir();
    let dir = root.join("identity-package");
    ensure_storylock_core_package(&dir).expect("init package");
    let custom_dir = root.join("templates").join("custom-template");
    fs::create_dir_all(&custom_dir).expect("create custom template dir");
    let mut custom = default_author_draft_json();
    custom["templateId"] = json!("custom-template");
    custom["storyTitle"] = json!("Custom User Template");
    fs::write(
        custom_dir.join("story-template.json"),
        serde_json::to_vec_pretty(&custom).expect("serialize custom template"),
    )
    .expect("write custom template");

    let summary = format_story_draft_template_summary(&dir);

    assert!(summary.contains("templateId=custom-template"));
    assert!(summary.contains("storyTitle=Custom User Template"));
}

#[test]
fn selected_story_template_directory_loads_as_full_workspace() {
    let root = temp_core_dir();
    let dir = root.join("identity-package");
    ensure_storylock_core_package(&dir).expect("init package");
    let selected = root.join("templates").join("zhizi-yilin-zh");
    ensure_storylock_core_package(&selected).expect("load selected template directory");
    let effective = read_effective_author_draft(&selected);
    assert_eq!(
        effective.get("templateId").and_then(Value::as_str),
        Some("zhizi-yilin-zh")
    );
    assert_eq!(
        effective
            .get("nodes")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(24)
    );
    assert_eq!(
        effective
            .get("nodes")
            .and_then(Value::as_array)
            .and_then(|nodes| nodes.first())
            .and_then(|node| node.get("question"))
            .and_then(Value::as_str),
        Some("\u{6545}\u{4e8b}\u{53d1}\u{751f}\u{5728}\u{4ec0}\u{4e48}\u{65f6}\u{95f4}\u{ff1f}")
    );
    assert!(selected.join("learning-policy.json").exists());
    assert!(selected.join("templates").join("login-sites.json").exists());
}

#[test]
fn default_story_templates_include_useful_fables() {
    let templates = default_story_draft_templates_json();
    let items = templates
        .get("items")
        .and_then(Value::as_array)
        .expect("default story draft templates");
    assert!(items.len() >= 3);
    for expected in [
        "shouzhudaitu-zh",
        "zhizi-yilin-zh",
        "emperor-new-clothes-en",
    ] {
        assert!(items.iter().any(|item| {
            item.get("templateId").and_then(Value::as_str) == Some(expected)
                && item.get("nodes").and_then(Value::as_array).map(Vec::len) == Some(24)
        }));
    }
}

#[test]
fn storylock_startup_refreshes_mojibake_builtin_templates() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let mut vault = read_storylock_vault_payload(&dir);
    let mut broken = shouzhudaitu_author_draft_json();
    broken["summary"] = json!("é\u{009d}\u{0092}å¹´å\u{0086}\u{009c}å¤«");
    broken["nodes"][0]["question"] = json!("?å®\u{0088}æ \u{00aa}å¾\u{0085}å\u{0085}\u{0094}?");
    vault["storyDraftTemplates"] = json!({
        "schemaVersion": "storylock-story-draft-templates-v1",
        "defaultTemplateId": "shouzhudaitu-zh",
        "items": [broken]
    });
    save_storylock_vault_payload(&dir, vault).expect("save broken templates");

    ensure_storylock_core_package(&dir).expect("refresh package");
    let vault = read_storylock_vault_payload(&dir);
    let refreshed = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("templateId").and_then(Value::as_str) == Some("shouzhudaitu-zh")
            })
        })
        .expect("refreshed template");
    assert_eq!(
        refreshed.get("summary").and_then(Value::as_str),
        shouzhudaitu_author_draft_json()
            .get("summary")
            .and_then(Value::as_str)
    );
    assert_eq!(
        refreshed
            .get("nodes")
            .and_then(Value::as_array)
            .and_then(|nodes| nodes.first())
            .and_then(|node| node.get("question"))
            .and_then(Value::as_str),
        Some("\u{6545}\u{4e8b}\u{53d1}\u{751f}\u{5728}\u{4ec0}\u{4e48}\u{65f6}\u{95f4}\u{ff1f}")
    );
}

#[test]
fn host_story_candidate_converts_to_author_draft_template() {
    let candidate = json!({
        "candidateId": "story-template-test",
        "framework": {
            "title": "Host Candidate",
            "summary": "Candidate summary",
            "storyPlot": "Candidate plot",
            "memoryAnchors": ["anchor-one", "anchor-two"]
        }
    });
    let draft = story_draft_from_candidate(&candidate);
    assert_eq!(
        draft.get("templateId").and_then(Value::as_str),
        Some("story-template-test")
    );
    assert_eq!(
        draft.get("storyTitle").and_then(Value::as_str),
        Some("Host Candidate")
    );
    assert_eq!(
        draft.get("nodes").and_then(Value::as_array).map(Vec::len),
        Some(24)
    );
}

#[test]
fn pre_learning_plan_has_two_well_spaced_prompts_per_question() {
    let progress = LearningProgress::new();
    assert_eq!(progress.total_prompts(), 48);

    let mut positions_by_question = vec![Vec::new(); 24];
    for (position, question_index) in progress.plan.iter().copied().enumerate() {
        positions_by_question[question_index].push(position);
    }

    for positions in positions_by_question {
        assert_eq!(positions.len(), 2);
        assert!(positions[1] - positions[0] >= 12);
    }
}

#[test]
fn export_preview_is_redacted() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let preview = build_export_preview(&dir);
    assert!(preview.contains("permissionObjects=2"));
    assert!(preview.contains("preflight=OK"));
    assert!(preview.contains("learning-policy.json"));
    assert!(preview.contains("StoryLock UI internal export preview only"));
    assert!(preview.contains("Yian Host reads learning-policy.json"));
    assert!(!preview.contains("signingKeyBytes="));
    assert!(!preview.contains("privateKey="));
    assert!(!preview.contains("password="));
}

#[test]
fn effective_author_draft_prefers_pending_temp_file() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let mut pending = read_effective_author_draft(&dir);
    pending["storyTitle"] = json!("pending temp title");
    write_pending_author_draft(&dir, &pending).expect("write pending draft");
    let effective = read_effective_author_draft(&dir);
    assert_eq!(
        effective.get("storyTitle").and_then(Value::as_str),
        Some("pending temp title")
    );
}

#[test]
fn story_draft_template_uses_author_draft_schema_and_restores_ui() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let mut draft = read_effective_author_draft(&dir);
    draft["templateId"] = json!("template-unified-title");
    draft["storyTitle"] = json!("Template Unified Title");
    draft["summary"] = json!("Template unified summary");
    draft["storyPlot"] = json!("Template unified plot detail");
    draft["nodes"][0]["question"] = json!("Unified question one?");
    normalize_author_draft_schema(&mut draft);
    let mut vault = read_storylock_vault_payload(&dir);
    vault["storyDraftTemplates"] = story_draft_templates_from_draft(&draft);
    save_storylock_vault_payload(&dir, vault).expect("save story draft template");

    let vault = read_storylock_vault(&dir);
    let template = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .expect("story draft template");
    assert_eq!(
        template.get("storyTitle").and_then(Value::as_str),
        Some("Template Unified Title")
    );
    assert_eq!(
        template.get("summary").and_then(Value::as_str),
        Some("Template unified summary")
    );
    assert_eq!(
        template.get("storyPlot").and_then(Value::as_str),
        Some("Template unified plot detail")
    );
    assert_eq!(
        template
            .get("nodes")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(24)
    );

    let mut pending = read_effective_author_draft(&dir);
    pending["storyTitle"] = json!("Different pending title");
    write_pending_author_draft(&dir, &pending).expect("write different pending");
    let mut vault = read_storylock_vault_payload(&dir);
    let restored = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .expect("template draft");
    vault["pendingAuthorDraft"] = restored;
    save_storylock_vault_payload(&dir, vault).expect("restore template as pending");
    let effective = read_effective_author_draft(&dir);
    assert_eq!(
        effective.get("storyTitle").and_then(Value::as_str),
        Some("Template Unified Title")
    );
    assert_eq!(
        effective
            .get("nodes")
            .and_then(Value::as_array)
            .and_then(|nodes| nodes.first())
            .and_then(|node| node.get("question"))
            .and_then(Value::as_str),
        Some("Unified question one?")
    );
}

#[test]
fn apply_template_uses_requested_template_id() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let mut vault = read_storylock_vault_payload(&dir);
    merge_builtin_story_draft_templates(&mut vault);
    let effective = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("templateId").and_then(Value::as_str) == Some("zhizi-yilin-zh")
            })
        })
        .cloned()
        .expect("selected template");
    assert_eq!(
        effective.get("templateId").and_then(Value::as_str),
        Some("zhizi-yilin-zh")
    );
    assert_eq!(
        effective.get("storyTitle").and_then(Value::as_str),
        Some("智子疑邻")
    );
    assert_eq!(
        effective
            .get("nodes")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(24)
    );
    assert_eq!(
        effective
            .get("nodes")
            .and_then(Value::as_array)
            .and_then(|nodes| nodes.first())
            .and_then(|node| node.get("question"))
            .and_then(Value::as_str),
        Some("\u{6545}\u{4e8b}\u{53d1}\u{751f}\u{5728}\u{4ec0}\u{4e48}\u{65f6}\u{95f4}\u{ff1f}")
    );
}

#[test]
fn export_promotes_and_clears_pending_temp_draft() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let mut pending = read_effective_author_draft(&dir);
    pending["storyTitle"] = json!("promoted title");
    write_pending_author_draft(&dir, &pending).expect("write pending draft");

    let export_dir = export_storylock_package(&dir).expect("export package");
    let vault = read_storylock_vault(&dir);
    assert_eq!(
        vault
            .get("authorDraft")
            .and_then(|draft| draft.get("storyTitle"))
            .and_then(Value::as_str),
        Some("promoted title")
    );
    assert!(vault
        .get("pendingAuthorDraft")
        .map(|value| value.is_null())
        .unwrap_or(true));
    assert!(export_dir.join("vault.stlk").exists());
    assert!(export_dir.join("learning-policy.json").exists());
}

#[test]
fn learning_policy_is_host_readable_and_blocks_invalid_values() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let policy = read_learning_policy(&dir);
    assert_eq!(
        policy
            .get("retentionLearning")
            .and_then(|value| value.get("questionCount"))
            .and_then(Value::as_i64),
        Some(22)
    );
    assert!(host_learning_plan_status(&dir).contains("Learning plan:"));

    let mut broken = policy;
    broken["preLearning"]["totalPrompts"] = json!(47);
    write_learning_policy(&dir, &broken).expect("write broken policy");
    let result = preflight_storylock_core_package(&dir);
    assert!(result
        .errors
        .iter()
        .any(|issue| issue.code == "SL_LEARNING_POLICY_INVALID"));
}

#[test]
fn template_bundle_summary_covers_three_template_files() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let summary = format_all_template_bundles(&dir);
    assert!(summary.contains("login-sites.json"));
    assert!(summary.contains("signing-actions.json"));
    assert!(summary.contains("agent-tasks.json"));
    assert!(summary.contains("username -> username"));
    assert!(!summary.contains("password="));
    assert!(!summary.contains("privateKey="));
}

#[test]
fn preflight_reports_invalid_template_role() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let mut vault = read_storylock_vault(&dir);
    vault["templates"]["agentTasks"] = json!({
        "version": "1",
        "templateType": "agent-tasks",
        "items": [{
            "templateId": "broken-agent",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "missing", "role": "missing_role" }
            ]
        }]
    });
    write_storylock_vault(&dir, &vault).expect("write broken template");
    let result = preflight_storylock_core_package(&dir);
    assert!(result
        .errors
        .iter()
        .any(|issue| issue.code == "SL_TEMPLATE_UNKNOWN_ROLE"));
    let preview = build_export_preview(&dir);
    assert!(preview.contains("preflight=FAILED"));
    assert!(preview.contains("SL_TEMPLATE_UNKNOWN_ROLE"));
}

#[test]
fn writes_all_twenty_four_node_slots() {
    let _guard = ui_test_guard();
    let mut draft = default_author_draft_json();
    let fake_core = StoryLockCoreApp::new().expect("core app");
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    fake_core.set_active_page(0);
    fake_core.set_overview_selection_enabled(true);
    initialize_storylock_core_window(&fake_core, &dir);
    assert_eq!(fake_core.get_active_page(), 1);
    assert!(!fake_core.get_overview_selection_enabled());
    assert_eq!(fake_core.get_template_id().as_str(), "shouzhudaitu-zh");

    fake_core.set_story_summary(SharedString::from("saved summary marker"));
    save_temp_draft_from_window(&fake_core, &dir).expect("save draft");
    let effective = read_effective_author_draft(&dir);
    assert_eq!(
        effective.get("templateId").and_then(Value::as_str),
        Some("shouzhudaitu-zh")
    );
    assert_eq!(
        effective.get("summary").and_then(Value::as_str),
        Some("saved summary marker")
    );

    fake_core.set_node_index(23);
    fake_core.set_node_id(SharedString::from("node-24-custom"));
    fake_core.set_node_title(SharedString::from("Custom Node 24"));
    fake_core.set_element_id(SharedString::from("ending"));
    fake_core.set_question_text(SharedString::from("Custom question 24?"));
    fake_core.set_selection_mode(SharedString::from("multi_select"));
    fake_core.set_correct_count(SharedString::from("3"));
    fake_core.set_candidate_pool_size(SharedString::from("9"));
    fake_core.set_recall_priority(SharedString::from("high"));
    fake_core.set_verify_policy(SharedString::from("caseInsensitive + trim"));
    fake_core.set_editor_notes(SharedString::from("local only"));
    fake_core.set_canonical_answer(SharedString::from("local answer"));
    fake_core.set_accepted_answers(SharedString::from("local answer; answer alt"));
    fake_core.set_correct_options(SharedString::from("A; B; C"));
    fake_core.set_answer_options(SharedString::from(
        "1. A | correct\n2. B | correct\n3. C | correct\n4. D | wrong\n5. E | wrong\n6. F | wrong\n7. G | wrong\n8. H | wrong\n9. I | wrong",
    ));
    write_current_node_to_draft(&fake_core, &mut draft);
    let nodes = draft.get("nodes").and_then(Value::as_array).expect("nodes");
    assert_eq!(nodes.len(), 24);
    assert_eq!(
        nodes[23].get("nodeId").and_then(Value::as_str),
        Some("node-24-custom")
    );
    assert_eq!(
        nodes[23].get("question").and_then(Value::as_str),
        Some("Custom question 24?")
    );

    let template_dir = dir
        .parent()
        .expect("runtime root")
        .join("templates")
        .join("shouzhudaitu-zh");
    ensure_storylock_core_package(&template_dir).expect("load template directory");
    initialize_storylock_core_window(&fake_core, &template_dir);
    fake_core.set_story_summary(SharedString::from("plain template saved marker"));
    save_temp_draft_from_window(&fake_core, &template_dir).expect("save template directory draft");
    let plain_template =
        read_json_or_default(&template_dir.join("story-template.json"), Value::Null);
    assert_eq!(
        plain_template.get("summary").and_then(Value::as_str),
        Some("plain template saved marker")
    );
    assert!(!template_dir.join("story-drafts").exists());
}

#[test]
fn default_author_draft_has_twenty_four_nodes() {
    let draft = default_author_draft_json();
    assert_eq!(
        draft.get("nodes").and_then(Value::as_array).map(Vec::len),
        Some(24)
    );
}

#[test]
fn storylock_ui_settings_round_trip_language_and_paths() {
    let dir = temp_identity_package_dir();
    let settings_path = dir.join("config").join("config.json");
    let core_dir = dir.join("identity-package");
    let export_dir = dir.join("exports");
    let settings = StoryLockUiSettings {
        language: Some(String::from("en")),
        core_data_dir: Some(core_dir.display().to_string()),
        export_package_dir: Some(export_dir.display().to_string()),
    };

    write_storylock_ui_settings(&settings_path, &settings).expect("write settings");
    let loaded = read_storylock_ui_settings(&settings_path).expect("read settings");

    assert_eq!(loaded, settings);
    assert_eq!(initial_storylock_core_package_dir(&loaded), core_dir);
}
