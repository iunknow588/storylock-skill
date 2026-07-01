use super::*;
use crate::host_runtime::resolve_data_dir;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{mpsc, OnceLock};
use std::thread;
use uuid::Uuid;

fn temp_core_dir() -> PathBuf {
    std::env::temp_dir().join(format!("storylock_core_ui_test_{}", Uuid::new_v4()))
}

fn temp_identity_package_dir() -> PathBuf {
    temp_core_dir().join("identity-package")
}

struct UiTestRequest {
    job: Box<dyn FnOnce() + Send + 'static>,
    done: mpsc::Sender<thread::Result<()>>,
}

fn run_ui_test(test: impl FnOnce() + Send + 'static) {
    static UI_TEST_WORKER: OnceLock<mpsc::Sender<UiTestRequest>> = OnceLock::new();
    let worker = UI_TEST_WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<UiTestRequest>();
        thread::Builder::new()
            .name("storylock-ui-test".to_string())
            .spawn(move || {
                while let Ok(request) = rx.recv() {
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        (request.job)();
                    }));
                    let _ = request.done.send(result);
                }
            })
            .expect("spawn UI test worker");
        tx
    });
    let (done_tx, done_rx) = mpsc::channel();
    worker
        .send(UiTestRequest {
            job: Box::new(test),
            done: done_tx,
        })
        .expect("send UI test job");
    match done_rx.recv().expect("receive UI test result") {
        Ok(()) => {}
        Err(payload) => resume_unwind(payload),
    }
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
    let policy_catalog = read_json_or_default(&storylock_core_catalog_path(&dir), Value::Null);
    assert!(policy_catalog
        .get("resources")
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty));
    assert!(policy_catalog
        .get("operationTemplates")
        .and_then(Value::as_array)
        .is_some());
    assert!(resource_by_id(&read_protected_resources(&dir), "github-main").is_some());
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
fn default_policy_catalog_has_no_user_protected_objects() {
    let catalog = default_resource_catalog_json();
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .expect("default resources");
    assert!(resources.is_empty());
    assert!(catalog
        .get("accessLevels")
        .and_then(Value::as_object)
        .is_some());
    assert!(catalog
        .get("operationTemplates")
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(|item| {
            item.get("operation").and_then(Value::as_str) == Some("policy_modify")
                && item.get("requiredCells").and_then(Value::as_u64) == Some(22)
        })));
}

#[test]
fn host_storylock_open_uses_twenty_two_question_authorization() {
    let dashboard_source = include_str!("../dashboard.rs");
    let host_source = include_str!("../host_dashboard.slint");
    let policy_source = include_str!("../../host_runtime/request_support/authorization.rs");
    let verification_source = include_str!("../../host_runtime/local_core/verification.rs");

    assert!(host_source.contains("StoryLockAuthorizationDialog"));
    assert!(host_source.contains("SideActionButton"));
    assert!(host_source.contains("Remote Web"));
    assert!(host_source.contains("StoryLock Core"));
    assert!(host_source.contains("Observation"));
    assert!(host_source.contains("browse-storylock-data-dir"));
    assert!(host_source.contains("LearningAnswerGrid"));
    assert!(host_source.contains("callback select-answer(int);"));
    assert!(host_source.contains("toggle-answer(index) => { root.select-answer(index); }"));
    assert!(host_source.contains("in property <bool> can-previous: current-index > 0;"));
    assert!(
        host_source.contains("in property <bool> can-next: current-index + 1 < challenge-count;")
    );
    assert!(host_source.contains("enabled: root.can-next;"));
    assert!(!host_source.contains("component ChallengeAnswerTile"));
    assert!(!host_source.contains("component ChallengeAnswerGrid"));
    assert!(!host_source.contains("AuthQuestionRow"));
    assert!(dashboard_source.contains("begin_storylock_open_authorization"));
    assert!(dashboard_source.contains("create_storylock_open_challenge(&active_package_dir, 22)"));
    assert!(dashboard_source.contains("read_effective_author_draft(package_dir)"));
    assert!(dashboard_source.contains("set_storylock_challenge_question"));
    assert!(dashboard_source.contains("if current_index_for_next.get() >= max_index"));
    assert!(dashboard_source.contains("set_option_1_state"));
    assert!(dashboard_source.contains("vec![Vec::<String>::new(); cells.len()]"));
    assert!(dashboard_source.contains("selected != expected"));
    assert!(dashboard_source.contains("show_storylock_authorization_result"));
    assert!(dashboard_source.contains("挑战通过，即将进入 StoryLock 编辑界面。"));
    assert!(dashboard_source.contains("挑战失败：{error}"));
    assert!(dashboard_source.contains("第 {} 题不匹配，已选 {}/应选 {}"));
    assert!(dashboard_source.contains("selection.clear();"));
    assert!(dashboard_source.contains("current_index_for_auth.set(0);"));
    assert!(!dashboard_source.contains("create_storylock_open_verification"));
    assert!(verification_source.contains("\"answerOptions\": cell.answer_options"));
    assert!(policy_source.contains("required_cells: 22"));
}

#[test]
fn storylock_open_challenge_uses_selected_story_draft_not_host_question_bank() {
    let root = temp_core_dir();
    let package_dir = root.join("templates").join("shouzhudaitu-zh");
    ensure_storylock_core_package(&package_dir).expect("init selected story package");

    let cells = super::super::dashboard::create_storylock_open_challenge(&package_dir, 22)
        .expect("create storylock challenge");

    assert_eq!(cells.len(), 22);
    assert!(cells.iter().all(|cell| cell.answer_options.len() == 9));
    assert!(cells.iter().all(|cell| !cell.expected_answers.is_empty()));
    assert!(cells.iter().any(|cell| {
        cell.prompt_text.contains('\u{6545}') || cell.prompt_text.contains('\u{4ec0}')
    }));
    assert!(cells
        .iter()
        .flat_map(|cell| cell.answer_options.iter())
        .any(|option| option.contains('\u{5b8b}') || option.contains('\u{7530}')));
    assert!(!cells.iter().any(|cell| {
        cell.prompt_text.contains("Object:")
            || cell.prompt_text.contains("Strength:")
            || cell.prompt_text.contains("Which memory")
    }));

    let all_correct_answers = cells
        .iter()
        .map(|cell| cell.expected_answers.clone())
        .collect::<Vec<_>>();
    super::super::dashboard::authorize_storylock_open(&cells, &all_correct_answers)
        .expect("all correct story draft answers authorize");

    assert!(
        cells.iter().any(|cell| cell.expected_count > 1),
        "bundled StoryLock challenges must exercise multi-select authorization"
    );
    let mut ui_click_answers = vec![Vec::<String>::new(); cells.len()];
    for (cell_index, cell) in cells.iter().enumerate() {
        for expected in &cell.expected_answers {
            let option_index = cell
                .answer_options
                .iter()
                .position(|option| {
                    super::super::dashboard::normalize_challenge_answer(option)
                        == super::super::dashboard::normalize_challenge_answer(expected)
                })
                .expect("expected answer is present in the visible answer grid");
            super::super::dashboard::toggle_storylock_challenge_selection(
                &cells,
                &mut ui_click_answers,
                cell_index,
                option_index,
            );
        }
    }
    super::super::dashboard::authorize_storylock_open(&cells, &ui_click_answers)
        .expect("selecting each visible correct grid option authorizes");

    let mut twenty_four_answers = all_correct_answers.clone();
    twenty_four_answers.push(vec!["extra answer 23".to_string()]);
    twenty_four_answers.push(vec!["extra answer 24".to_string()]);
    assert!(
        super::super::dashboard::authorize_storylock_open(&cells, &twenty_four_answers).is_err(),
        "StoryLock open authorization must compare exactly the 22 challenged cells, not accept a 24-cell answer set"
    );

    let twenty_one_answers = all_correct_answers
        .iter()
        .take(21)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        super::super::dashboard::authorize_storylock_open(&cells, &twenty_one_answers).is_err(),
        "StoryLock open authorization must reject truncated answer sets instead of zip-truncating"
    );

    let partial_answers = cells
        .iter()
        .map(|cell| vec![cell.expected_answers[0].clone()])
        .collect::<Vec<_>>();
    assert!(super::super::dashboard::authorize_storylock_open(&cells, &partial_answers).is_err());

    let mut wrong_answers = all_correct_answers;
    wrong_answers[0].push("definitely wrong".to_string());
    assert!(super::super::dashboard::authorize_storylock_open(&cells, &wrong_answers).is_err());
}

