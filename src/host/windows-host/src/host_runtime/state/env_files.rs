use super::*;

const STORY_ENV_DIR: &str = "姒瑨娅ㄥù浣衡柤";

fn explicit_env_file_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for name in ["STORYLOCK_EXTERNAL_ENV_FILE", "STORYLOCK_STORY_ENV_FILE"] {
        if let Ok(explicit) = std::env::var(name) {
            let trimmed = explicit.trim();
            if !trimmed.is_empty() {
                candidates.push(PathBuf::from(trimmed));
            }
        }
    }
    candidates
}

fn default_env_file_candidates() -> Vec<PathBuf> {
    let story_env_root = PathBuf::from("..")
        .join("..")
        .join("..")
        .join("..")
        .join(STORY_ENV_DIR);
    vec![
        PathBuf::from(".env"),
        story_env_root.join(".env"),
        story_env_root.join(".env.example"),
    ]
}

fn parse_env_file(content: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !key.is_empty() {
            values.insert(key.to_string(), value);
        }
    }
    values
}

pub(crate) fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

pub(crate) fn read_env_file_map() -> HashMap<String, String> {
    let mut candidates = explicit_env_file_candidates();
    candidates.extend(default_env_file_candidates());

    for candidate in candidates {
        if let Ok(content) = fs::read_to_string(&candidate) {
            let values = parse_env_file(&content);
            if !values.is_empty() {
                return values;
            }
        }
    }
    HashMap::new()
}

pub(crate) fn env_lookup(name: &str, file_env: &HashMap<String, String>) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            file_env
                .get(name)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

pub(crate) fn truthy_env(name: &str, fallback: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => fallback,
    }
}
