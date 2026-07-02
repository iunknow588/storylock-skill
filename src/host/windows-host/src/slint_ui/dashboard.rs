use super::puzzle_adapter::{
    set_storylock_challenge_question, show_storylock_authorization_result,
};
use super::*;
use std::path::PathBuf;
use storylock_puzzle_plugin::{create_open_challenge_from_draft, StoryLockChallengeCell};

fn host_language_is_zh(language: &str) -> bool {
    language == "zh"
}

fn browse_host_config_file_path() -> Result<Option<PathBuf>> {
    let config_path = host_ui_settings_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(pick_host_config_file_once(&config_path))
}

fn remote_mode_label(remote_enabled: bool, restart_required: bool, language: &str) -> &'static str {
    match (
        host_language_is_zh(language),
        remote_enabled,
        restart_required,
    ) {
        (true, true, true) => {
            "\u{8fdc}\u{7a0b}\u{4e2d}\u{7ee7}\u{5df2}\u{6253}\u{5f00}\u{ff08}\u{91cd}\u{65b0}\u{6253}\u{5f00}\u{540e}\u{751f}\u{6548}\u{ff09}"
        }
        (true, false, true) => {
            "\u{672c}\u{673a}\u{6a21}\u{5f0f}\u{ff08}\u{91cd}\u{65b0}\u{6253}\u{5f00}\u{540e}\u{751f}\u{6548}\u{ff09}"
        }
        (true, true, false) => {
            "\u{8fdc}\u{7a0b}\u{4e2d}\u{7ee7}\u{5df2}\u{6253}\u{5f00}"
        }
        (true, false, false) => "\u{672c}\u{673a}\u{6a21}\u{5f0f}",
        (false, true, true) => "Remote relay enabled after restart",
        (false, false, true) => "Local only after restart",
        (false, true, false) => "Remote relay enabled",
        (false, false, false) => "Local only",
    }
}

fn remote_config_status(remote_enabled: bool, language: &str) -> String {
    if host_language_is_zh(language) {
        format!(
            "\u{8fdc}\u{7a0b}\u{4e2d}\u{7ee7}\u{5f53}\u{524d}\u{4e3a}{}\u{3002}\u{66f4}\u{6539}\u{4f1a}\u{4fdd}\u{5b58}\u{5230} host-config.json\u{ff0c}\u{5e76}\u{5728}\u{91cd}\u{65b0}\u{6253}\u{5f00} Windows Host \u{540e}\u{751f}\u{6548}\u{3002}",
            if remote_enabled {
                "\u{6253}\u{5f00}"
            } else {
                "\u{5173}\u{95ed}"
            }
        )
    } else {
        format!(
            "Remote relay is {}. Changes are saved to host-config.json and take effect after restart.",
            if remote_enabled { "enabled" } else { "disabled" }
        )
    }
}

fn remote_status_summary(
    remote_enabled: bool,
    health_url: &str,
    gateway_url: &str,
    package_dir: Option<&Path>,
    restart_required: bool,
    language: &str,
) -> String {
    let mode = remote_mode_label(remote_enabled, restart_required, language);
    let restart_note = if restart_required {
        if host_language_is_zh(language) {
            " | \u{9700}\u{8981}\u{91cd}\u{65b0}\u{6253}\u{5f00} Windows Host \u{751f}\u{6548}"
        } else {
            " | restart required"
        }
    } else {
        ""
    };
    let base = if host_language_is_zh(language) {
        if remote_enabled {
            format!(
                "{mode} | \u{672c}\u{5730} API {health_url} | \u{7f51}\u{5173} {gateway_url}{restart_note}"
            )
        } else {
            format!("{mode} | \u{672c}\u{5730} API {health_url}{restart_note}")
        }
    } else if remote_enabled {
        format!("{mode} | local API {health_url} | gateway {gateway_url}{restart_note}")
    } else {
        format!("{mode} | local API {health_url}{restart_note}")
    };

    if let Some(package_dir) = package_dir {
        if host_language_is_zh(language) {
            format!(
                "{base} | \u{5305} {package}",
                package = package_dir.display()
            )
        } else {
            format!(
                "{base} | package {package}",
                package = package_dir.display()
            )
        }
    } else {
        base
    }
}

fn remote_save_success_message(language: &str) -> &'static str {
    if host_language_is_zh(language) {
        "\u{8fdc}\u{7a0b} Web \u{914d}\u{7f6e}\u{5df2}\u{4fdd}\u{5b58}\u{3002}\u{8bf7}\u{91cd}\u{65b0}\u{6253}\u{5f00} Windows Host \u{8ba9}\u{4e2d}\u{7ee7}\u{914d}\u{7f6e}\u{751f}\u{6548}\u{3002}"
    } else {
        "Remote Web config saved. Restart Windows Host for relay changes to take effect."
    }
}

fn remote_save_error_message(error: &anyhow::Error, language: &str) -> String {
    if host_language_is_zh(language) {
        format!(
            "\u{8fdc}\u{7a0b} Web \u{914d}\u{7f6e}\u{4fdd}\u{5b58}\u{5931}\u{8d25}\u{ff1a}{error}"
        )
    } else {
        format!("Remote Web config save failed: {error}")
    }
}

fn connection_test_running_label(language: &str) -> &'static str {
    if host_language_is_zh(language) {
        "\u{6d4b}\u{8bd5}\u{4e2d}"
    } else {
        "Testing"
    }
}

fn connection_test_running_message(target: &str, language: &str) -> String {
    if host_language_is_zh(language) {
        format!("\u{6b63}\u{5728}\u{6d4b}\u{8bd5}{target}...")
    } else {
        format!("Testing {target}...")
    }
}

fn nine_grid_running_label(language: &str) -> &'static str {
    if host_language_is_zh(language) {
        "\u{6d4b}\u{8bd5}\u{4e2d}"
    } else {
        "Testing"
    }
}

fn nine_grid_running_message(tier: &str, language: &str) -> String {
    let label = match tier {
        "confidential" => {
            if host_language_is_zh(language) {
                "\u{6536}\u{96c6} 12 \u{683c}\u{5bf9}\u{8c61}"
            } else {
                "12-cell Object"
            }
        }
        "top-secret" => {
            if host_language_is_zh(language) {
                "\u{6536}\u{96c6} 22 \u{683c}\u{5bf9}\u{8c61}"
            } else {
                "22-cell Object"
            }
        }
        _ => {
            if host_language_is_zh(language) {
                "\u{6536}\u{96c6} 6 \u{683c}\u{5bf9}\u{8c61}"
            } else {
                "6-cell Object"
            }
        }
    };
    if host_language_is_zh(language) {
        format!("\u{6b63}\u{5728}\u{6d4b}\u{8bd5}{label}...")
    } else {
        format!("Testing {label}...")
    }
}