#[test]
fn default_protected_resources_live_in_encrypted_vault_shape() {
    let catalog = default_protected_resources_catalog_json();
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .expect("default protected resources");
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
    run_ui_test(move || {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);

        core.set_resource_id(SharedString::from(""));
        core.set_display_name(SharedString::from("www.aliyun.com"));
        core.set_provider_id(SharedString::from("alice"));
        core.set_secret_reference(SharedString::from("secret-pass"));
        core.set_object_kind(SharedString::from("password_fill"));

        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

        let catalog = read_protected_resources(&dir);
        let resource = resource_by_id(&catalog, "www_aliyun_com").expect("saved resource");
        assert_eq!(
            resource.get("displayName").and_then(Value::as_str),
            Some("www.aliyun.com")
        );
        assert_eq!(read_username_for_resource(resource), "alice");
        assert_eq!(
            binding_secret_ref(resource, "password")
                .and_then(|secret_ref| read_stored_credential_field(
                    secret_ref.as_str(),
                    "password"
                ))
                .as_deref(),
            Some("secret-pass")
        );

        let policy_catalog = read_json_or_default(&storylock_core_catalog_path(&dir), Value::Null);
        assert!(
            policy_catalog
                .get("resources")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty),
            "resource-catalog.json must not persist protected objects in plaintext"
        );
        assert!(resource_by_id(&read_protected_resources(&dir), "www_aliyun_com").is_some());
    });
}

#[test]
fn managed_object_editor_save_updates_selected_existing_resource() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);

        core.set_resource_id(SharedString::from("github-main"));
        core.set_display_name(SharedString::from("https://github.com/new-owner/new-login"));
        core.set_provider_id(SharedString::from("new-user"));
        core.set_secret_reference(SharedString::from("new-pass"));
        core.set_object_kind(SharedString::from("password_fill"));

        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

        let catalog = read_protected_resources(&dir);
        assert_eq!(
            catalog
                .get("resources")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        let resource = resource_by_id(&catalog, "github-main").expect("existing resource");
        assert_eq!(
            resource.get("displayName").and_then(Value::as_str),
            Some("https://github.com/new-owner/new-login")
        );
        assert_eq!(read_username_for_resource(resource), "new-user");
    });
}

#[test]
fn managed_object_editor_save_preserves_signing_resource_kind() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_display_name(SharedString::from("wallet.example.com"));
        core.set_provider_id(SharedString::from("wallet-user"));
        core.set_secret_reference(SharedString::from("wallet-pass"));
        core.set_object_kind(SharedString::from("sign"));

        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

        let catalog = read_protected_resources(&dir);
        let resource = resource_by_id(&catalog, "wallet_example_com").expect("saved resource");
        assert_eq!(
            resource.get("resourceKind").and_then(Value::as_str),
            Some("key_pair")
        );
        assert!(resource
            .get("bindings")
            .and_then(Value::as_array)
            .is_some_and(|bindings| bindings.iter().any(|binding| {
                binding.get("role").and_then(Value::as_str) == Some("private_key")
            })));
    });
}

#[test]
fn protected_object_rows_show_username_for_password_fill_resources() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_group(SharedString::from("normal"));
        core.set_display_name(SharedString::from("www.huawei.com"));
        core.set_provider_id(SharedString::from("bob"));
        core.set_secret_reference(SharedString::from("pw-123"));
        core.set_object_kind(SharedString::from("password_fill"));
        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

        let catalog = read_protected_resources(&dir);
        let rows = protected_object_rows(&catalog, "normal");
        let row = rows
            .iter()
            .find(|item| item.name == "www.huawei.com")
            .expect("row for saved site");
        assert_eq!(row.secret, "bob");
        assert_eq!(row.usage, "website");
        assert_eq!(row.level, "normal");
        assert!(!protected_object_rows(&catalog, "secret")
            .iter()
            .any(|item| item.name == "www.huawei.com"));
    });
}

#[test]
fn secret_password_fill_resource_is_saved_only_in_secret_rows() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_group(SharedString::from("secret"));
        core.set_editing_resource_group(SharedString::from("secret"));
        core.set_display_name(SharedString::from("secret-login.example.com"));
        core.set_provider_id(SharedString::from("secret-user"));
        core.set_secret_reference(SharedString::from("secret-pass"));
        core.set_object_kind(SharedString::from("password_fill"));
        save_object_editor_resource_from_window(&core, &dir).expect("save secret managed object");

        let catalog = read_protected_resources(&dir);
        let saved_resource_id = core.get_resource_id().to_string();
        let resource = resource_by_id(&catalog, &saved_resource_id).expect("saved secret resource");
        assert_eq!(
            resource.get("resourceGroup").and_then(Value::as_str),
            Some("secret")
        );
        assert!(protected_object_rows(&catalog, "secret")
            .iter()
            .any(|item| item.id == saved_resource_id));
        assert!(!protected_object_rows(&catalog, "normal")
            .iter()
            .any(|item| item.id == saved_resource_id));
        assert_eq!(core.get_resource_group().as_str(), "secret");
    });
}

