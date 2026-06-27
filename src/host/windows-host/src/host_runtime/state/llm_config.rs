use super::*;

const API_KEY_ENV_NAMES: &[&str] = &[
    "STORYLOCK_STORY_LLM_API_KEY",
    "XFYUN_MAAS_API_KEY",
    "OPENAI_API_KEY",
    "ARK_API_KEY",
    "SEEDANCE_API_KEY",
];

const BASE_URL_ENV_NAMES: &[&str] = &[
    "STORYLOCK_STORY_LLM_BASE_URL",
    "XFYUN_MAAS_BASE_URL",
    "OPENAI_BASE_URL",
    "ARK_BASE_URL",
    "SEEDANCE_BASE_URL",
];

const MODEL_ENV_NAMES: &[&str] = &[
    "STORYLOCK_STORY_LLM_MODEL",
    "XFYUN_MAAS_MODEL",
    "OPENAI_MODEL",
    "ARK_MODEL",
    "ARK_VIDEO_MODEL",
    "SEEDANCE_MODEL",
];

fn find_first_env_value(names: &[&str], file_env: &HashMap<String, String>) -> Option<String> {
    names.iter().find_map(|name| env_lookup(name, file_env))
}

fn llm_provider_name(file_env: &HashMap<String, String>) -> &'static str {
    if env_lookup("XFYUN_MAAS_API_KEY", file_env).is_some() {
        "xfyun-maas"
    } else if env_lookup("SEEDANCE_API_KEY", file_env).is_some() {
        "seedance"
    } else if env_lookup("ARK_API_KEY", file_env).is_some() {
        "ark"
    } else if env_lookup("OPENAI_API_KEY", file_env).is_some() {
        "openai"
    } else {
        "story-llm"
    }
}

pub(crate) fn story_llm_config() -> Option<StoryLlmConfig> {
    let file_env = read_env_file_map();
    let api_key = find_first_env_value(API_KEY_ENV_NAMES, &file_env)?;
    let base_url = find_first_env_value(BASE_URL_ENV_NAMES, &file_env)?;
    let model = find_first_env_value(MODEL_ENV_NAMES, &file_env)?;
    Some(StoryLlmConfig {
        provider: llm_provider_name(&file_env).to_string(),
        api_key,
        base_url,
        model,
    })
}