fn restart_message(language: &str) -> &'static str {
    if host_language_is_zh(language) {
        "\u{6b63}\u{5728}\u{91cd}\u{542f} Windows Host..."
    } else {
        "Restarting Windows Host..."
    }
}

fn restart_error_message(error: &anyhow::Error, language: &str) -> String {
    if host_language_is_zh(language) {
        format!("\u{91cd}\u{542f} Windows Host \u{5931}\u{8d25}\u{ff1a}{error}")
    } else {
        format!("Windows Host restart failed: {error}")
    }
}

fn restart_windows_host_process() -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    std::process::Command::new(current_exe)
        .args(args)
        .env("YIAN_WINDOWS_HOST_RESTART_DELAY_MS", "700")
        .spawn()?;
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(120));
        std::process::exit(0);
    });
    Ok(())
}

pub fn run(config: WindowsHostConfig) -> Result<()> {
    let app = HostDashboard::new()?;
    app.set_product(SharedString::from(config.product.clone()));
    app.set_version(SharedString::from(config.version.clone()));
    app.set_identity_id(SharedString::from(config.identity_id.clone()));
    app.set_device_id(SharedString::from(config.device_id.clone()));
    app.set_remote_relay_enabled(config.remote_enabled);
    app.set_gateway_url(SharedString::from(config.gateway_base_url.clone()));
    app.set_management_stats(SharedString::from(format!(
        "Live redacted statistics are available at http://127.0.0.1:{}/ui and /ui/status.\n\nYian Host may show authorization modes, required grid cells, managed-object call counts, agent/requester counts, remote-interface access counts, and error-call totals.\n\nStory template candidates can be generated by Host and queued at /story-template/generate; StoryLock must explicitly pull them from /story-template/candidates. Host never invokes StoryLock.\n\nLLM keys are direct-access generator config. Host may show configured/missing, but must not display key values.\n\nIt must not display StoryLock drafts, vault files, package paths, question answers, passwords, private keys, signing key bytes, shared secrets, or raw story text.",
        config.host_port
    )));
    app.set_diagnostics(SharedString::from(
        "Yian Host is storage-blind. It does not read or display StoryLock drafts, vault files, manifests, catalogs, templates, package paths, question answers, passwords, private keys, signing key bytes, shared secrets, or raw story text.\n\nThe local API exists for loopback health checks and diagnostics only. Remote capability names and call chains are surfaced in observation mode, not as user workflow steps.",
    ));
    let saved_host_settings = Rc::new(RefCell::new(load_host_ui_settings()));
    let saved_settings = Rc::new(RefCell::new(load_storylock_ui_settings()));
    let initial_language = saved_host_settings
        .borrow()
        .language
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| String::from("zh"));
    app.set_language(SharedString::from(initial_language.clone()));
    app.set_mode(SharedString::from(remote_mode_label(
        config.remote_enabled,
        false,
        &initial_language,
    )));
    app.set_status_summary(SharedString::from(remote_status_summary(
        config.remote_enabled,
        &config.health_url,
        &config.gateway_base_url,
        None,
        false,
        &initial_language,
    )));
    app.set_remote_config_status(SharedString::from(remote_config_status(
        config.remote_enabled,
        &initial_language,
    )));
    let host_language = Rc::new(RefCell::new(initial_language));
    let remote_restart_required = Rc::new(RefCell::new(false));

    let local_health_url = config.health_url.clone();
    let host_for_local_test = app.as_weak();
    let host_language_for_local_test = Rc::clone(&host_language);
    app.on_test_local_host(move || {
        if let Some(host) = host_for_local_test.upgrade() {
            if host.get_connection_test_running() {
                return;
            }
            let language = host_language_for_local_test.borrow().clone();
            host.set_connection_test_running(true);
            host.set_connection_test_badge_label(SharedString::from(
                connection_test_running_label(&language),
            ));
            host.set_connection_test_badge_tone(SharedString::from("neutral"));
            host.set_connection_test_status(SharedString::from(connection_test_running_message(
                "Local Host",
                &language,
            )));
            let weak = host.as_weak();
            let local_health_url = local_health_url.clone();
            std::thread::spawn(move || {
                let status = test_http_endpoint("Local Host", &local_health_url);
                let (label, tone) = connection_badge_for_status(&status);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(host) = weak.upgrade() {
                        host.set_connection_test_status(SharedString::from(status));
                        host.set_connection_test_badge_label(SharedString::from(label));
                        host.set_connection_test_badge_tone(SharedString::from(tone));
                        host.set_connection_test_running(false);
                    }
                });
            });
        }
    });

    let host_for_remote_test = app.as_weak();
    let host_language_for_remote_test = Rc::clone(&host_language);
    app.on_test_remote_connection(move || {
        if let Some(host) = host_for_remote_test.upgrade() {
            if host.get_connection_test_running() {
                return;
            }
            let language = host_language_for_remote_test.borrow().clone();
            let gateway_url = host
                .get_gateway_url()
                .to_string()
                .trim()
                .trim_end_matches('/')
                .to_string();
            let gateway_url = if gateway_url.is_empty() {
                "https://yian.cdao.online".to_string()
            } else {
                gateway_url
            };
            host.set_connection_test_running(true);
            host.set_connection_test_badge_label(SharedString::from(
                connection_test_running_label(&language),
            ));
            host.set_connection_test_badge_tone(SharedString::from("neutral"));
            host.set_connection_test_status(SharedString::from(connection_test_running_message(
                "Remote Gateway",
                &language,
            )));
            let weak = host.as_weak();
            std::thread::spawn(move || {
                let status = test_http_endpoint("Remote Gateway", &gateway_url);
                let (label, tone) = connection_badge_for_status(&status);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(host) = weak.upgrade() {
                        host.set_connection_test_status(SharedString::from(status));
                        host.set_connection_test_badge_label(SharedString::from(label));
                        host.set_connection_test_badge_tone(SharedString::from(tone));
                        host.set_connection_test_running(false);
                    }
                });
            });
        }
    });

    let remote_config_dir = config.data_dir.clone();
    let remote_health_url = config.health_url.clone();
    let host_for_remote_save = app.as_weak();
    let host_language_for_remote_save = Rc::clone(&host_language);
    let remote_restart_required_for_save = Rc::clone(&remote_restart_required);
    app.on_save_remote_web_config(move |enabled, gateway_url| {
        let gateway_url = gateway_url.to_string();
        let language = host_language_for_remote_save.borrow().clone();
        match crate::host_runtime::state::save_host_remote_config(
            &remote_config_dir,
            enabled,
            &gateway_url,
        ) {
            Ok(()) => {
                let normalized_gateway = gateway_url.trim().trim_end_matches('/').to_string();
                *remote_restart_required_for_save.borrow_mut() = true;
                if let Some(host) = host_for_remote_save.upgrade() {
                    host.set_remote_restart_required(true);
                    host.set_gateway_url(SharedString::from(normalized_gateway.clone()));
                    host.set_mode(SharedString::from(remote_mode_label(
                        enabled, true, &language,
                    )));
                    host.set_status_summary(SharedString::from(remote_status_summary(
                        enabled,
                        &remote_health_url,
                        &normalized_gateway,
                        None,
                        true,
                        &language,
                    )));
                }
                SharedString::from(remote_save_success_message(&language))
            }
            Err(error) => SharedString::from(remote_save_error_message(&error, &language)),
        }
    });

    let host_language_for_restart = Rc::clone(&host_language);
    app.on_restart_windows_host(move || {
        let language = host_language_for_restart.borrow().clone();
        match restart_windows_host_process() {
            Ok(()) => SharedString::from(restart_message(&language)),
            Err(error) => SharedString::from(restart_error_message(&error, &language)),
        }
    });

    let host_port_for_nine_grid = config.host_port;
    let host_for_nine_grid = app.as_weak();
    let host_language_for_nine_grid = Rc::clone(&host_language);
    app.on_test_managed_object_nine_grid(move |tier| {
        let language = host_language_for_nine_grid.borrow().clone();
        if let Some(host) = host_for_nine_grid.upgrade() {
            host.set_nine_grid_running(true);
            host.set_nine_grid_badge_label(SharedString::from(nine_grid_running_label(&language)));
            host.set_nine_grid_badge_tone(SharedString::from("neutral"));
            host.set_nine_grid_status(SharedString::from(nine_grid_running_message(
                &tier, &language,
            )));
        }
        let status = test_managed_object_nine_grid(host_port_for_nine_grid, tier.to_string());
        let (label, tone) = nine_grid_badge_for_status(&status);
        if let Some(host) = host_for_nine_grid.upgrade() {
            host.set_nine_grid_badge_label(SharedString::from(label));
            host.set_nine_grid_badge_tone(SharedString::from(tone));
            host.set_nine_grid_running(false);
            host.set_nine_grid_status(SharedString::from(status.clone()));
        }
        SharedString::from(status)
    });

    let normalized_settings = {
        let mut settings = saved_settings.borrow().clone();
        normalize_storylock_ui_settings(&mut settings);
        settings
    };
    *saved_settings.borrow_mut() = normalized_settings.clone();
    if let Err(error) = save_storylock_ui_settings(&normalized_settings) {
        eprintln!("failed to normalize StoryLock UI settings: {error}");
    }
    if let Err(error) = save_host_ui_settings(&saved_host_settings.borrow()) {
        eprintln!("failed to save Host UI settings: {error}");
    }
    if let Err(error) = retire_legacy_combined_ui_settings_if_split() {
        eprintln!("failed to retire legacy combined UI settings: {error}");
    }
    if let Err(error) = cleanup_legacy_host_config_storylock_templates() {
        eprintln!("failed to clean legacy StoryLock templates from Host config dir: {error}");
    }
    let core_package_dir = resolve_storylock_core_package_with_conflict_prompt(
        &initial_storylock_core_package_dir(&normalized_settings),
        normalized_settings
            .export_package_dir
            .as_deref()
            .map(Path::new),
        Some(&config.data_dir),
    )?;
    let _ = ensure_storylock_core_package(&core_package_dir);
    app.set_storylock_data_dir(SharedString::from(core_package_dir.display().to_string()));
    app.set_package_self_check(SharedString::from(package_dir_status_report(
        &core_package_dir,
    )));
    app.set_status_summary(SharedString::from(remote_status_summary(
        config.remote_enabled,
        &config.health_url,
        &config.gateway_base_url,
        Some(&core_package_dir),
        false,
        &host_language.borrow(),
    )));
    app.set_management_stats(SharedString::from(format!(
        "Live redacted statistics are available at http://127.0.0.1:{}/ui and /ui/status.\n\n{}\n\nYian Host reads learning-policy.json for retention-learning scheduling, but does not read StoryLock drafts, vault files, question text, answers, passwords, private keys, signing key bytes, shared secrets, or raw story text.",
        config.host_port,
        host_learning_plan_status(&core_package_dir)
    )));

    let core_window: Rc<RefCell<Option<StoryLockCoreApp>>> = Rc::new(RefCell::new(None));
    let settings_window: Rc<RefCell<Option<SettingsDialog>>> = Rc::new(RefCell::new(None));
    let storylock_auth_window: Rc<RefCell<Option<StoryLockAuthorizationDialog>>> =
        Rc::new(RefCell::new(None));
    let settings_window_for_open = Rc::clone(&settings_window);
    let host_for_settings = app.as_weak();
    let host_language_for_settings = Rc::clone(&host_language);
    let core_window_for_settings = Rc::clone(&core_window);
    let host_for_settings_lock = app.as_weak();
    let shared_status = Rc::new(RefCell::new(String::from("")));
    let saved_host_settings_for_settings = Rc::clone(&saved_host_settings);
    let host_for_main_browse = app.as_weak();
    let host_language_for_main_browse = Rc::clone(&host_language);
    let saved_settings_for_main_browse = Rc::clone(&saved_settings);
    let core_window_for_main_browse = Rc::clone(&core_window);

    app.on_browse_storylock_data_dir(move || {
        let current_dir =
            initial_storylock_core_package_dir(&saved_settings_for_main_browse.borrow());
        let Some(selected_path) = pick_storylock_core_package_path(current_dir.as_path()) else {
            return;
        };
        let package_dir = resolve_storylock_core_package_path(&selected_path);
        match ensure_storylock_core_package(&package_dir) {
            Ok(()) => {
                let language = host_language_for_main_browse.borrow().clone();
                let next_settings = merge_storylock_package_settings(
                    &saved_settings_for_main_browse.borrow(),
                    Some(package_dir.display().to_string()),
                );
                if let Err(error) = save_storylock_ui_settings(&next_settings) {
                    if let Some(host) = host_for_main_browse.upgrade() {
                        host.set_connection_test_status(SharedString::from(format!(
                            "StoryLock encrypted files path save failed: {error}"
                        )));
                    }
                    return;
                }
                *saved_settings_for_main_browse.borrow_mut() = next_settings;
                if let Some(core) = core_window_for_main_browse.borrow().as_ref() {
                    initialize_storylock_core_empty_window(core, &package_dir);
                    core.set_language(SharedString::from(language));
                }
                if let Some(host) = host_for_main_browse.upgrade() {
                    host.set_storylock_data_dir(SharedString::from(
                        package_dir.display().to_string(),
                    ));
                    host.set_package_self_check(SharedString::from(package_dir_status_report(
                        &package_dir,
                    )));
                    host.set_connection_test_status(SharedString::from(
                        "StoryLock package directory selected.",
                    ));
                }
            }
            Err(error) => {
                if let Some(host) = host_for_main_browse.upgrade() {
                    host.set_connection_test_status(SharedString::from(format!(
                        "StoryLock package load failed: {error}"
                    )));
                }
            }
        }
    });

    app.on_open_settings(move || {
        let existing_show_result = {
            settings_window_for_open
                .borrow()
                .as_ref()
                .map(|existing| existing.show())
        };
        if let Some(show_result) = existing_show_result {
            if let Err(error) = show_result {
                eprintln!("failed to show settings window: {error}");
                *settings_window_for_open.borrow_mut() = None;
                if let Some(host) = host_for_settings.upgrade() {
                    host.set_settings_open(false);
                    host.set_connection_test_status(SharedString::from(format!(
                        "Settings reopen failed: {error}"
                    )));
                }
            } else if let Some(host) = host_for_settings.upgrade() {
                host.set_settings_open(true);
                host.set_connection_test_status(SharedString::from("Settings opened"));
            }
            return;
        }

        match SettingsDialog::new() {
            Ok(settings) => {
                settings.set_language(SharedString::from(
                    host_language_for_settings.borrow().clone(),
                ));
                settings.set_host_config_file(SharedString::from(
                    host_ui_settings_path().display().to_string(),
                ));

                if let Some(host) = host_for_settings.upgrade() {
                    host.set_connection_test_status(SharedString::from("Settings opened"));
                    host.set_settings_open(true);
                }

                let settings_weak = settings.as_weak();
                let settings_weak_for_close = settings_weak.clone();
                let host_for_language = host_for_settings.clone();
                let host_language_for_language = Rc::clone(&host_language_for_settings);
                let core_window_for_language = Rc::clone(&core_window_for_settings);
                let saved_host_settings_for_language = Rc::clone(&saved_host_settings_for_settings);
                let remote_restart_required_for_language = Rc::clone(&remote_restart_required);
                let health_url_for_language = config.health_url.clone();
                let core_package_dir_for_language = core_package_dir.clone();

                settings.on_language_changed(move |language| {
                    let language_string = language.to_string();
                    *host_language_for_language.borrow_mut() = language_string.clone();
                    let next_settings = merge_host_settings(
                        &saved_host_settings_for_language.borrow(),
                        &language_string,
                    );
                    if let Err(error) = save_host_ui_settings(&next_settings) {
                        eprintln!("failed to save Host UI settings: {error}");
                    }
                    *saved_host_settings_for_language.borrow_mut() = next_settings;

                    if let Some(settings) = settings_weak.upgrade() {
                        settings.set_language(SharedString::from(language_string.clone()));
                    }
                    if let Some(host) = host_for_language.upgrade() {
                        let remote_enabled = host.get_remote_relay_enabled();
                        let gateway_url = host.get_gateway_url().to_string();
                        let restart_required = *remote_restart_required_for_language.borrow();
                        host.set_language(SharedString::from(language_string.clone()));
                        host.set_mode(SharedString::from(remote_mode_label(
                            remote_enabled,
                            restart_required,
                            &language_string,
                        )));
                        host.set_status_summary(SharedString::from(remote_status_summary(
                            remote_enabled,
                            &health_url_for_language,
                            &gateway_url,
                            Some(&core_package_dir_for_language),
                            restart_required,
                            &language_string,
                        )));
                        host.set_remote_config_status(SharedString::from(remote_config_status(
                            remote_enabled,
                            &language_string,
                        )));
                        host.set_connection_test_status(SharedString::from(
                            if host_language_is_zh(&language_string) {
                                "\u{8bed}\u{8a00}\u{5df2}\u{5207}\u{6362}\u{ff0c}\u{8bbe}\u{7f6e}\u{7a97}\u{53e3}\u{4fdd}\u{6301}\u{6253}\u{5f00}"
                            } else {
                                "Language changed, settings stay open"
                            },
                        ));
                    }
                    if let Some(core) = core_window_for_language.borrow().as_ref() {
                        core.set_language(SharedString::from(language_string));
                    }
                });

                let settings_weak_for_config_browse = settings.as_weak();
                let host_for_browse = host_for_settings.clone();
                settings.on_browse_host_config_file(move || {
                    let result = browse_host_config_file_path();
                    match result {
                        Ok(Some(path)) => {
                            if let Some(settings) = settings_weak_for_config_browse.upgrade() {
                                settings
                                    .set_host_config_file(SharedString::from(path.display().to_string()));
                            }
                            if let Some(host) = host_for_browse.upgrade() {
                                host.set_connection_test_status(SharedString::from(
                                    "Host config file selected.",
                                ));
                            }
                        }
                        Ok(None) => {
                            if let Some(host) = host_for_browse.upgrade() {
                                host.set_connection_test_status(SharedString::from(
                                    "Host config file selection cancelled.",
                                ));
                            }
                        }
                        Err(error) => {
                            if let Some(host) = host_for_browse.upgrade() {
                                host.set_connection_test_status(SharedString::from(format!(
                                    "Host config file selection failed: {error}"
                                )));
                            }
                        }
                    }
                });

                let host_for_settings_lock_close = host_for_settings_lock.clone();
                let host_language_for_close = Rc::clone(&host_language_for_settings);
                let saved_host_settings_for_close = Rc::clone(&saved_host_settings_for_settings);
                let settings_window_for_close = Rc::clone(&settings_window_for_open);
                let close_settings = Rc::new(move |settings: &SettingsDialog| {
                    let next_settings = merge_host_settings(
                        &saved_host_settings_for_close.borrow(),
                        &host_language_for_close.borrow(),
                    );
                    if let Err(error) = save_host_ui_settings(&next_settings) {
                        eprintln!("failed to save Host UI settings: {error}");
                    } else {
                        *saved_host_settings_for_close.borrow_mut() = next_settings;
                    }
                    let _ = settings.hide();
                    *settings_window_for_close.borrow_mut() = None;
                    if let Some(host) = host_for_settings_lock_close.upgrade() {
                        host.set_settings_open(false);
                        let text = if host_language_for_close.borrow().as_str() == "zh" {
                            "设置已关闭"
                        } else {
                            "Settings closed"
                        };
                        host.set_connection_test_status(SharedString::from(text));
                    }
                });

                let close_settings_for_button = Rc::clone(&close_settings);
                settings.on_close_requested(move || {
                    if let Some(settings) = settings_weak_for_close.upgrade() {
                        close_settings_for_button(&settings);
                    }
                });

                let settings_weak_for_window_close = settings.as_weak();
                let close_settings_for_window = Rc::clone(&close_settings);
                settings.window().on_close_requested(move || {
                    if let Some(settings) = settings_weak_for_window_close.upgrade() {
                        close_settings_for_window(&settings);
                    }
                    slint::CloseRequestResponse::HideWindow
                });

                if let Err(error) = settings.show() {
                    eprintln!("failed to show settings window: {error}");
                    return;
                }
                *settings_window_for_open.borrow_mut() = Some(settings);
            }
            Err(error) => eprintln!("failed to create settings window: {error}"),
        }
    });

    let core_window_for_callback = Rc::clone(&core_window);
    let host_for_storylock_close = app.as_weak();
    let settings_window_for_storylock_close = Rc::clone(&settings_window);
    let shared_status_for_storylock_close = Rc::clone(&shared_status);
    let host_language_for_storylock = Rc::clone(&host_language);
    let saved_settings_for_storylock = Rc::clone(&saved_settings);
    let storylock_auth_window_for_open = Rc::clone(&storylock_auth_window);
    let host_port_for_storylock_auth = config.host_port;

    app.on_open_storylock_core(move || {
        let Some(host) = host_for_storylock_close.upgrade() else {
            return;
        };
        host.set_connection_test_status(SharedString::from(
            "StoryLock opened in empty mode. Select or confirm the current package, then unlock it to load current package content.",
        ));
        open_storylock_core_in_empty_mode(
            host_for_storylock_close.clone(),
            Rc::clone(&core_window_for_callback),
            Rc::clone(&settings_window_for_storylock_close),
            Rc::clone(&shared_status_for_storylock_close),
            Rc::clone(&host_language_for_storylock),
            Rc::clone(&saved_settings_for_storylock),
            Rc::clone(&storylock_auth_window_for_open),
            host_port_for_storylock_auth,
        );
    });

    let weak = app.as_weak();
    let host_language_for_close = Rc::clone(&host_language);
    let saved_host_settings_for_close = Rc::clone(&saved_host_settings);
    app.on_close_requested(move || {
        let next_settings = merge_host_settings(
            &saved_host_settings_for_close.borrow(),
            &host_language_for_close.borrow(),
        );
        if let Err(error) = save_host_ui_settings(&next_settings) {
            eprintln!("failed to save Host UI settings: {error}");
        }
        *saved_host_settings_for_close.borrow_mut() = next_settings;
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
    });

    app.run()?;
    let final_settings =
        merge_host_settings(&saved_host_settings.borrow(), &host_language.borrow());
    if let Err(error) = save_host_ui_settings(&final_settings) {
        eprintln!("failed to save Host UI settings: {error}");
    }
    Ok(())
}

