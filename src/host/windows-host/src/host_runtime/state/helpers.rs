use super::*;

pub(crate) fn resolve_data_dir() -> PathBuf {
    if let Ok(configured) = std::env::var("STORYLOCK_WINDOWS_DATA_DIR") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(appdata).join("Yian").join("windows-host");
    }
    PathBuf::from(".").join(".windows-host-data")
}

pub(crate) fn sanitize_ref(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn short_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

pub(crate) fn content_type_json() -> Header {
    Header::from_bytes(
        &b"content-type"[..],
        &b"application/json; charset=utf-8"[..],
    )
    .expect("static header is valid")
}

pub(crate) fn content_type_html() -> Header {
    Header::from_bytes(&b"content-type"[..], &b"text/html; charset=utf-8"[..])
        .expect("static header is valid")
}
