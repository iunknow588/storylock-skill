use super::*;

pub(crate) fn question_bank_import(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let request_id = request_id_from(request);
    let source_path = match request.get("sourcePath").and_then(Value::as_str) {
        Some(source_path) if !source_path.trim().is_empty() => source_path.trim(),
        _ => {
            return error_response(
                &runtime.config,
                &request_id,
                "questionBankImport",
                "SLG-001",
                "validation_error",
                "sourcePath is required",
                "Provide the question bank JSON file path to import.",
            )
        }
    };
    match import_question_bank(&runtime.config.data_dir, Path::new(source_path)).and_then(|bank| {
        runtime.replace_question_bank(bank.clone())?;
        Ok(bank)
    }) {
        Ok(bank) => json!({
            "requestId": request_id,
            "status": "success",
            "capability": "questionBankImport",
            "executionLocation": "local",
            "result": {
                "path": question_bank_path(&runtime.config.data_dir).display().to_string(),
                "questionSetVersion": bank.question_set_version,
                "normalizationVersion": bank.normalization_version,
                "questionCount": bank.questions.len()
            },
            "redactionLevel": "none",
            "retentionGranted": "audit_meta_only",
            "auditMeta": {
                "timestamp": now_timestamp()
            },
            "error": Value::Null
        }),
        Err(error) => error_response(
            &runtime.config,
            &request_id,
            "questionBankImport",
            "SLG-005",
            "host_storage_error",
            &format!("failed to import question bank: {error}"),
            "Validate the source JSON file and retry the question bank import request.",
        ),
    }
}