fn open_storylock_core_in_empty_mode(
    host_weak: slint::Weak<HostDashboard>,
    core_window: Rc<RefCell<Option<StoryLockCoreApp>>>,
    settings_window: Rc<RefCell<Option<SettingsDialog>>>,
    shared_status: Rc<RefCell<String>>,
    host_language: Rc<RefCell<String>>,
    saved_settings: Rc<RefCell<StoryLockUiSettings>>,
    auth_window: Rc<RefCell<Option<StoryLockAuthorizationDialog>>>,
    host_port: u16,
) {
    if let Some(host) = host_weak.upgrade() {
        host.set_connection_test_status(SharedString::from(
            "StoryLock Core opened in empty mode. Current package content stays unloaded until unlock.",
        ));
    }
    let current_settings = saved_settings.borrow().clone();
    let active_package_dir = initial_storylock_core_package_dir(&current_settings);
    if let Err(error) = ensure_storylock_core_package(&active_package_dir) {
        eprintln!("failed to initialize StoryLock Core package: {error}");
    }

    if let Some(core) = core_window.borrow().as_ref() {
        initialize_storylock_core_empty_window(core, &active_package_dir);
        core.set_language(SharedString::from(host_language.borrow().clone()));
        apply_storylock_ui_settings(core, &current_settings);
        if let Err(error) = core.show() {
            eprintln!("failed to show existing StoryLock Core window: {error}");
        }
        return;
    }

    *core_window.borrow_mut() = None;
    match StoryLockCoreApp::new() {
        Ok(core) => {
            core.set_language(SharedString::from(host_language.borrow().clone()));
            initialize_storylock_core_empty_window(&core, &active_package_dir);
            apply_storylock_ui_settings(&core, &current_settings);
            let host_for_closed = host_weak.clone();
            let shared_status_for_closed = Rc::clone(&shared_status);
            let notify_storylock_closed: Rc<dyn Fn()> = Rc::new(move || {
                let status = "StoryLock closed".to_string();
                *shared_status_for_closed.borrow_mut() = status.clone();
                if let Some(host) = host_for_closed.upgrade() {
                    host.set_connection_test_status(SharedString::from(status.clone()));
                }
            });
            let host_for_unlock = host_weak.clone();
            let core_window_for_unlock = Rc::clone(&core_window);
            let settings_window_for_unlock = Rc::clone(&settings_window);
            let shared_status_for_unlock = Rc::clone(&shared_status);
            let host_language_for_unlock = Rc::clone(&host_language);
            let saved_settings_for_unlock = Rc::clone(&saved_settings);
            let auth_window_for_unlock = Rc::clone(&auth_window);
            let unlock_package: Rc<dyn Fn()> = Rc::new(move || {
                if let Some(host) = host_for_unlock.upgrade() {
                    host.set_connection_test_status(SharedString::from(
                        "StoryLock unlock requested. Complete the current package challenge to load current package content.",
                    ));
                }
                if let Err(error) = begin_storylock_open_authorization(
                    host_port,
                    host_for_unlock.clone(),
                    Rc::clone(&core_window_for_unlock),
                    Rc::clone(&settings_window_for_unlock),
                    Rc::clone(&shared_status_for_unlock),
                    Rc::clone(&host_language_for_unlock),
                    Rc::clone(&saved_settings_for_unlock),
                    Rc::clone(&auth_window_for_unlock),
                ) {
                    if let Some(host) = host_for_unlock.upgrade() {
                        host.set_connection_test_status(SharedString::from(format!(
                            "StoryLock unlock blocked: {error}"
                        )));
                    }
                }
            });
            wire_storylock_core_callbacks(
                &core,
                active_package_dir,
                Rc::clone(&core_window),
                notify_storylock_closed,
                unlock_package,
                host_port,
            );
            if let Err(error) = core.show() {
                eprintln!("failed to show StoryLock Core window: {error}");
                return;
            }
            *core_window.borrow_mut() = Some(core);
        }
        Err(error) => {
            eprintln!("failed to create StoryLock Core window: {error}");
        }
    }
}