#[test]
fn secret_website_accounts_stay_in_secret_group() {
    let mut catalog = json!({
        "version": "1",
        "resources": [
            {
                "resourceId": "secret-login",
                "resourceKind": "website_account",
                "providerId": "legacy",
                "displayName": "secret.example.com",
                "resourceGroup": "secret",
                "bindings": [
                    {
                        "role": "username",
                        "objectId": "credential/legacy/login/username",
                        "objectMeta": { "objectKind": "username", "encoding": "text", "sensitivity": "private" }
                    },
                    {
                        "role": "password",
                        "objectId": "credential/legacy/login/password",
                        "objectMeta": { "objectKind": "password", "encoding": "secret", "sensitivity": "secret" }
                    }
                ]
            },
            {
                "resourceId": "real-secret-key",
                "resourceKind": "key_pair",
                "providerId": "wallet",
                "displayName": "wallet key",
                "resourceGroup": "secret",
                "bindings": [
                    {
                        "role": "private_key",
                        "objectId": "keypair/wallet/main/private_key",
                        "objectMeta": { "objectKind": "private_key", "encoding": "secret", "sensitivity": "secret" }
                    }
                ]
            }
        ]
    });

    assert!(!normalize_legacy_resource_catalog_groups(&mut catalog));
    let login = resource_by_id(&catalog, "secret-login").expect("secret login");
    let key = resource_by_id(&catalog, "real-secret-key").expect("key pair");

    assert_eq!(
        login.get("resourceGroup").and_then(Value::as_str),
        Some("secret")
    );
    assert_eq!(
        key.get("resourceGroup").and_then(Value::as_str),
        Some("secret")
    );
    assert!(protected_object_rows(&catalog, "secret")
        .iter()
        .any(|item| item.id == "secret-login"));
    assert!(!protected_object_rows(&catalog, "normal")
        .iter()
        .any(|item| item.id == "secret-login"));
}

#[test]
fn managed_object_editor_delete_removes_selected_resource() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_resource_group(SharedString::from("normal"));
        core.set_display_name(SharedString::from("delete.example.com"));
        core.set_provider_id(SharedString::from("delete-user"));
        core.set_secret_reference(SharedString::from("delete-pass"));
        core.set_object_kind(SharedString::from("password_fill"));
        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");
        assert_eq!(core.get_resource_id().as_str(), "delete_example_com");

        delete_object_editor_resource_from_window(&core, &dir).expect("delete managed object");

        let catalog = read_protected_resources(&dir);
        assert!(resource_by_id(&catalog, "delete_example_com").is_none());
        assert_eq!(core.get_resource_id().as_str(), "");
        assert!(!protected_object_rows(&catalog, "normal")
            .iter()
            .any(|item| item.name == "delete.example.com"));
        assert!(!resolve_data_dir()
            .join("credentials")
            .join("credential-delete-user-delete_example_com.json")
            .exists());
    });
}

#[test]
fn managed_object_editor_save_updates_existing_credential_file() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_resource_group(SharedString::from("normal"));
        core.set_display_name(SharedString::from("replace.example.com"));
        core.set_provider_id(SharedString::from("old-user"));
        core.set_secret_reference(SharedString::from("old-pass"));
        core.set_object_kind(SharedString::from("password_fill"));
        save_object_editor_resource_from_window(&core, &dir).expect("initial save");

        core.set_provider_id(SharedString::from("new-user"));
        core.set_secret_reference(SharedString::from("new-pass"));
        save_object_editor_resource_from_window(&core, &dir).expect("resave");

        let catalog = read_protected_resources(&dir);
        let resource = resource_by_id(&catalog, "replace_example_com").expect("saved resource");
        assert_eq!(read_username_for_resource(resource), "new-user");
        assert_eq!(
            binding_secret_ref(resource, "password")
                .and_then(|secret_ref| read_stored_credential_field(
                    secret_ref.as_str(),
                    "password"
                ))
                .as_deref(),
            Some("new-pass")
        );
    });
}

#[test]
fn managed_object_save_keeps_other_template_bundle_items() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_resource_group(SharedString::from("normal"));
        core.set_display_name(SharedString::from("keep.example.com"));
        core.set_provider_id(SharedString::from("keep-user"));
        core.set_secret_reference(SharedString::from("keep-pass"));
        core.set_object_kind(SharedString::from("password_fill"));
        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

        let vault = read_storylock_vault_payload(&dir);
        let templates = storylock_templates_from_vault(&vault);
        let login_items = templates
            .get("loginSites")
            .and_then(|bundle| bundle.get("items"))
            .and_then(Value::as_array)
            .expect("login site items");
        assert!(login_items
            .iter()
            .any(|item| { item.get("resourceId").and_then(Value::as_str) == Some("github-main") }));
        assert!(login_items.iter().any(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some("keep_example_com")
        }));
    });
}

#[test]
fn managed_object_delete_removes_only_its_template_children() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_resource_group(SharedString::from("normal"));
        core.set_display_name(SharedString::from("remove.example.com"));
        core.set_provider_id(SharedString::from("remove-user"));
        core.set_secret_reference(SharedString::from("remove-pass"));
        core.set_object_kind(SharedString::from("password_fill"));
        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");
        assert_eq!(core.get_resource_id().as_str(), "remove_example_com");

        delete_object_editor_resource_from_window(&core, &dir).expect("delete managed object");

        let vault = read_storylock_vault_payload(&dir);
        let templates = storylock_templates_from_vault(&vault);
        let login_items = templates
            .get("loginSites")
            .and_then(|bundle| bundle.get("items"))
            .and_then(Value::as_array)
            .expect("login site items");
        assert!(!login_items.iter().any(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some("remove_example_com")
        }));
        assert!(login_items
            .iter()
            .any(|item| { item.get("resourceId").and_then(Value::as_str) == Some("github-main") }));
    });
}

#[test]
fn managed_object_resource_records_template_shell_children() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_display_name(SharedString::from("shell.example.com"));
        core.set_provider_id(SharedString::from("shell-user"));
        core.set_secret_reference(SharedString::from("shell-pass"));
        core.set_object_kind(SharedString::from("password_fill"));

        save_object_editor_resource_from_window(&core, &dir).expect("save managed object");

        let catalog = read_protected_resources(&dir);
        let resource = resource_by_id(&catalog, "shell_example_com").expect("saved resource");
        let children = resource
            .get("templateShell")
            .and_then(|shell| shell.get("children"))
            .and_then(Value::as_array)
            .expect("template shell children");
        assert_eq!(children.len(), 3);
        assert!(children.iter().any(|child| {
            child.get("bundleKey").and_then(Value::as_str) == Some("loginSites")
                && child.get("enabled").and_then(Value::as_bool) == Some(true)
        }));
        assert!(children.iter().any(|child| {
            child.get("bundleKey").and_then(Value::as_str) == Some("signingActions")
                && child.get("enabled").and_then(Value::as_bool) == Some(false)
        }));
    });
}

