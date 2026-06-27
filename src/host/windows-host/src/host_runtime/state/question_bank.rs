use super::*;

pub(crate) fn question_bank_path(data_dir: &Path) -> PathBuf {
    data_dir.join("question-bank.json")
}

pub(crate) fn write_host_json_if_missing(path: &Path, value: &Value) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
    }
    Ok(())
}

pub(crate) fn default_question_bank_json() -> &'static str {
    include_str!("../../../assets/question-bank.json")
}

pub(crate) fn load_or_init_question_bank(data_dir: &Path) -> Result<QuestionBankFile> {
    let path = question_bank_path(data_dir);
    if !path.exists() {
        fs::create_dir_all(data_dir)?;
        fs::write(&path, default_question_bank_json())?;
    }
    read_and_validate_question_bank(&path)
}

pub(crate) fn read_and_validate_question_bank(path: &Path) -> Result<QuestionBankFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read question bank file: {}", path.display()))?;
    let parsed: QuestionBankFile = serde_json::from_str(content.trim_start_matches('\u{feff}'))
        .with_context(|| format!("failed to parse question bank file: {}", path.display()))?;
    validate_question_bank(&parsed)?;
    Ok(parsed)
}

pub(crate) fn validate_question_bank(question_bank: &QuestionBankFile) -> Result<()> {
    if question_bank.schema_version.trim().is_empty() {
        return Err(anyhow!("question bank schemaVersion must be non-empty"));
    }
    if question_bank.question_set_version.trim().is_empty() {
        return Err(anyhow!(
            "question bank questionSetVersion must be non-empty"
        ));
    }
    if question_bank.normalization_version.trim().is_empty() {
        return Err(anyhow!(
            "question bank normalizationVersion must be non-empty"
        ));
    }
    if question_bank.questions.is_empty() {
        return Err(anyhow!("question bank file contains no questions"));
    }
    for (index, question) in question_bank.questions.iter().enumerate() {
        if question.question_id.trim().is_empty() {
            return Err(anyhow!("question {} has empty questionId", index + 1));
        }
        if question.prompt_ref.trim().is_empty() {
            return Err(anyhow!("question {} has empty promptRef", index + 1));
        }
        if question.version_tag.trim().is_empty() {
            return Err(anyhow!("question {} has empty versionTag", index + 1));
        }
        if question.prompt_text.trim().is_empty() {
            return Err(anyhow!("question {} has empty promptText", index + 1));
        }
        if question.answer.trim().is_empty() {
            return Err(anyhow!("question {} has empty answer", index + 1));
        }
    }
    Ok(())
}

pub(crate) fn import_question_bank(data_dir: &Path, source_path: &Path) -> Result<QuestionBankFile> {
    let imported = read_and_validate_question_bank(source_path)?;
    fs::create_dir_all(data_dir)?;
    fs::copy(source_path, question_bank_path(data_dir)).with_context(|| {
        format!(
            "failed to copy question bank from {} into {}",
            source_path.display(),
            data_dir.display()
        )
    })?;
    Ok(imported)
}