fn open_storylock_core_after_authorization(
    host_weak: slint::Weak<HostDashboard>,
    core_window: Rc<RefCell<Option<StoryLockCoreApp>>>,
    settings_window: Rc<RefCell<Option<SettingsDialog>>>,
    shared_status: Rc<RefCell<String>>,
    host_language: Rc<RefCell<String>>,
    saved_settings: Rc<RefCell<StoryLockUiSettings>>,
    host_port: u16,
) {
    if let Some(host) = host_weak.upgrade() {
        host.set_connection_test_status(SharedString::from(
            "StoryLock authorization passed. Current package content is now loading.",
        ));
    }
    let current_settings = saved_settings.borrow().clone();
    let active_package_dir = initial_storylock_core_package_dir(&current_settings);
    if let Err(error) = ensure_storylock_core_package(&active_package_dir) {
        eprintln!("failed to initialize StoryLock Core package: {error}");
    }

    if let Some(core) = core_window.borrow().as_ref() {
        initialize_storylock_core_window(core, &active_package_dir);
        core.set_language(SharedString::from(host_language.borrow().clone()));
        apply_storylock_ui_settings(core, &current_settings);
        set_storylock_start_page_to_questions(core);
        core.set_config_status(SharedString::from(
            "StoryLock Core is already open. Existing local window was focused.",
        ));
        if let Err(error) = core.show() {
            eprintln!("failed to show existing StoryLock Core window: {error}");
        }
        return;
    }

    *core_window.borrow_mut() = None;
    match StoryLockCoreApp::new() {
        Ok(core) => {
            core.set_language(SharedString::from(host_language.borrow().clone()));
            initialize_storylock_core_window(&core, &active_package_dir);
            apply_storylock_ui_settings(&core, &current_settings);
            set_storylock_start_page_to_questions(&core);
            let host_for_closed = host_weak.clone();
            let shared_status_for_closed = Rc::clone(&shared_status);
            let notify_storylock_closed: Rc<dyn Fn()> = Rc::new(move || {
                let status = "StoryLock closed".to_string();
                *shared_status_for_closed.borrow_mut() = status.clone();
                if let Some(host) = host_for_closed.upgrade() {
                    host.set_connection_test_status(SharedString::from(status.clone()));
                }
            });
            let unlock_package: Rc<dyn Fn()> = Rc::new(|| {});
            wire_storylock_core_callbacks(
                &core,
                active_package_dir,
                Rc::clone(&core_window),
                notify_storylock_closed,
                unlock_package,
                host_port,
            );
            if let Err(error) = core.show() {
                eprintln!("failed to show StoryLock Core window: {error}");
                return;
            }
            *core_window.borrow_mut() = Some(core);
        }
        Err(error) => {
            eprintln!("failed to create StoryLock Core window: {error}");
        }
    }
}