#[test]
fn signing_resource_syncs_only_signing_template_child() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");

        core.set_resource_id(SharedString::from(""));
        core.set_resource_group(SharedString::from("secret"));
        core.set_display_name(SharedString::from("wallet.sign.example"));
        core.set_provider_id(SharedString::from("wallet-user"));
        core.set_secret_reference(SharedString::from("wallet-secret"));
        core.set_object_kind(SharedString::from("sign"));

        save_object_editor_resource_from_window(&core, &dir).expect("save signing object");

        let vault = read_storylock_vault_payload(&dir);
        let templates = storylock_templates_from_vault(&vault);

        let signing_items = templates
            .get("signingActions")
            .and_then(|bundle| bundle.get("items"))
            .and_then(Value::as_array)
            .expect("signing items");
        assert!(signing_items.iter().any(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some("wallet_sign_example")
        }));

        let login_items = templates
            .get("loginSites")
            .and_then(|bundle| bundle.get("items"))
            .and_then(Value::as_array)
            .expect("login items");
        assert!(!login_items.iter().any(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some("wallet_sign_example")
        }));

        let agent_items = templates
            .get("agentTasks")
            .and_then(|bundle| bundle.get("items"))
            .and_then(Value::as_array)
            .expect("agent items");
        assert!(!agent_items.iter().any(|item| {
            item.get("resourceId").and_then(Value::as_str) == Some("wallet_sign_example")
        }));
    });
}

#[test]
fn new_managed_object_keeps_selected_resource_group() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        core.set_resource_group(SharedString::from("private"));

        let catalog = read_protected_resources(&dir);
        prepare_new_resource_in_window(&core, &catalog);

        assert_eq!(core.get_resource_group().as_str(), "private");
    });
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
    broken["summary"] = json!("broken summary placeholder");
    broken["nodes"][0]["question"] =
        json!("?濮樻挸绨甛u{0088}韫?\u{00aa}濮樻挻鍩塡u{0085}濮樻彫u{0085}\u{0094}?");
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
fn learning_progress_caches_answer_state_by_question() {
    let mut progress = LearningProgress::new();
    progress.cache_answers_for_question(
        3,
        &[true, false, true, false, true, false, true, false, true],
    );
    assert_eq!(
        progress.cached_answers_for_question(3),
        vec![true, false, true, false, true, false, true, false, true]
    );
    assert_eq!(progress.cached_answers_for_question(4), vec![false; 9]);
}

#[test]
fn learning_answer_cache_round_trips_through_window_state() {
    run_ui_test(|| {
        let core = StoryLockCoreApp::new().expect("core app");
        let progress = Rc::new(RefCell::new(LearningProgress::new()));
        let selected = vec![true, false, true, false, true, false, true, false, true];

        core.set_learning_index(3);
        set_learning_answer_states_into_window(&core, &selected);
        cache_current_learning_answers(&core, &progress);

        set_learning_answer_states_into_window(&core, &[false; 9]);
        assert_eq!(learning_answer_states_from_window(&core), vec![false; 9]);

        restore_cached_learning_answers_into_window(&core, &progress, 3);
        assert_eq!(learning_answer_states_from_window(&core), selected);

        restore_cached_learning_answers_into_window(&core, &progress, 4);
        assert_eq!(learning_answer_states_from_window(&core), vec![false; 9]);
    });
}

#[test]
fn learning_answer_state_toggle_flips_once() {
    run_ui_test(|| {
        let core = StoryLockCoreApp::new().expect("core app");

        set_learning_answer_states_into_window(&core, &[false; 9]);
        let mut states = learning_answer_states_from_window(&core);
        states[0] = !states[0];
        set_learning_answer_states_into_window(&core, &states);
        assert_eq!(
            learning_answer_states_from_window(&core),
            vec![true, false, false, false, false, false, false, false, false]
        );

        let mut states = learning_answer_states_from_window(&core);
        states[0] = !states[0];
        set_learning_answer_states_into_window(&core, &states);
        assert_eq!(learning_answer_states_from_window(&core), vec![false; 9]);
    });
}

#[test]
fn learning_answer_toggle_helper_updates_window_state() {
    run_ui_test(|| {
        let core = StoryLockCoreApp::new().expect("core app");

        set_learning_answer_states_into_window(&core, &[false; 9]);
        toggle_learning_answer_state(&core, 2);
        assert_eq!(
            learning_answer_states_from_window(&core),
            vec![false, false, true, false, false, false, false, false, false]
        );

        toggle_learning_answer_state(&core, 2);
        assert_eq!(learning_answer_states_from_window(&core), vec![false; 9]);
    });
}

#[test]
fn learning_previous_restores_cursor_and_cached_answers_before_reselect() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        let progress = Rc::new(RefCell::new(LearningProgress::new()));
        let first_question_states =
            vec![true, false, false, false, false, false, false, false, false];

        load_learning_node_into_window(&core, &dir, 0, Some(&progress));
        set_learning_answer_states_into_window(&core, &first_question_states);
        cache_current_learning_answers(&core, &progress);
        check_learning_current(&core, &dir, &progress).expect("advance to next question");

        assert_eq!(progress.borrow().cursor(), 1);
        assert_eq!(progress.borrow().checked(), 1);
        assert_eq!(core.get_learning_index(), 1);
        assert_eq!(core.get_learning_current_question(), 2);
        assert_eq!(core.get_selected_question().as_str(), "2");
        assert_eq!(core.get_learning_checked_prompts(), 1);

        set_learning_answer_states_into_window(
            &core,
            &[false, true, false, false, false, false, false, false, false],
        );
        cache_current_learning_answers(&core, &progress);
        retreat_learning_cursor(&core, &dir, &progress);

        assert_eq!(progress.borrow().cursor(), 0);
        assert_eq!(progress.borrow().checked(), 0);
        assert_eq!(core.get_learning_index(), 0);
        assert_eq!(core.get_learning_current_question(), 1);
        assert_eq!(core.get_selected_question().as_str(), "1");
        assert_eq!(core.get_learning_checked_prompts(), 0);
        assert_eq!(
            learning_answer_states_from_window(&core),
            first_question_states
        );

        load_learning_cursor_into_window(&core, &dir, &progress);
        assert_eq!(core.get_learning_index(), 0);
        assert_eq!(core.get_learning_current_question(), 1);
        assert_eq!(core.get_selected_question().as_str(), "1");
        assert_eq!(
            learning_answer_states_from_window(&core),
            first_question_states
        );
        assert!(check_learning_current(&core, &dir, &progress).is_ok());
    });
}

