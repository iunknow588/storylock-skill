use super::*;

#[test]
fn story_template_generation_queues_candidates_for_storylock_pull() {
    let runtime = test_runtime();
    std::env::set_var("STORYLOCK_STORY_LLM_API_KEY", "sk-test-secret-value");
    std::env::set_var("STORYLOCK_STORY_LLM_BASE_URL", "https://example.test/v1");
    std::env::set_var("STORYLOCK_STORY_LLM_MODEL", "storylock-test-model");

    let generated = story_template_generate(
        &runtime,
        &json!({
            "requestId": "req-story-template",
            "theme": "train station memory",
            "audience": "desktop tester",
            "tone": "precise",
            "questionCount": 24
        }),
    );
    assert_eq!(
        generated.get("status").and_then(Value::as_str),
        Some("success")
    );
    assert_eq!(
        generated
            .get("result")
            .and_then(|value| value.get("storyLockPullRequired"))
            .and_then(Value::as_bool),
        Some(true)
    );

    let candidates = story_template_candidates(
        &runtime,
        &json!({
            "requestId": "req-story-template-candidates",
            "limit": 10
        }),
    );
    let result = candidates.get("result").expect("candidate result");
    assert_eq!(
        result
            .get("hostInvokesStoryLock")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        result
            .get("pullModel")
            .and_then(Value::as_str),
        Some("storylock_explicit_pull_only")
    );
    assert!(result
        .get("candidates")
        .and_then(Value::as_array)
        .expect("candidate array")
        .iter()
        .any(|candidate| {
            candidate
                .get("framework")
                .and_then(|framework| framework.get("title"))
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("train station memory"))
        }));

    let status = ui_status(&runtime);
    let template_status = status
        .get("result")
        .and_then(|value| value.get("storyTemplateGenerator"))
        .expect("template generator status");
    assert_eq!(
        template_status.get("llmKey").and_then(Value::as_str),
        Some("configured_direct_access")
    );
    assert!(!serde_json::to_string(&status)
        .expect("status json")
        .contains("sk-test-secret-value"));

    std::env::remove_var("STORYLOCK_STORY_LLM_API_KEY");
    std::env::remove_var("STORYLOCK_STORY_LLM_BASE_URL");
    std::env::remove_var("STORYLOCK_STORY_LLM_MODEL");
}