fn test_http_endpoint(label: &str, url: &str) -> String {
    let client = match Client::builder().timeout(Duration::from_secs(5)).build() {
        Ok(client) => client,
        Err(error) => return format!("{label}: client setup failed: {error}"),
    };
    match client.get(url).send() {
        Ok(response) if response.status().is_success() => {
            format!("{label}: OK ({})", response.status())
        }
        Ok(response) => format!("{label}: HTTP {}", response.status()),
        Err(error) => format!("{label}: failed: {error}"),
    }
}

fn connection_badge_for_status(status: &str) -> (&'static str, &'static str) {
    let lower = status.to_ascii_lowercase();
    if lower.contains(": ok") {
        ("OK", "success")
    } else if lower.contains("failed")
        || lower.contains("error")
        || lower.contains("blocked")
        || lower.contains(": http ")
    {
        ("Failed", "warning")
    } else {
        ("Updated", "neutral")
    }
}

fn nine_grid_badge_for_status(status: &str) -> (&'static str, &'static str) {
    let lower = status.to_ascii_lowercase();
    if lower.contains("nine-grid ready") {
        ("Ready", "success")
    } else if lower.contains("failed") || lower.contains("error") {
        ("Failed", "warning")
    } else {
        ("Updated", "neutral")
    }
}