#[test]
fn learning_next_after_previous_edit_keeps_cursor_consistent() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        let progress = Rc::new(RefCell::new(LearningProgress::new()));

        load_learning_node_into_window(&core, &dir, 0, Some(&progress));
        set_learning_answer_states_into_window(
            &core,
            &[true, false, false, false, false, false, false, false, false],
        );
        cache_current_learning_answers(&core, &progress);
        check_learning_current(&core, &dir, &progress).expect("advance to second question");

        let second_question = core.get_learning_index();
        set_learning_answer_states_into_window(
            &core,
            &[false, true, false, false, false, false, false, false, false],
        );
        cache_current_learning_answers(&core, &progress);
        retreat_learning_cursor(&core, &dir, &progress);

        toggle_learning_answer_state(&core, 4);
        cache_current_learning_answers(&core, &progress);
        assert!(check_learning_current(&core, &dir, &progress).is_ok());

        assert_eq!(core.get_learning_index(), second_question);
        assert_eq!(
            progress.borrow().current_node_index(),
            second_question as usize
        );
    });
}

#[test]
fn opening_learning_dialog_twice_reuses_existing_window_state() {
    run_ui_test(|| {
        let core = StoryLockCoreApp::new().expect("core app");
        let learning_dialog: Rc<RefCell<Option<LearningTestDialog>>> = Rc::new(RefCell::new(None));

        core.set_learning_question(SharedString::from("Question A"));
        core.set_learning_result(SharedString::from("Result A"));
        open_learning_test_dialog(&core, Rc::clone(&learning_dialog));

        {
            let borrowed = learning_dialog.borrow();
            let dialog = borrowed.as_ref().expect("learning dialog should open");
            assert_eq!(dialog.get_learning_question().as_str(), "Question A");
            assert_eq!(dialog.get_learning_result().as_str(), "Result A");
        }

        core.set_learning_question(SharedString::from("Question B"));
        core.set_learning_result(SharedString::from("Result B"));
        open_learning_test_dialog(&core, Rc::clone(&learning_dialog));

        {
            let borrowed = learning_dialog.borrow();
            let dialog = borrowed.as_ref().expect("learning dialog should be reused");
            assert_eq!(dialog.get_learning_question().as_str(), "Question B");
            assert_eq!(dialog.get_learning_result().as_str(), "Result B");
        }
    });
}

#[test]
fn final_learning_pass_enables_manual_export_and_reports_success() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let export_dir = dir
            .parent()
            .expect("runtime root")
            .join("auto-export-package");
        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        core.set_language(SharedString::from("zh"));
        core.set_export_package_dir(SharedString::from(export_dir.display().to_string()));
        core.set_pre_learning_prompts_per_question(SharedString::from("1"));
        core.set_pre_learning_error_tolerance(SharedString::from("1"));
        core.set_weak_item_limit(SharedString::from("1"));

        let learning_passed = Rc::new(RefCell::new(LearningProgress::new()));
        let learning_dialog: Rc<RefCell<Option<LearningTestDialog>>> = Rc::new(RefCell::new(None));
        callbacks::learning_export::register_learning_export_callbacks(
            &core,
            &dir,
            Rc::clone(&learning_passed),
            Rc::clone(&learning_dialog),
            4510,
        );

        core.invoke_run_learning();
        for _ in 0..24 {
            let node_index = normalize_node_index(core.get_learning_index());
            let draft = read_effective_author_draft(&dir);
            let node = draft
                .get("nodes")
                .and_then(Value::as_array)
                .and_then(|nodes| nodes.get(node_index))
                .expect("learning node");
            let expected = node_answer_options(node)
                .iter()
                .map(|option| {
                    option
                        .get("isCorrect")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            set_learning_answer_states_into_window(&core, &expected);
            core.invoke_learning_next();
        }

        assert!(core.get_export_ready());
        assert!(storylock_core_learning_state_path(&dir).exists());
        assert!(
            has_current_learning_completed_state(&dir),
            "learning completion must remain valid after the final prompt so the export page does not return to locked"
        );
        assert!(
            !export_dir.join("vault.stlk").exists(),
            "learning completion should enable export without running export automatically"
        );

        core.invoke_export_package();

        assert!(export_dir.join("vault.stlk").exists());
        assert!(export_dir.join("EXPORT_STATUS.txt").exists());
        assert!(
            core.get_learning_result().contains("Training complete"),
            "manual export should not overwrite the learning result, got {}",
            core.get_learning_result()
        );
        assert!(
            core.get_config_status()
                .contains("\u{5bfc}\u{51fa}\u{6210}\u{529f}")
                || core.get_config_status().contains("Export complete"),
            "manual export should report export success only in export/config status, got {}",
            core.get_config_status()
        );
    });
}

#[test]
fn completed_learning_unlocks_export_even_with_recorded_errors() {
    run_ui_test(move || {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        core.set_pre_learning_prompts_per_question(SharedString::from("2"));
        core.set_pre_learning_error_tolerance(SharedString::from("1"));
        core.set_weak_item_limit(SharedString::from("1"));
        save_learning_policy_from_window(&core, &dir).expect("save policy");

        let progress = Rc::new(RefCell::new(LearningProgress::from_prompts_per_question(2)));
        load_learning_node_into_window(&core, &dir, 0, Some(&progress));
        let mut report = String::new();
        for _ in 0..48 {
            set_learning_answer_states_into_window(&core, &[false; 9]);
            report =
                check_learning_current(&core, &dir, &progress).expect("record learning prompt");
        }

        assert_eq!(core.get_learning_checked_prompts(), 48);
        assert!(core.get_learning_error_count() > 1);
        assert!(
            core.get_export_ready(),
            "finishing the full learning test should unlock export even when errors are recorded"
        );
        assert!(has_current_learning_completed_state(&dir));
        assert!(report.contains("Export is enabled"));
    });
}

#[test]
fn completed_learning_unlocks_export_without_package_preflight() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let mut catalog = read_json_or_default(
            &storylock_core_catalog_path(&dir),
            default_resource_catalog_json(),
        );
        catalog["operationTemplates"] = Value::String("broken policy catalog".to_string());
        fs::write(
            storylock_core_catalog_path(&dir),
            serde_json::to_string_pretty(&catalog).expect("serialize broken catalog"),
        )
        .expect("write broken catalog");

        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        core.set_pre_learning_prompts_per_question(SharedString::from("1"));

        let progress = Rc::new(RefCell::new(LearningProgress::from_prompts_per_question(1)));
        load_learning_node_into_window(&core, &dir, 0, Some(&progress));
        let mut report = String::new();
        for _ in 0..24 {
            let node_index = normalize_node_index(core.get_learning_index());
            let draft = read_effective_author_draft(&dir);
            let node = draft
                .get("nodes")
                .and_then(Value::as_array)
                .and_then(|nodes| nodes.get(node_index))
                .expect("learning node");
            let expected = node_answer_options(node)
                .iter()
                .map(|option| {
                    option
                        .get("isCorrect")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            set_learning_answer_states_into_window(&core, &expected);
            report = check_learning_current(&core, &dir, &progress)
                .expect("learning should complete independently from export preflight");
        }

        assert!(core.get_export_ready());
        assert!(has_current_learning_completed_state(&dir));
        assert_eq!(core.get_learning_error_count(), 0);
        assert!(
            report.contains("Training complete"),
            "learning completion should be reported even when export preflight is broken, got {report}"
        );
        assert!(
            !preflight_storylock_core_package(&dir).errors.is_empty(),
            "test setup should keep package preflight broken"
        );
    });
}

