use super::*;

pub(crate) fn main() -> Result<()> {
    if let Ok(delay_ms) = std::env::var("YIAN_WINDOWS_HOST_RESTART_DELAY_MS") {
        if let Ok(delay_ms) = delay_ms.parse::<u64>() {
            thread::sleep(Duration::from_millis(delay_ms));
        }
        std::env::remove_var("YIAN_WINDOWS_HOST_RESTART_DELAY_MS");
    }

    let config = WindowsHostConfig::from_env();
    let args: Vec<String> = std::env::args().collect();
    let start_mode = std::env::var("STORYLOCK_WINDOWS_START_MODE")
        .unwrap_or_default()
        .to_ascii_lowercase();

    if args.iter().any(|arg| arg == "--slint-ui") {
        return run_slint_ui_entry(config);
    }
    if args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "--server-only" | "--debug-host" | "--runtime-debug"
        )
    }) || matches!(
        start_mode.as_str(),
        "server" | "server-only" | "console" | "debug" | "debug-host" | "runtime-debug"
    ) {
        return run_console_entry(config);
    }
    if args.iter().any(|arg| arg == "--print-config") {
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--print-question-bank-path") {
        println!("{}", question_bank_path(&config.data_dir).display());
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--validate-question-bank") {
        let bank = load_or_init_question_bank(&config.data_dir)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "path": question_bank_path(&config.data_dir).display().to_string(),
                "questionSetVersion": bank.question_set_version,
                "normalizationVersion": bank.normalization_version,
                "questionCount": bank.questions.len()
            }))?
        );
        return Ok(());
    }
    if let Some(index) = args.iter().position(|arg| arg == "--import-question-bank") {
        let source = args
            .get(index + 1)
            .ok_or_else(|| anyhow!("--import-question-bank requires a source file path"))?;
        let imported = import_question_bank(&config.data_dir, Path::new(source))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ok",
                "path": question_bank_path(&config.data_dir).display().to_string(),
                "questionSetVersion": imported.question_set_version,
                "normalizationVersion": imported.normalization_version,
                "questionCount": imported.questions.len()
            }))?
        );
        return Ok(());
    }
    run_default_entry(config)
}

pub(crate) fn run_console_entry(config: WindowsHostConfig) -> Result<()> {
    let runtime = WindowsHostRuntime::new(config)?;
    println!("{}", serde_json::to_string_pretty(&runtime.config)?);
    let server_runtime = runtime.clone();
    let _server = start_local_server(server_runtime)?;
    run_runtime_loop(runtime)
}

#[cfg(feature = "ui-slint")]
pub(crate) fn run_default_entry(config: WindowsHostConfig) -> Result<()> {
    run_desktop_ui_entry(config)
}

#[cfg(not(feature = "ui-slint"))]
pub(crate) fn run_default_entry(config: WindowsHostConfig) -> Result<()> {
    run_console_entry(config)
}

#[cfg(feature = "ui-slint")]
pub(crate) fn run_slint_ui_entry(config: WindowsHostConfig) -> Result<()> {
    run_desktop_ui_entry(config)
}

#[cfg(not(feature = "ui-slint"))]
pub(crate) fn run_slint_ui_entry(_config: WindowsHostConfig) -> Result<()> {
    Err(anyhow!(
        "Slint UI is not enabled. Run with: cargo run --features ui-slint -- --slint-ui"
    ))
}

#[cfg(feature = "ui-slint")]
pub(crate) fn run_desktop_ui_entry(config: WindowsHostConfig) -> Result<()> {
    let runtime = WindowsHostRuntime::new(config.clone())?;
    let server_runtime = runtime.clone();
    let _server = match start_local_server(server_runtime) {
        Ok(server) => server,
        Err(error) if error.to_string().contains("failed to bind local server") => {
            return slint_ui::run(config);
        }
        Err(error) => return Err(error),
    };
    thread::spawn(move || run_runtime_loop(runtime));
    slint_ui::run(config)
}