fn begin_storylock_open_authorization(
    host_port: u16,
    host_weak: slint::Weak<HostDashboard>,
    core_window: Rc<RefCell<Option<StoryLockCoreApp>>>,
    settings_window: Rc<RefCell<Option<SettingsDialog>>>,
    shared_status: Rc<RefCell<String>>,
    host_language: Rc<RefCell<String>>,
    saved_settings: Rc<RefCell<StoryLockUiSettings>>,
    auth_window: Rc<RefCell<Option<StoryLockAuthorizationDialog>>>,
) -> Result<()> {
    let existing_show_result = auth_window
        .borrow()
        .as_ref()
        .map(|existing| existing.show());
    if let Some(show_result) = existing_show_result {
        if let Err(error) = show_result {
            *auth_window.borrow_mut() = None;
            return Err(anyhow::anyhow!(
                "authorization window reopen failed: {error}"
            ));
        }
        return Ok(());
    }

    let language = host_language.borrow().clone();
    let active_package_dir = initial_storylock_core_package_dir(&saved_settings.borrow());
    ensure_storylock_core_package(&active_package_dir)?;
    let cells = create_storylock_open_challenge(&active_package_dir, 22)?;
    let expected_answers = storylock_open_expected_answers(&active_package_dir, cells.len())?;

    let dialog = StoryLockAuthorizationDialog::new()?;
    dialog.set_is_zh(language == "zh");
    dialog.set_challenge_count(cells.len() as i32);
    let selections = Rc::new(RefCell::new(vec![Vec::<String>::new(); cells.len()]));
    let current_index = Rc::new(Cell::new(0usize));
    set_storylock_challenge_question(&dialog, &cells, &selections.borrow(), current_index.get());

    let dialog_weak = dialog.as_weak();
    let host_for_auth = host_weak.clone();
    let cells_for_auth = cells.clone();
    let expected_answers_for_auth = expected_answers.clone();
    let core_window_for_auth = Rc::clone(&core_window);
    let settings_window_for_auth = Rc::clone(&settings_window);
    let shared_status_for_auth = Rc::clone(&shared_status);
    let host_language_for_auth = Rc::clone(&host_language);
    let saved_settings_for_auth = Rc::clone(&saved_settings);
    let auth_window_for_auth = Rc::clone(&auth_window);
    let selections_for_auth = Rc::clone(&selections);
    let current_index_for_auth = Rc::clone(&current_index);
    dialog.on_authorize_requested(move || {
        let Some(dialog) = dialog_weak.upgrade() else {
            return;
        };
        let answers = selections_for_auth.borrow().clone();
        if answers.iter().any(Vec::is_empty) {
            show_storylock_authorization_result(
                "StoryLock 九宫格挑战",
                "挑战未完成：请完成全部 22 题后再授权。",
                false,
            );
            if let Some(host) = host_for_auth.upgrade() {
                host.set_connection_test_status(SharedString::from(
                    "StoryLock open blocked: complete every grid challenge selection first.",
                ));
            }
            return;
        }
        match authorize_storylock_open(&answers, &expected_answers_for_auth) {
            Ok(()) => {
                show_storylock_authorization_result(
                    "StoryLock 九宫格挑战",
                    "挑战通过，即将进入 StoryLock 编辑界面。",
                    true,
                );
                let _ = dialog.hide();
                *auth_window_for_auth.borrow_mut() = None;
                open_storylock_core_after_authorization(
                    host_for_auth.clone(),
                    Rc::clone(&core_window_for_auth),
                    Rc::clone(&settings_window_for_auth),
                    Rc::clone(&shared_status_for_auth),
                    Rc::clone(&host_language_for_auth),
                    Rc::clone(&saved_settings_for_auth),
                    host_port,
                );
            }
            Err(error) => {
                let message = format!("StoryLock authorization failed: {error}");
                show_storylock_authorization_result(
                    "StoryLock 九宫格挑战",
                    &format!("挑战失败：{error}"),
                    false,
                );
                {
                    let mut selections = selections_for_auth.borrow_mut();
                    for selection in selections.iter_mut() {
                        selection.clear();
                    }
                }
                current_index_for_auth.set(0);
                set_storylock_challenge_question(
                    &dialog,
                    &cells_for_auth,
                    &selections_for_auth.borrow(),
                    0,
                );
                if let Some(host) = host_for_auth.upgrade() {
                    host.set_connection_test_status(SharedString::from(message));
                }
            }
        }
    });

    let dialog_weak = dialog.as_weak();
    let cells_for_select = cells.clone();
    let selections_for_select = Rc::clone(&selections);
    let current_index_for_select = Rc::clone(&current_index);
    dialog.on_select_answer(move |answer_index| {
        let index = current_index_for_select.get();
        let selected = cells_for_select
            .get(index)
            .and_then(|cell| cell.answer_options.get(answer_index as usize))
            .cloned()
            .unwrap_or_default();
        if !selected.trim().is_empty() {
            toggle_storylock_challenge_selection(
                &cells_for_select,
                &mut selections_for_select.borrow_mut(),
                index,
                answer_index as usize,
            );
        }
        if let Some(dialog) = dialog_weak.upgrade() {
            set_storylock_challenge_question(
                &dialog,
                &cells_for_select,
                &selections_for_select.borrow(),
                index,
            );
        }
    });

    let dialog_weak = dialog.as_weak();
    let cells_for_previous = cells.clone();
    let selections_for_previous = Rc::clone(&selections);
    let current_index_for_previous = Rc::clone(&current_index);
    dialog.on_previous_requested(move || {
        if current_index_for_previous.get() == 0 {
            return;
        }
        let next_index = current_index_for_previous.get().saturating_sub(1);
        current_index_for_previous.set(next_index);
        if let Some(dialog) = dialog_weak.upgrade() {
            set_storylock_challenge_question(
                &dialog,
                &cells_for_previous,
                &selections_for_previous.borrow(),
                next_index,
            );
        }
    });

    let dialog_weak = dialog.as_weak();
    let cells_for_next = cells.clone();
    let selections_for_next = Rc::clone(&selections);
    let current_index_for_next = Rc::clone(&current_index);
    dialog.on_next_requested(move || {
        let max_index = cells_for_next.len().saturating_sub(1);
        if current_index_for_next.get() >= max_index {
            return;
        }
        let next_index = (current_index_for_next.get() + 1).min(max_index);
        current_index_for_next.set(next_index);
        if let Some(dialog) = dialog_weak.upgrade() {
            set_storylock_challenge_question(
                &dialog,
                &cells_for_next,
                &selections_for_next.borrow(),
                next_index,
            );
        }
    });

    let dialog_weak = dialog.as_weak();
    let host_for_cancel = host_weak.clone();
    let auth_window_for_cancel = Rc::clone(&auth_window);
    dialog.on_cancel_requested(move || {
        if let Some(dialog) = dialog_weak.upgrade() {
            let _ = dialog.hide();
        }
        *auth_window_for_cancel.borrow_mut() = None;
        if let Some(host) = host_for_cancel.upgrade() {
            host.set_connection_test_status(SharedString::from(
                "StoryLock open blocked: authorization cancelled.",
            ));
        }
    });

    let dialog_weak = dialog.as_weak();
    let host_for_close = host_weak.clone();
    let auth_window_for_close = Rc::clone(&auth_window);
    dialog.window().on_close_requested(move || {
        if let Some(dialog) = dialog_weak.upgrade() {
            let _ = dialog.hide();
        }
        *auth_window_for_close.borrow_mut() = None;
        if let Some(host) = host_for_close.upgrade() {
            host.set_connection_test_status(SharedString::from(
                "StoryLock open blocked: authorization cancelled.",
            ));
        }
        slint::CloseRequestResponse::HideWindow
    });

    dialog.show()?;
    *auth_window.borrow_mut() = Some(dialog);
    Ok(())
}