#[test]
fn completed_learning_state_restores_export_and_resets_after_answer_change() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        write_learning_completed_state(&dir).expect("write learning state");
        assert!(has_current_learning_completed_state(&dir));

        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        assert!(core.get_export_ready());
        assert!(core
            .get_learning_status()
            .contains("Learning completed for current content"));

        load_node_into_window(&core, &dir, 0);
        core.set_answer_1(SharedString::from("changed answer after learning"));
        save_current_node_from_window(&core, &dir).expect("save changed answer");
        let progress = Rc::new(RefCell::new(LearningProgress::new()));
        reset_learning_gate(
            &core,
            &dir,
            &progress,
            "Answer changed. Run learning test again before export.",
        );

        assert!(!core.get_export_ready());
        assert!(!storylock_core_learning_state_path(&dir).exists());
        assert!(!has_current_learning_completed_state(&dir));
    });
}

#[test]
fn completed_learning_state_survives_policy_save_and_unchanged_draft_save() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        write_learning_completed_state(&dir).expect("write learning state");
        let before = stored_learning_state_fingerprint(&dir).expect("learning fingerprint");

        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        core.set_pre_learning_prompts_per_question(SharedString::from("3"));
        save_learning_policy_from_window(&core, &dir).expect("save policy");
        assert!(has_current_learning_completed_state(&dir));
        assert_eq!(
            stored_learning_state_fingerprint(&dir).as_deref(),
            Some(before.as_str())
        );

        let bad_export_dir = dir.join("missing-parent").join("blocked-export");
        core.set_export_package_dir(SharedString::from(bad_export_dir.display().to_string()));
        let learning_passed = Rc::new(RefCell::new(LearningProgress::new()));
        reset_learning_gate(
            &core,
            &dir,
            &learning_passed,
            "No answer change should keep learning passed.",
        );
        assert!(core.get_export_ready());
        assert!(has_current_learning_completed_state(&dir));
    });
}

#[test]
fn export_failure_does_not_reset_completed_learning_state() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        write_learning_completed_state(&dir).expect("write learning state");

        let blocked_export_path = dir
            .parent()
            .expect("runtime root")
            .join("blocked-export-target");
        fs::write(&blocked_export_path, "not a directory").expect("create blocking export file");

        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        core.set_export_package_dir(SharedString::from(
            blocked_export_path.display().to_string(),
        ));
        core.set_learning_status(SharedString::from("Learning already passed."));
        core.set_learning_result(SharedString::from("Training complete."));
        assert!(core.get_export_ready());

        let learning_passed = Rc::new(RefCell::new(LearningProgress::new()));
        let learning_dialog: Rc<RefCell<Option<LearningTestDialog>>> = Rc::new(RefCell::new(None));
        callbacks::learning_export::register_learning_export_callbacks(
            &core,
            &dir,
            Rc::clone(&learning_passed),
            Rc::clone(&learning_dialog),
            4511,
        );

        core.invoke_export_package();

        assert!(core.get_export_ready());
        assert!(has_current_learning_completed_state(&dir));
        assert_eq!(
            core.get_learning_status().as_str(),
            "Learning already passed."
        );
        assert_eq!(core.get_learning_result().as_str(), "Training complete.");
        assert!(
            core.get_config_status().contains("Export failed")
                || core
                    .get_config_status()
                    .contains("\u{5bfc}\u{51fa}\u{5931}\u{8d25}"),
            "export failure should be reported only as export status, got {}",
            core.get_config_status()
        );
    });
}

#[test]
fn export_normalizes_legacy_builtin_template_resources() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    write_learning_completed_state(&dir).expect("write learning state");
    let export_dir = dir
        .parent()
        .expect("runtime root")
        .join("normalized-export-package");

    let mut vault = read_storylock_vault_payload(&dir);
    vault["protectedResources"]["resources"] = json!([]);
    vault["templates"]["signingActions"]["items"][0]["resourceId"] = json!("github-main");
    save_storylock_vault_payload(&dir, vault).expect("write legacy broken vault");
    assert!(
        !preflight_storylock_core_package(&dir).errors.is_empty(),
        "test setup should reproduce the legacy template/resource mismatch"
    );

    export_storylock_package_to(&dir, &export_dir)
        .expect("export should normalize builtin resources");

    assert!(export_dir.join("vault.stlk").exists());
    assert!(preflight_storylock_core_package(&dir).errors.is_empty());
    let repaired = read_storylock_vault_payload(&dir);
    assert!(resource_by_id(&protected_resources_from_vault(&repaired), "github-main").is_some());
    assert!(resource_by_id(&protected_resources_from_vault(&repaired), "wallet-main").is_some());
    assert_eq!(
        repaired["templates"]["signingActions"]["items"][0]
            .get("resourceId")
            .and_then(Value::as_str),
        Some("wallet-main")
    );
}

#[test]
fn restart_learning_releases_current_modal_before_rerun() {
    run_ui_test(|| {
        let core = StoryLockCoreApp::new().expect("core app");
        let learning_dialog: Rc<RefCell<Option<LearningTestDialog>>> = Rc::new(RefCell::new(None));
        let restarted = Rc::new(RefCell::new(false));
        let restarted_for_callback = Rc::clone(&restarted);

        core.on_run_learning(move || {
            *restarted_for_callback.borrow_mut() = true;
        });

        open_learning_test_dialog(&core, Rc::clone(&learning_dialog));
        let dialog = {
            let borrowed = learning_dialog.borrow();
            borrowed
                .as_ref()
                .expect("learning dialog should open")
                .as_weak()
        };

        dialog
            .upgrade()
            .expect("learning dialog should still exist")
            .invoke_restart_learning();

        assert!(
            *restarted.borrow(),
            "restart should invoke a fresh learning run"
        );
        assert!(
            learning_dialog.borrow().is_none(),
            "restart should release the modal dialog so close/exit state is not trapped"
        );
    });
}

#[test]
fn learning_order_mismatch_does_not_panic_on_reload() {
    run_ui_test(|| {
        let dir = temp_identity_package_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let core = StoryLockCoreApp::new().expect("core app");
        let progress = Rc::new(RefCell::new(LearningProgress::new()));

        load_learning_node_into_window(&core, &dir, 0, Some(&progress));
        core.set_learning_index(1);

        let result = check_learning_current(&core, &dir, &progress);
        assert!(result.is_ok());
    });
}

