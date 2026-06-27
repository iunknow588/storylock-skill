use super::*;

pub(crate) fn story_template_interface_manifest() -> Value {
    json!({
        "schemaVersion": "story-template-interface-manifest-v1",
        "owner": "yian-windows-host",
        "direction": "storylock_pulls_candidates",
        "interfaces": {
            "candidateQueue": "story-template-candidates.jsonl",
            "localHttpGenerate": "/story-template/generate",
            "localHttpCandidates": "/story-template/candidates"
        },
        "boundary": {
            "hostMayGenerateCandidates": true,
            "hostMustNotInvokeStoryLock": true,
            "storyLockImportsOnlyAfterExplicitPull": true,
            "llmKeysAreDirectAccessConfig": true
        }
    })
}

pub(crate) fn story_template_generator_status(runtime: &WindowsHostRuntime) -> Value {
    let llm = story_llm_config();
    json!({
        "schemaVersion": "story-template-generator-status-v1",
        "mode": if llm.is_some() { "llm_ready" } else { "local_template_fallback" },
        "llmKey": if llm.is_some() { "configured_direct_access" } else { "missing" },
        "llmEndpoint": llm.as_ref().map(|config| config.base_url.as_str()).unwrap_or("missing"),
        "llmModel": llm.as_ref().map(|config| config.model.as_str()).unwrap_or("missing"),
        "llmProvider": llm.as_ref().map(|config| config.provider.as_str()).unwrap_or("missing"),
        "candidateCount": runtime.secret_store.read_story_template_candidates(1000).len(),
        "interfaces": {
            "generate": format!("http://127.0.0.1:{}/story-template/generate", runtime.config.host_port),
            "candidates": format!("http://127.0.0.1:{}/story-template/candidates", runtime.config.host_port)
        },
        "boundary": "Host generates and queues candidates only. StoryLock must pull; Host never invokes StoryLock."
    })
}