pub(super) fn create_storylock_open_challenge(
    package_dir: &Path,
    required_cells: usize,
) -> Result<Vec<StoryLockChallengeCell>> {
    let draft = read_effective_author_draft(package_dir);
    create_open_challenge_from_draft(&draft, required_cells)
}

pub(super) fn authorize_storylock_open(
    answers: &[Vec<String>],
    expected_answers: &[Vec<String>],
) -> Result<()> {
    if answers.len() != expected_answers.len() {
        anyhow::bail!(
            "challenge answer count mismatch: expected {}, got {}",
            expected_answers.len(),
            answers.len()
        );
    }

    for (index, (answer, expected_answer)) in
        answers.iter().zip(expected_answers.iter()).enumerate()
    {
        let selected = answer
            .iter()
            .map(|item| storylock_puzzle_plugin::normalize_answer(item))
            .filter(|item| !item.is_empty())
            .collect::<HashSet<_>>();
        let expected = expected_answer
            .iter()
            .map(|item| storylock_puzzle_plugin::normalize_answer(item))
            .filter(|item| !item.is_empty())
            .collect::<HashSet<_>>();
        if selected != expected {
            anyhow::bail!(
                "challenge cell {} mismatch: selected {}, expected {}",
                index + 1,
                selected.len(),
                expected.len()
            );
        }
    }

    Ok(())
}