#[test]
fn export_preview_is_redacted() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let preview = build_export_preview(&dir);
    assert!(preview.contains("permissionObjects=2"));
    assert!(preview.contains("preflight=OK"));
    assert!(preview.contains("learning-policy.json"));
    assert!(preview.contains("StoryLock UI internal preview only"));
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
        Some("\u{667a}\u{5b50}\u{7591}\u{90bb}")
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
    assert_eq!(export_dir, dir);
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
    assert!(export_dir.join("EXPORT_STATUS.txt").exists());
}

#[test]
fn default_export_stays_in_selected_storylock_package_directory() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    assert_eq!(default_storylock_export_dir(&dir), dir);

    run_ui_test(move || {
        let core = StoryLockCoreApp::new().expect("core app");
        initialize_storylock_core_window(&core, &dir);
        assert_eq!(core.get_core_data_dir().as_str(), dir.display().to_string());
        assert_eq!(
            core.get_export_package_dir().as_str(),
            dir.display().to_string(),
            "default export path must not fork into a separate storylock-managed-key-package"
        );
    });
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
    let mut vault = read_storylock_vault_payload(&dir);
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
    save_storylock_vault_payload(&dir, vault).expect("write broken template");
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
    run_ui_test(|| {
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
        save_temp_draft_from_window(&fake_core, &template_dir)
            .expect("save template directory draft");
        let plain_template =
            read_json_or_default(&template_dir.join("story-template.json"), Value::Null);
        assert_eq!(
            plain_template.get("summary").and_then(Value::as_str),
            Some("plain template saved marker")
        );
        assert!(!template_dir.join("story-drafts").exists());
    });
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
        managed_key_package_dir: None,
    };

    write_storylock_ui_settings(&settings_path, &settings).expect("write settings");
    let loaded = read_storylock_ui_settings(&settings_path).expect("read settings");

    assert_eq!(loaded, settings);
    assert_eq!(initial_storylock_core_package_dir(&loaded), core_dir);
}

#[test]
fn host_settings_merge_preserves_and_updates_encrypted_data_dir() {
    let original = StoryLockUiSettings {
        language: Some(String::from("zh")),
        core_data_dir: Some(String::from("E:/storylock/vault-a")),
        export_package_dir: Some(String::from("E:/storylock/export")),
        managed_key_package_dir: None,
    };
    let language_only = merge_host_settings(&original, "en", None);
    assert_eq!(language_only.language.as_deref(), Some("en"));
    assert_eq!(
        language_only.core_data_dir.as_deref(),
        Some("E:/storylock/vault-a")
    );
    assert_eq!(
        language_only.export_package_dir.as_deref(),
        Some("E:/storylock/export")
    );

    let changed_path = merge_host_settings(&language_only, "en", Some(String::from("D:/vault-b")));
    assert_eq!(changed_path.core_data_dir.as_deref(), Some("D:/vault-b"));
}

#[test]
fn encrypted_data_path_resolution_uses_real_storylock_package_directory() {
    let root = temp_core_dir();
    let package_dir = root.join("identity-package");
    ensure_storylock_core_package(&package_dir).expect("init package");
    assert_eq!(resolve_storylock_core_package_path(&root), package_dir);
    assert_eq!(
        resolve_storylock_core_package_path(package_dir.join("vault.stlk")),
        package_dir
    );

    let template_dir = root.join("templates").join("shouzhudaitu-zh");
    ensure_storylock_core_package(&template_dir).expect("init template package");
    assert_eq!(
        resolve_storylock_core_package_path(&template_dir),
        template_dir
    );

    let merged = merge_host_settings(
        &StoryLockUiSettings::default(),
        "zh",
        Some(root.display().to_string()),
    );
    assert_eq!(
        merged.core_data_dir.as_deref(),
        Some(package_dir.display().to_string().as_str())
    );
}

#[test]
fn host_settings_page_exposes_encrypted_data_dir_picker() {
    let host_source = include_str!("../host_dashboard.slint");
    let dashboard_source = include_str!("../dashboard.rs");

    assert!(host_source.contains("encrypted-data-dir"));
    assert!(host_source.contains("browse-encrypted-data-dir"));
    assert!(host_source.contains("PathBrowseRow"));
    assert!(dashboard_source.contains("on_browse_encrypted_data_dir"));
    assert!(dashboard_source.contains("resolve_storylock_core_package_path(&selected_path)"));
    assert!(dashboard_source.contains("ensure_storylock_core_package(&package_dir)"));
}

#[test]
fn storylock_ui_settings_drop_legacy_managed_key_package_paths() {
    let mut settings = StoryLockUiSettings {
        language: Some(String::from("zh")),
        core_data_dir: Some(String::from("E:/storylock/identity-package")),
        export_package_dir: Some(String::from("E:/storylock/storylock-managed-key-package")),
        managed_key_package_dir: Some(String::from("E:/storylock/storylock-managed-key-package")),
    };

    normalize_storylock_ui_settings(&mut settings);

    assert_eq!(
        settings.core_data_dir.as_deref(),
        Some("E:/storylock/identity-package")
    );
    assert_eq!(settings.export_package_dir, None);
    assert_eq!(settings.managed_key_package_dir, None);
}

#[test]
fn package_self_check_report_exposes_vault_mtime_and_statuses() {
    let dir = temp_identity_package_dir();
    ensure_storylock_core_package(&dir).expect("init package");
    let report = package_dir_status_report(&dir);
    assert!(report.contains("Package: "));
    assert!(report.contains("Vault mtime (unix): "));
    assert!(report.contains("Learning policy: present"));
    assert!(report.contains("Export status: missing"));
    assert!(report.contains("Path boundary: current package root only"));
}

