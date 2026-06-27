use super::*;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_core_dir() -> PathBuf {
    std::env::temp_dir().join(format!("storylock_core_ui_test_{}", Uuid::new_v4()))
}

#[test]
fn initializes_storylock_core_package_files() {
    let dir = temp_core_dir();
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
        "story-drafts/manifest.json",
        "story-drafts/shouzhudaitu-zh.json",
        "story-drafts/zhizi-yilin-zh.json",
        "story-drafts/emperor-new-clothes-en.json",
        "templates/story-template-directories/shouzhudaitu-zh/story-template.json",
        "templates/story-template-directories/zhizi-yilin-zh/story-template.json",
        "templates/story-template-directories/emperor-new-clothes-en/story-template.json",
        "templates/story-draft-templates.json",
    ] {
        assert!(manifest
            .get("files")
            .and_then(Value::as_array)
            .is_some_and(|files| files.iter().any(|item| item.as_str() == Some(required))));
        assert!(dir.join(required).exists(), "{required} should exist");
    }
}

#[test]
fn package_templates_dir_contains_three_story_template_directories() {
    let dir = temp_core_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    for template_id in [
        "shouzhudaitu-zh",
        "zhizi-yilin-zh",
        "emperor-new-clothes-en",
    ] {
        let template = read_json_or_default(
            &dir.join("templates")
                .join("story-template-directories")
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
        for required in [
            "package-manifest.json",
            "resource-catalog.json",
            "learning-policy.json",
            "vault.stlk",
            "templates/login-sites.json",
            "templates/signing-actions.json",
            "templates/agent-tasks.json",
            "story-drafts/manifest.json",
            "story-drafts/current-story-template.json",
        ] {
            assert!(
                dir.join("templates")
                    .join("story-template-directories")
                    .join(template_id)
                    .join(required)
                    .exists(),
                "{template_id}/{required} should exist"
            );
        }
    }
}

#[test]
fn selected_story_template_directory_loads_as_full_workspace() {
    let dir = temp_core_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let selected = dir
        .join("templates")
        .join("story-template-directories")
        .join("zhizi-yilin-zh");
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
        Some("《智子疑邻》故事具体发生在什么时间？")
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
    let dir = temp_core_dir();
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
        Some("\u{300a}\u{5b88}\u{682a}\u{5f85}\u{5154}\u{300b}\u{6545}\u{4e8b}\u{5177}\u{4f53}\u{53d1}\u{751f}\u{5728}\u{4ec0}\u{4e48}\u{65f6}\u{95f4}\u{ff1f}")
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
    let dir = temp_core_dir();
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
    let dir = temp_core_dir();
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
    let dir = temp_core_dir();
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
    let dir = temp_core_dir();
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
        Some("《智子疑邻》故事具体发生在什么时间？")
    );
}

#[test]
fn export_promotes_and_clears_pending_temp_draft() {
    let dir = temp_core_dir();
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
    let dir = temp_core_dir();
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
    let dir = temp_core_dir();
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
    let dir = temp_core_dir();
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
    let mut draft = default_author_draft_json();
    let fake_core = StoryLockCoreApp::new().expect("core app");
    let dir = temp_core_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    fake_core.set_active_page(0);
    fake_core.set_overview_selection_enabled(true);
    initialize_storylock_core_window(&fake_core, &dir);
    assert_eq!(fake_core.get_active_page(), 1);
    assert!(!fake_core.get_overview_selection_enabled());

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
}

#[test]
fn default_author_draft_has_twenty_four_nodes() {
    let draft = default_author_draft_json();
    assert_eq!(
        draft.get("nodes").and_then(Value::as_array).map(Vec::len),
        Some(24)
    );
}
