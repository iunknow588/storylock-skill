use super::*;

fn candidate_generator_meta(generator_status: &Value) -> Value {
    json!({
        "owner": "yian-host",
        "mode": generator_status
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("local_template_fallback"),
        "llmKey": generator_status
            .get("llmKey")
            .and_then(Value::as_str)
            .unwrap_or("missing")
    })
}

fn template_candidate_payload(
    request_id: &str,
    framework: Value,
    generator_status: &Value,
) -> Value {
    json!({
        "schemaVersion": "story-template-candidate-v1",
        "candidateId": format!("story-template-{}", Uuid::new_v4()),
        "requestId": request_id,
        "createdAt": now_timestamp(),
        "generator": candidate_generator_meta(generator_status),
        "framework": framework,
        "consumption": {
            "direction": "storylock_pulls_candidate",
            "hostInvokesStoryLock": false,
            "status": "queued"
        },
        "redactionLevel": "candidate_only"
    })
}

pub(crate) fn story_template_generate(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    let framework = generate_story_framework(runtime, request);
    let generator_status = story_template_generator_status(runtime);
    let candidate = template_candidate_payload(&request_id, framework, &generator_status);
    match runtime
        .secret_store
        .append_story_template_candidate(&candidate)
    {
        Ok(()) => json!({
            "requestId": request_id,
            "status": "success",
            "capability": "generateStoryTemplateCandidate",
            "executionLocation": "local",
            "result": {
                "candidateId": candidate.get("candidateId").cloned().unwrap_or(Value::Null),
                "queued": true,
                "storyLockPullRequired": true,
                "framework": candidate.get("framework").cloned().unwrap_or(Value::Null)
            },
            "redactionLevel": "candidate_only",
            "retentionGranted": "candidate_queue_only",
            "auditMeta": {
                "timestamp": now_timestamp(),
                "interface": "story-template/generate",
                "llmKey": candidate
                    .get("generator")
                    .and_then(|value| value.get("llmKey"))
                    .cloned()
                    .unwrap_or(Value::String("missing".to_string()))
            },
            "error": Value::Null
        }),
        Err(error) => error_response(
            &runtime.config,
            &request_id,
            "generateStoryTemplateCandidate",
            "SLG-005",
            "host_storage_error",
            &format!("failed to queue story template candidate: {error}"),
            "Check the Windows host data directory and retry template generation.",
        ),
    }
}

pub(crate) fn story_template_candidates(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    let limit = request
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 100) as usize;
    json!({
        "requestId": request_id,
        "status": "success",
        "capability": "storyTemplateCandidates",
        "executionLocation": "local",
        "result": {
            "pullModel": "storylock_explicit_pull_only",
            "hostInvokesStoryLock": false,
            "interfaceManifest": story_template_interface_manifest(),
            "candidates": runtime.secret_store.read_story_template_candidates(limit)
        },
        "redactionLevel": "candidate_only",
        "retentionGranted": "candidate_queue_only",
        "auditMeta": {
            "timestamp": now_timestamp(),
            "interface": "story-template/candidates"
        },
        "error": Value::Null
    })
}