#[test]
fn learning_test_dialog_keeps_sixteen_nine_window_size() {
    let dialogs = include_str!("../storylock_core/dialogs.slint");
    let common = include_str!("../common.slint");
    let core_shell = include_str!("../storylock_core.slint");
    let learning_export = include_str!("../storylock_core/pages_learning_export.slint");
    let learning_callbacks = include_str!("callbacks/learning_export.rs");
    let editor_dialogs = include_str!("editor_flow/dialogs.rs");
    let template_shell = include_str!("resource_export/template_shell.rs");
    let start = dialogs
        .find("export component LearningTestDialog inherits Window {")
        .expect("learning dialog component");
    let section = &dialogs[start..];
    assert!(section.contains("preferred-width: 960px;"));
    assert!(section.contains("preferred-height: 540px;"));
    assert!(section.contains("min-width: 960px;"));
    assert!(section.contains("max-width: 960px;"));
    assert!(section.contains("min-height: 540px;"));
    assert!(section.contains("max-height: 540px;"));
    assert!(section.contains("learning-answer-1-state: \"wrong\";"));
    assert!(section.contains("learning-answer-9-state: \"wrong\";"));
    assert!(
        section.contains("in property <bool> can-learning-previous: learning-checked-prompts > 0;")
    );
    assert!(section.contains(
        "in property <bool> can-learning-next: learning-checked-prompts < learning-total-prompts;"
    ));
    assert!(section.contains("text: root.learning-position;"));
    assert!(section.contains("text: root.learning-question;"));
    assert!(section.contains("text: root.learning-action-hint;"));
    assert!(section.contains("text: root.learning-result;"));
    assert!(!section.contains("value: root.learning-position + \"  \" + root.learning-question;"));
    assert!(!section.contains("StaticTextPanel {"));
    assert!(section.contains("toggle-answer(index) => { root.learning-toggle-answer(index); }"));
    assert!(common.contains("callback toggle-requested();"));
    assert!(section.contains("label: root.is-zh ? \"\\u{4e0a}\\u{4e00}\\u{9898}\" : \"Previous\";"));
    assert!(section.contains("label: root.is-zh ? \"\\u{4e0b}\\u{4e00}\\u{9898}\" : \"Next\";"));
    assert!(!section.contains("StatusStrip {"));
    assert!(section.contains("ActionButton { x: 12px; y: 20px; label: root.is-zh ? \"\\u{4e0a}\\u{4e00}\\u{9898}\" : \"Previous\"; primary: false; enabled: root.can-learning-previous; button-width: 156px; clicked => { root.learning-previous(); } }"));
    assert!(section.contains("ActionButton { x: 12px; y: 58px; label: root.is-zh ? \"\\u{4e0b}\\u{4e00}\\u{9898}\" : \"Next\"; primary: false; enabled: root.can-learning-next; button-width: 156px; clicked => { root.learning-next(); } }"));
    assert!(section.contains("y: 100px;"));
    assert!(section.contains("ActionButton { x: 12px; y: 112px; label: root.is-zh ? \"\\u{91cd}\\u{65b0}\\u{5f00}\\u{59cb}\" : \"Restart\"; primary: true; button-width: 156px; clicked => { root.restart-learning(); } }"));
    assert!(section.contains("ActionButton { x: 12px; y: 150px; label: root.is-zh ? \"\\u{5173}\\u{95ed}\" : \"Close\"; primary: false; button-width: 156px; clicked => { root.close-requested(); } }"));
    assert!(common.contains("export component BinaryStateMark inherits Rectangle {"));
    assert!(common.contains("export component BinaryStateCheckBox inherits Rectangle {"));
    assert!(common.contains("in property <bool> enabled: true;"));
    assert!(common.contains("enabled: root.enabled;"));
    assert!(core_shell.contains("学习策略设置"));
    assert!(core_shell.contains("Learning Policy Settings"));
    assert!(core_shell.contains("Learning Policy"));
    assert!(learning_export.contains("enabled: root.export-ready;"));
    assert!(learning_export.contains("Export Locked"));
    assert!(learning_export.contains("in property <string> config-status;"));
    assert!(learning_export.contains("Test Status: "));
    assert!(learning_export.contains("Export Status: "));
    assert!(learning_export.contains("CopyableLogPanel {"));
    assert_eq!(learning_export.matches("CopyableLogPanel {").count(), 1);
    assert!(!learning_export.contains("        LogPanel {"));
    assert!(!learning_export.contains("LogPanel,"));
    assert!(common.contains("export component CopyableLogPanel inherits Rectangle {"));
    assert!(common.contains("read-only: true;"));
    assert!(core_shell.contains("config-status: root.config-status;"));
    assert!(learning_callbacks.contains("Export blocked. Run and pass the nine-grid test first."));
    assert!(learning_callbacks.contains("Exporting encrypted package..."));
    let export_callback = learning_callbacks
        .split("core.on_export_package(move || {")
        .nth(1)
        .expect("export callback");
    assert!(!export_callback.contains("begin_learning_gate("));
    assert!(!common.contains("export component StatusStrip inherits Rectangle {"));
    assert!(common.contains("in property <bool> show-label: true;"));
    assert!(common.contains("in property <bool> show-frame: true;"));
    assert!(common.contains("if root.show-state: BinaryStateCheckBox {"));
    assert!(common.contains("text: root.state == \"correct\" ? \"\\u{2713}\" : \"X\";"));
    assert!(common.contains("x: root.show-label ? 8px : 0px;"));
    assert!(common.contains("width: root.show-label ? parent.width - 20px : parent.width;"));
    assert!(common.contains("width: root.show-label ? 20px : parent.width;"));
    assert!(common.contains("height: root.show-label ? 20px : parent.height;"));
    assert!(common.contains("glyph-size: root.show-label ? 12px : 24px;"));
    assert!(common.contains("mark-radius: root.show-label ? 3px : 4px;"));
    assert!(common.contains("if !root.show-label: BinaryStateMark {"));
    assert!(common.contains("width: parent.width;"));
    assert!(common.contains("height: parent.height;"));
    assert!(common.contains("show-label: false;"));
    assert!(common.contains("width: parent.width - 76px;"));
    assert!(common.contains("x: parent.width - 62px;"));
    assert!(common.contains("y: parent.height - 34px;"));
    assert!(common.contains("width: 60px;"));
    assert!(common.contains("height: 32px;"));
    assert!(common.contains("interactive: false;"));
    assert!(common.contains("clicked => {\n                root.toggle-requested();"));
    assert!(common.contains("double-clicked => {"));
    assert!(common.contains("width: parent.width - 76px;\n            height: parent.height;"));
    assert!(common.contains("width: 724px;"));
    assert!(common.contains("width: (root.width - 12px) / 3;"));
    assert!(common.contains("height: (root.height - 12px) / 3;"));
    assert!(common.contains("show-frame: false;"));
    assert!(editor_dialogs.contains("let modal_owner = Rc::new(Cell::new(null_mut()));"));
    assert!(editor_dialogs.contains("EnableWindow(owner, 0);"));
    assert!(editor_dialogs.contains("let restart_slot = Rc::clone(&learning_dialog);"));
    assert!(editor_dialogs.contains("restore_learning_modal_owner(&modal_owner_for_restart);"));
    assert!(editor_dialogs.contains("show_learning_passed_message();"));
    assert!(editor_dialogs.contains("测试已通过，可以导出。"));
    assert!(editor_dialogs.contains("restore_learning_modal_owner(&modal_owner_for_next);"));
    assert!(editor_dialogs.contains("restore_learning_modal_owner(&modal_owner_for_button);"));
    assert!(editor_dialogs.contains("restore_learning_modal_owner(&modal_owner);"));
    assert!(editor_dialogs.contains("fn restore_learning_modal_owner(modal_owner: &Cell<HWND>)"));
    assert!(editor_dialogs.contains("EnableWindow(owner, 1);"));
    assert!(template_shell
        .contains("pub(crate) const TEMPLATE_CHILD_SPECS: [TemplateChildSpec; 3] = ["));
    assert!(template_shell.contains("pub(crate) fn template_bundle_item_from_resource("));
    assert!(template_shell.contains("pub(crate) fn sync_template_children_for_resource("));
}
