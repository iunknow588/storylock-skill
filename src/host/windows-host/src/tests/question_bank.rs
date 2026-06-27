use super::*;

#[test]
fn question_bank_can_be_loaded_and_validated() {
    let runtime = test_runtime();
    let loaded = load_or_init_question_bank(&runtime.config.data_dir).expect("load question bank");
    assert_eq!(loaded.question_set_version, "windows-local-v1");
    assert!(!loaded.questions.is_empty());
}

#[test]
fn question_bank_import_replaces_runtime_state() {
    let runtime = test_runtime();
    let import_path = runtime.config.data_dir.join("import-bank.json");
    fs::write(
        &import_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "windows-local-question-bank-v1",
            "questionSetVersion": "windows-local-v2",
            "normalizationVersion": "upper-ascii-v1",
            "questions": [{
                "questionId": "story-q-99",
                "promptRef": "prompt-99",
                "versionTag": "v2",
                "promptText": "Imported question.",
                "answer": "HORIZON"
            }]
        }))
        .expect("serialize import bank"),
    )
    .expect("write import bank");

    let response = question_bank_import(
        &runtime,
        &json!({
            "requestId": "req-import-1",
            "sourcePath": import_path.display().to_string()
        }),
    );
    assert_eq!(
        response.get("status").and_then(Value::as_str),
        Some("success")
    );
    let current = runtime.current_question_bank().expect("current question bank");
    assert_eq!(current.question_set_version, "windows-local-v2");
    assert_eq!(current.questions.len(), 1);
    assert_eq!(current.questions[0].answer, "HORIZON");
}

#[test]
fn question_bank_import_accepts_utf8_bom() {
    let runtime = test_runtime();
    let import_path = runtime.config.data_dir.join("import-bank-with-bom.json");
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend(
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "windows-local-question-bank-v1",
            "questionSetVersion": "windows-local-bom-v1",
            "normalizationVersion": "upper-ascii-v1",
            "questions": [{
                "questionId": "story-q-bom",
                "promptRef": "prompt-bom",
                "versionTag": "v1",
                "promptText": "BOM encoded question.",
                "answer": "ANCHOR"
            }]
        }))
        .expect("serialize import bank"),
    );
    fs::write(&import_path, bytes).expect("write bom import bank");

    let response = question_bank_import(
        &runtime,
        &json!({
            "requestId": "req-import-bom",
            "sourcePath": import_path.display().to_string()
        }),
    );
    assert_eq!(
        response.get("status").and_then(Value::as_str),
        Some("success")
    );
    assert_eq!(
        response
            .get("result")
            .and_then(|value| value.get("questionSetVersion"))
            .and_then(Value::as_str),
        Some("windows-local-bom-v1")
    );
}
