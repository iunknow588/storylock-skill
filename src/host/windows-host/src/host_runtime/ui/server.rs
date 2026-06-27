use super::*;

fn read_request_payload(request: &mut tiny_http::Request) -> Value {
    let mut body = String::new();
    match request.as_reader().read_to_string(&mut body) {
        Ok(_) => serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({})),
        Err(_) => json!({}),
    }
}

pub(crate) fn start_local_server(runtime: WindowsHostRuntime) -> Result<thread::JoinHandle<()>> {
    let address = format!("127.0.0.1:{}", runtime.config.host_port);
    let server = Server::http(&address)
        .map_err(|error| anyhow!("failed to bind local server on {address}: {error}"))?;
    println!("local server listening on http://{address}");

    Ok(thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let path = request.url().split('?').next().unwrap_or("/");
            let response = match (request.method(), path) {
                (&Method::Get, "/health") => {
                    Response::from_string(runtime.config.health_json().to_string())
                        .with_header(content_type_json())
                }
                (&Method::Get, "/question-bank/status") => Response::from_string(
                    question_bank_status(
                        &runtime,
                        &json!({
                            "requestId": format!("req-{}", Uuid::new_v4())
                        }),
                    )
                    .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Get, "/ui") => Response::from_string(windows_host_management_ui_html())
                    .with_header(content_type_html()),
                (&Method::Get, "/ui/status") => {
                    Response::from_string(ui_status(&runtime).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Get, "/diagnostics") => {
                    Response::from_string(diagnostics_status(&runtime).to_string())
                        .with_header(content_type_json())
                }
                (&Method::Get, "/story-template/candidates") => Response::from_string(
                    story_template_candidates(
                        &runtime,
                        &json!({
                            "requestId": format!("req-{}", Uuid::new_v4())
                        }),
                    )
                    .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/shutdown") => {
                    thread::spawn(|| {
                        thread::sleep(Duration::from_millis(150));
                        std::process::exit(0);
                    });
                    Response::from_string(
                        json!({
                            "status": "success",
                            "capability": "windowsHostShutdown",
                            "message": "shutdown scheduled"
                        })
                        .to_string(),
                    )
                    .with_header(content_type_json())
                }
                (&Method::Post, "/verify") => Response::from_string(
                    create_grid_verification(&runtime, &read_request_payload(&mut request))
                        .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/authorize") => Response::from_string(
                    authorize_local_action(&runtime, &read_request_payload(&mut request))
                        .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/revoke") => Response::from_string(
                    revoke_local_authorization(&runtime, &read_request_payload(&mut request))
                        .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/question-bank/import") => Response::from_string(
                    question_bank_import(&runtime, &read_request_payload(&mut request)).to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/story-template/generate") => Response::from_string(
                    story_template_generate(&runtime, &read_request_payload(&mut request))
                        .to_string(),
                )
                .with_header(content_type_json()),
                (&Method::Post, "/execute") => {
                    let execution = execute_request(&runtime, read_request_payload(&mut request));
                    runtime.record_execution_summary(&execution);
                    Response::from_string(execution.to_string()).with_header(content_type_json())
                }
                _ => Response::from_string("{\"status\":\"error\",\"message\":\"not found\"}")
                    .with_status_code(StatusCode(404))
                    .with_header(content_type_json()),
            };
            let _ = request.respond(response);
        }
    }))
}