pub(super) fn storylock_open_expected_answers(
    package_dir: &Path,
    required_cells: usize,
) -> Result<Vec<Vec<String>>> {
    let draft = read_effective_author_draft(package_dir);
    let nodes = draft
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("StoryLock draft has no question nodes"))?;
    if nodes.len() < required_cells {
        anyhow::bail!("StoryLock draft has fewer than {required_cells} questions");
    }

    let expected_answers = nodes
        .iter()
        .take(required_cells)
        .map(|node| {
            node.get("answerOptionsLocalOnly")
                .and_then(Value::as_array)
                .map(|options| {
                    options
                        .iter()
                        .filter(|option| {
                            option
                                .get("isCorrect")
                                .and_then(Value::as_bool)
                                .unwrap_or(false)
                        })
                        .filter_map(|option| option.get("text").and_then(Value::as_str))
                        .map(storylock_puzzle_plugin::normalize_answer)
                        .filter(|answer| !answer.is_empty())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    if expected_answers.iter().any(Vec::is_empty) {
        anyhow::bail!("StoryLock challenge contains incomplete verification data");
    }

    Ok(expected_answers)
}

pub(super) fn toggle_storylock_challenge_selection(
    cells: &[StoryLockChallengeCell],
    selections: &mut [Vec<String>],
    current_index: usize,
    answer_index: usize,
) {
    storylock_puzzle_plugin::toggle_selection(cells, selections, current_index, answer_index)
}

fn test_managed_object_nine_grid(host_port: u16, tier: String) -> String {
    let client = match Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(client) => client,
        Err(error) => return format!("Nine-grid test failed: client setup failed: {error}"),
    };
    let status_url = format!("http://127.0.0.1:{host_port}/ui/status");
    let status = match client.get(&status_url).send() {
        Ok(response) => match response.json::<Value>() {
            Ok(value) => value,
            Err(error) => {
                return format!("Nine-grid test failed: could not parse ui status: {error}");
            }
        },
        Err(error) => return format!("Nine-grid test failed: could not read ui status: {error}"),
    };
    let management = status
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get("managementStats"))
        .cloned()
        .unwrap_or(Value::Null);
    let object = management
        .get("objects")
        .and_then(Value::as_array)
        .and_then(|objects| objects.first())
        .cloned();
    let Some(object) = object else {
        return "Nine-grid test failed: no managed objects available yet".to_string();
    };
    let object_ref = object
        .get("objectRef")
        .and_then(Value::as_str)
        .unwrap_or("unknown-object");
    let (authorization_channel, requested_action, capability, label) = match tier.as_str() {
        "confidential" => (
            "batch_read",
            "batch_read",
            "requestPasswordFill",
            "Confidential object",
        ),
        "top-secret" => (
            "story_edit",
            "story_edit",
            "requestPasswordFill",
            "Top secret object",
        ),
        _ => (
            "single_read",
            "password_fill",
            "requestPasswordFill",
            "Standard object",
        ),
    };
    let verification_request = json!({
        "requestId": format!("req-nine-grid-{}", Uuid::new_v4()),
        "capability": capability,
        "credentialRef": object_ref,
        "authorizationChannel": authorization_channel,
        "requestedAction": requested_action,
        "requester": "host-ui-nine-grid"
    });
    let verify_url = format!("http://127.0.0.1:{host_port}/verify");
    let verification = match client.post(&verify_url).json(&verification_request).send() {
        Ok(response) => match response.json::<Value>() {
            Ok(value) => value,
            Err(error) => {
                return format!(
                    "Nine-grid test failed: could not parse verification response: {error}"
                );
            }
        },
        Err(error) => return format!("Nine-grid test failed: verify request failed: {error}"),
    };
    let result = verification.get("result").cloned().unwrap_or(Value::Null);
    let verification_id = result
        .get("verificationId")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let grid = result.get("grid").cloned().unwrap_or(Value::Null);
    let required_cells = grid
        .get("requiredCells")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let grid_size = grid.get("gridSize").and_then(Value::as_u64).unwrap_or(0);
    format!(
        "Nine-grid ready: {label}, object={object_ref}, verificationId={verification_id}, {required_cells}/{grid_size} cells"
    )
}
