use super::*;

#[derive(Clone)]
pub(crate) struct WindowsHostRuntime {
    pub(crate) config: WindowsHostConfig,
    pub(crate) secret_store: SecretStore,
    pub(crate) question_bank: Arc<Mutex<QuestionBankFile>>,
    pub(crate) ui_state: Arc<Mutex<RuntimeUiState>>,
}

impl WindowsHostRuntime {
    pub(crate) fn new(config: WindowsHostConfig) -> Result<Self> {
        let secret_store = SecretStore::new(config.data_dir.clone())?;
        let question_bank = load_or_init_question_bank(&config.data_dir)?;
        Ok(Self {
            config,
            secret_store,
            question_bank: Arc::new(Mutex::new(question_bank)),
            ui_state: Arc::new(Mutex::new(RuntimeUiState {
                started_at: now_timestamp(),
                relay_status: "starting".to_string(),
                last_relay_error: None,
                last_relay_poll_at: None,
                last_execution: None,
                last_confirmation: None,
            })),
        })
    }

    pub(crate) fn current_question_bank(&self) -> Result<QuestionBankFile> {
        self.question_bank
            .lock()
            .map(|bank| bank.clone())
            .map_err(|_| anyhow!("question bank lock was poisoned"))
    }

    pub(crate) fn replace_question_bank(&self, next: QuestionBankFile) -> Result<()> {
        let mut bank = self
            .question_bank
            .lock()
            .map_err(|_| anyhow!("question bank lock was poisoned"))?;
        *bank = next;
        Ok(())
    }

    pub(crate) fn set_relay_status(&self, status: &str, error: Option<String>) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.relay_status = status.to_string();
            state.last_relay_error = error;
            state.last_relay_poll_at = Some(now_timestamp());
        }
    }

    pub(crate) fn record_execution_summary(&self, response: &Value) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.last_execution = Some(summarize_execution_for_ui(response));
        }
    }

    pub(crate) fn record_confirmation_summary(&self, summary: Value) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.last_confirmation = Some(summary);
        }
    }

    pub(crate) fn ui_state_snapshot(&self) -> RuntimeUiState {
        self.ui_state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| RuntimeUiState {
                started_at: now_timestamp(),
                relay_status: "local_only".to_string(),
                last_relay_error: Some("ui state lock was poisoned".to_string()),
                last_relay_poll_at: None,
                last_execution: None,
                last_confirmation: None,
            })
    }
}
