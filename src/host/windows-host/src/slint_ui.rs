use crate::{load_or_init_question_bank, question_bank_path, WindowsHostConfig};
use anyhow::Result;
use serde_json::Value;
use slint::SharedString;
use std::cell::Cell;
use std::rc::Rc;

slint::slint! {
    import { Button, LineEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";

    component MenuButton inherits Rectangle {
        in property <string> label;
        in property <bool> selected;
        callback clicked();

        width: 144px;
        height: 36px;
        border-radius: 6px;
        background: selected ? #b8c8cd : #e2e9ec;
        TouchArea {
            clicked => { root.clicked(); }
        }
        Text {
            text: root.label;
            color: #17252f;
            font-size: 14px;
            font-weight: selected ? 700 : 500;
            vertical-alignment: center;
            x: 14px;
            width: parent.width - 28px;
            height: parent.height;
        }
    }

    component InfoRow inherits HorizontalLayout {
        in property <string> name;
        in property <string> value;
        spacing: 14px;
        Text {
            text: root.name;
            width: 150px;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        Text {
            text: root.value;
            color: #17252f;
            font-size: 13px;
            wrap: word-wrap;
            vertical-alignment: center;
        }
    }

    component FormRow inherits HorizontalLayout {
        in property <string> name;
        in property <string> value;
        spacing: 14px;
        Text {
            text: root.name;
            width: 150px;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        LineEdit {
            text: root.value;
            width: 420px;
            height: 32px;
        }
    }

    component StaticRow inherits HorizontalLayout {
        in property <string> name;
        in property <string> value;
        spacing: 14px;
        Text {
            text: root.name;
            width: 150px;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        Rectangle {
            width: 420px;
            height: 32px;
            border-radius: 4px;
            border-width: 1px;
            border-color: #d7e1e6;
            background: #e8eef1;
            Text {
                x: 10px;
                width: parent.width - 20px;
                height: parent.height;
                text: root.value;
                color: #17252f;
                font-size: 13px;
                vertical-alignment: center;
                overflow: elide;
            }
        }
    }

    component LogPanel inherits Rectangle {
        in property <string> value;
        height: 300px;
        border-radius: 6px;
        border-width: 1px;
        border-color: #c9d5da;
        background: #f7fafb;
        Text {
            x: 12px;
            y: 12px;
            width: parent.width - 24px;
            height: parent.height - 24px;
            text: root.value;
            color: #17252f;
            font-size: 12px;
            wrap: word-wrap;
        }
    }

    export component HostDashboard inherits Window {
        property <int> active-page: 0;
        in property <string> product;
        in property <string> version;
        in property <string> mode;
        in property <string> identity-id;
        in property <string> device-id;
        in property <string> local-api;
        in property <string> question-bank;
        in property <string> data-dir;
        in property <string> storage-provider;
        in property <string> capabilities;
        in property <string> call-chain;
        in property <string> diagnostics;
        property <string> current-title: active-page == 0 ? "Status" : active-page == 1 ? "Local Core" : active-page == 2 ? "Data" : "Diagnostics";
        callback close-requested();

        title: "Yian Windows Host";
        preferred-width: 960px;
        preferred-height: 620px;
        background: #eef3f5;

        HorizontalBox {
            padding: 0px;
            spacing: 0px;

            Rectangle {
                min-width: 180px;
                max-width: 180px;
                background: #eef3f5;

                VerticalBox {
                    x: 18px;
                    y: 28px;
                    width: 144px;
                    height: 186px;
                    spacing: 14px;

                    MenuButton {
                        label: "Status";
                        selected: root.active-page == 0;
                        clicked => { root.active-page = 0; }
                    }
                    MenuButton {
                        label: "Local Core";
                        selected: root.active-page == 1;
                        clicked => { root.active-page = 1; }
                    }
                    MenuButton {
                        label: "Data";
                        selected: root.active-page == 2;
                        clicked => { root.active-page = 2; }
                    }
                    MenuButton {
                        label: "Diagnostics";
                        selected: root.active-page == 3;
                        clicked => { root.active-page = 3; }
                    }
                }
            }

            Rectangle {
                min-width: 640px;
                background: #eef3f5;
                VerticalBox {
                    padding: 24px;
                    spacing: 16px;

                    Rectangle {
                        height: 36px;
                        background: transparent;
                        Image {
                            x: 0px;
                            y: 4px;
                            source: @image-url("assets/lock.png");
                            width: 28px;
                            height: 28px;
                        }
                        Text {
                            x: 38px;
                            y: 4px;
                            width: 520px;
                            height: 28px;
                            text: "Yian: StoryLock - " + root.current-title;
                            font-size: 16px;
                            font-weight: 800;
                            color: #17252f;
                            overflow: elide;
                        }
                    }

                    Rectangle {
                        height: 1px;
                        background: #d7e1e6;
                    }

                    Rectangle {
                        width: 600px;
                        height: 420px;
                        background: transparent;

                        if root.active-page == 0: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 600px;
                            height: 420px;
                            spacing: 12px;
                            FormRow { name: "Identity"; value: root.identity-id; }
                            FormRow { name: "Device"; value: root.device-id; }
                            FormRow { name: "Local API"; value: root.local-api; }
                            StaticRow { name: "Mode"; value: root.mode; }
                        }

                        if root.active-page == 1: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 600px;
                            height: 420px;
                            spacing: 12px;
                            FormRow { name: "Capabilities"; value: root.capabilities; }
                            FormRow { name: "Call Chain"; value: root.call-chain; }
                            StaticRow { name: "Boundary"; value: "Windows DPAPI local only"; }
                            StaticRow { name: "Remote Access"; value: "Disabled by default"; }
                        }

                        if root.active-page == 2: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 600px;
                            height: 420px;
                            spacing: 12px;
                            FormRow { name: "Question Bank"; value: root.question-bank; }
                            FormRow { name: "Data Directory"; value: root.data-dir; }
                            StaticRow { name: "Storage"; value: root.storage-provider; }
                        }

                        if root.active-page == 3: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 600px;
                            height: 420px;
                            spacing: 12px;
                            LogPanel { value: root.diagnostics; }
                        }
                    }

                }
            }
        }
    }

    export component RequestConfirmation inherits Window {
        in property <string> request-id;
        in property <string> capability;
        in property <string> object-ref;
        in property <string> requester;
        in property <string> origin;
        in property <string> required-strength;
        in property <string> allowed-action;
        in property <string> expiry;
        in property <string> risk;
        callback approve-requested();
        callback deny-requested();

        title: "Yian Request Confirmation";
        preferred-width: 620px;
        preferred-height: 500px;
        background: #eef3f5;

        VerticalBox {
            padding: 22px;
            spacing: 14px;
            Text { text: "Confirm Local Request"; font-size: 22px; font-weight: 800; color: #17252f; }
            InfoRow { name: "Request"; value: root.request-id; }
            InfoRow { name: "Capability"; value: root.capability; }
            InfoRow { name: "Object"; value: root.object-ref; }
            InfoRow { name: "Requester"; value: root.requester; }
            InfoRow { name: "Origin"; value: root.origin; }
            InfoRow { name: "Strength"; value: root.required-strength; }
            InfoRow { name: "Action"; value: root.allowed-action; }
            InfoRow { name: "Expiry"; value: root.expiry; }
            Text { text: root.risk; color: #17252f; wrap: word-wrap; }
            HorizontalBox {
                alignment: end;
                spacing: 10px;
                Button { text: "Deny"; clicked => { root.deny-requested(); } }
                Button { text: "Approve"; clicked => { root.approve-requested(); } }
            }
        }
    }
}

pub fn run(config: WindowsHostConfig) -> Result<()> {
    let bank = load_or_init_question_bank(&config.data_dir)?;
    let app = HostDashboard::new()?;
    app.set_product(SharedString::from(config.product.clone()));
    app.set_version(SharedString::from(config.version.clone()));
    app.set_mode(SharedString::from(if config.remote_enabled {
        "Remote relay enabled"
    } else {
        "Local only"
    }));
    app.set_identity_id(SharedString::from(config.identity_id.clone()));
    app.set_device_id(SharedString::from(config.device_id.clone()));
    app.set_local_api(SharedString::from(config.health_url.clone()));
    app.set_question_bank(SharedString::from(format!(
        "{} ({} questions)",
        bank.question_set_version,
        bank.questions.len()
    )));
    app.set_storage_provider(SharedString::from("Windows DPAPI"));
    app.set_data_dir(SharedString::from(config.data_dir.display().to_string()));
    app.set_capabilities(SharedString::from(if config.remote_enabled {
        "health, verify, authorize, revoke, execute, relay_poll"
    } else {
        "health, verify, authorize, revoke, execute"
    }));
    app.set_call_chain(SharedString::from("verify -> authorize -> execute -> revoke"));
    app.set_diagnostics(SharedString::from(format!(
        "Sensitive values are hidden. Question answers, passwords, private keys, signing key bytes, and raw story text are never shown in this UI.\nQuestion bank path: {}",
        question_bank_path(&config.data_dir).display()
    )));
    let weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
    });
    app.run()?;
    Ok(())
}

fn summary_field(summary: &Value, name: &str) -> SharedString {
    SharedString::from(summary.get(name).and_then(Value::as_str).unwrap_or(""))
}

pub fn confirm_request(summary: &Value) -> Result<bool> {
    let app = RequestConfirmation::new()?;
    app.set_request_id(summary_field(summary, "requestId"));
    app.set_capability(summary_field(summary, "capability"));
    app.set_object_ref(summary_field(summary, "objectRef"));
    app.set_requester(summary_field(summary, "requester"));
    app.set_origin(summary_field(summary, "origin"));
    app.set_required_strength(summary_field(summary, "requiredStrength"));
    app.set_allowed_action(summary_field(summary, "allowedAction"));
    app.set_expiry(summary_field(summary, "expiry"));
    app.set_risk(summary_field(summary, "risk"));

    let approved = Rc::new(Cell::new(false));
    let weak = app.as_weak();
    let approve_result = Rc::clone(&approved);
    app.on_approve_requested(move || {
        approve_result.set(true);
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
    });
    let weak = app.as_weak();
    let deny_result = Rc::clone(&approved);
    app.on_deny_requested(move || {
        deny_result.set(false);
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
    });

    app.run()?;
    Ok(approved.get())
}
