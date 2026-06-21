use crate::{load_or_init_question_bank, question_bank_path, WindowsHostConfig};
use anyhow::Result;
use serde_json::json;
use serde_json::Value;
use slint::SharedString;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

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

    component EditableRow inherits HorizontalLayout {
        in property <string> name;
        in-out property <string> value;
        spacing: 14px;
        Text {
            text: root.name;
            width: 150px;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        LineEdit {
            text <=> root.value;
            width: 420px;
            height: 32px;
        }
    }

    component ActionButton inherits Rectangle {
        in property <string> label;
        in property <bool> primary;
        callback clicked();

        width: 150px;
        height: 34px;
        border-radius: 6px;
        background: primary ? #45606b : #d7e1e6;
        TouchArea {
            clicked => { root.clicked(); }
        }
        Text {
            text: root.label;
            color: primary ? white : #17252f;
            font-size: 13px;
            font-weight: 700;
            horizontal-alignment: center;
            vertical-alignment: center;
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
        in property <length> panel-height: 300px;
        height: root.panel-height;
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
        in property <string> managed-objects;
        property <string> core-launch-status: "StoryLock Core is closed. The host can only view redacted permission metadata.";
        property <string> current-title: active-page == 0 ? "Status" : active-page == 1 ? "Local Core" : active-page == 2 ? "Data" : active-page == 3 ? "StoryLock Core" : "Diagnostics";
        callback close-requested();
        callback open-storylock-core();

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
                    height: 236px;
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
                        label: "StoryLock Core";
                        selected: root.active-page == 3;
                        clicked => { root.active-page = 3; }
                    }
                    MenuButton {
                        label: "Diagnostics";
                        selected: root.active-page == 4;
                        clicked => { root.active-page = 4; }
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
                            StaticRow { name: "Boundary"; value: "Host is permission-view only"; }
                            StaticRow { name: "Editable Config"; value: "StoryLock Core only"; }
                            StaticRow { name: "Private Data"; value: "Not readable from host window"; }
                            HorizontalBox {
                                spacing: 14px;
                                Text {
                                    text: "Application";
                                    width: 150px;
                                    color: #62727d;
                                    font-size: 13px;
                                    vertical-alignment: center;
                                }
                                ActionButton {
                                    label: "Open StoryLock Core";
                                    primary: true;
                                    clicked => {
                                        root.core-launch-status = "StoryLock Core opened in a separate local window. Host remains read-only.";
                                        root.open-storylock-core();
                                    }
                                }
                            }
                            LogPanel { value: "Managed object permission metadata visible to host:\n" + root.managed-objects; panel-height: 180px; }
                            LogPanel { value: root.core-launch-status; panel-height: 86px; }
                        }

                        if root.active-page == 4: VerticalBox {
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

    export component StoryLockCoreApp inherits Window {
        property <int> active-page: 0;
        in-out property <string> story-title: "梅雨傍晚的旧火车站录音卡";
        in-out property <string> story-summary: "梅雨季的周三傍晚，林澈在南城旧火车站二号候车厅取到藏有录音卡的蓝色保温杯，并在发车铃前把它交给记者周苓。";
        in-out property <string> memory-anchors: "梅雨季 / 周三傍晚 / 17号寄存柜 / 蓝色保温杯 / 发车铃前二十分钟";
        in-out property <string> element-group: "时间,地点,人物,外部,起步,核心,过程,收束";
        in-out property <int> node-index: 0;
        in-out property <string> node-position: "1 / 24";
        in-out property <string> node-id: "node-01";
        in-out property <string> node-title: "日期";
        in-out property <string> element-id: "time";
        in-out property <string> question-text: "故事发生在什么季节和星期几？";
        in-out property <string> canonical-answer: "梅雨季的一个周三傍晚。";
        in-out property <string> accepted-answers: "梅雨季周三; 周三傍晚; 梅雨季";
        in-out property <string> correct-options: "梅雨季; 周三傍晚; 发车铃前二十分钟";
        in-out property <string> distractors: "初雪清晨; 周日午夜; 海边码头; 红色背包; 3号寄存柜; 午夜电台";
        in-out property <string> selection-mode: "multi_select";
        in-out property <string> correct-count: "3";
        in-out property <string> candidate-pool-size: "9";
        in-out property <string> recall-priority: "high";
        in-out property <string> verify-policy: "caseInsensitive + trim";
        in-out property <string> editor-notes: "作者稿可编辑；发布态不得暴露正确项数量、答案或训练备注。";
        in-out property <string> node-output: "编辑 24 节点作者稿后，可生成本地运行时题目预览。";
        in-out property <string> vault-name: "storylock-local-vault";
        in-out property <string> resource-id: "github-main";
        in-out property <string> resource-kind: "website_account";
        in-out property <string> provider-id: "github";
        in-out property <string> display-name: "GitHub 主账号";
        in-out property <string> resource-bindings: "username -> credential/github/main/username\npassword -> credential/github/main/password\ntotp_secret -> credential/github/main/totp_secret";
        in-out property <string> object-meta: "username: reference utf8\npassword: secret utf8\ntotp_secret: secret utf8";
        in-out property <string> template-kind: "login-sites.json";
        in-out property <string> template-id: "github.com";
        in-out property <string> template-display-name: "GitHub 主账号登录";
        in-out property <string> template-bindings: "login-sites.json\n  username -> username\n  password -> password\n\nsigning-actions.json\n  username -> username\n\nagent-tasks.json\n  username -> username";
        in-out property <string> export-preview: "identity-package/\n  vault.stlk\n  resource-catalog.json\n  package-manifest.json\n  templates/login-sites.json\n  templates/signing-actions.json\n  templates/agent-tasks.json";
        in-out property <string> config-status: "All edits stay inside StoryLock Core. Host receives only derived permission metadata.";
        in-out property <string> core-data-dir: "";
        property <string> current-title: active-page == 0 ? "Story" : active-page == 1 ? "24 Nodes" : active-page == 2 ? "Resources" : active-page == 3 ? "Templates" : "Export";
        callback save-story();
        callback save-node();
        callback previous-node();
        callback next-node();
        callback save-resource();
        callback save-template();
        callback refresh-export();

        title: "StoryLock Core";
        preferred-width: 920px;
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
                    height: 236px;
                    spacing: 14px;
                    MenuButton {
                        label: "Story";
                        selected: root.active-page == 0;
                        clicked => { root.active-page = 0; }
                    }
                    MenuButton {
                        label: "24 Nodes";
                        selected: root.active-page == 1;
                        clicked => { root.active-page = 1; }
                    }
                    MenuButton {
                        label: "Resources";
                        selected: root.active-page == 2;
                        clicked => { root.active-page = 2; }
                    }
                    MenuButton {
                        label: "Templates";
                        selected: root.active-page == 3;
                        clicked => { root.active-page = 3; }
                    }
                    MenuButton {
                        label: "Export";
                        selected: root.active-page == 4;
                        clicked => { root.active-page = 4; }
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
                            width: 560px;
                            height: 28px;
                            text: "StoryLock Core - " + root.current-title;
                            font-size: 16px;
                            font-weight: 800;
                            color: #17252f;
                            overflow: elide;
                        }
                    }

                    Rectangle { height: 1px; background: #d7e1e6; }

                    Rectangle {
                        width: 620px;
                        height: 430px;
                        background: transparent;

                        if root.active-page == 0: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 620px;
                            height: 430px;
                            spacing: 12px;
                            StaticRow { name: "Workbench"; value: "8 elements + 24 nodes author draft"; }
                            StaticRow { name: "Data Dir"; value: root.core-data-dir; }
                            EditableRow { name: "Story Title"; value <=> root.story-title; }
                            EditableRow { name: "Summary"; value <=> root.story-summary; }
                            EditableRow { name: "Memory Anchors"; value <=> root.memory-anchors; }
                            EditableRow { name: "Element Groups"; value <=> root.element-group; }
                            StaticRow { name: "Fixed Nodes"; value: "时间/地点/人物/外部/起步/核心/过程/收束 x 3 = 24"; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Save";
                                    primary: true;
                                    clicked => {
                                        root.save-story();
                                    }
                                }
                                ActionButton {
                                    label: "Refresh";
                                    primary: false;
                                    clicked => {
                                        root.refresh-export();
                                    }
                                }
                            }
                            LogPanel { value: root.config-status; panel-height: 102px; }
                        }

                        if root.active-page == 1: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 620px;
                            height: 430px;
                            spacing: 12px;
                            StaticRow { name: "Node No"; value: root.node-position; }
                            EditableRow { name: "Node ID"; value <=> root.node-id; }
                            EditableRow { name: "Title"; value <=> root.node-title; }
                            EditableRow { name: "Element ID"; value <=> root.element-id; }
                            EditableRow { name: "Question"; value <=> root.question-text; }
                            EditableRow { name: "Canonical Answer"; value <=> root.canonical-answer; }
                            EditableRow { name: "Accepted Answers"; value <=> root.accepted-answers; }
                            EditableRow { name: "Correct Options"; value <=> root.correct-options; }
                            EditableRow { name: "Distractors"; value <=> root.distractors; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Prev";
                                    primary: false;
                                    clicked => {
                                        root.previous-node();
                                    }
                                }
                                ActionButton {
                                    label: "Save";
                                    primary: true;
                                    clicked => {
                                        root.save-node();
                                    }
                                }
                                ActionButton {
                                    label: "Next";
                                    primary: false;
                                    clicked => {
                                        root.next-node();
                                    }
                                }
                            }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Build Preview";
                                    primary: false;
                                    clicked => {
                                        root.node-output = "runtime question preview\nnodeId=" + root.node-id
                                            + "\ntitle=" + root.node-title
                                            + "\nelementId=" + root.element-id
                                            + "\nselectionMode=" + root.selection-mode
                                            + "\ncorrectCount=" + root.correct-count
                                            + "\ncandidatePoolSize=" + root.candidate-pool-size
                                            + "\nrecallPriority=" + root.recall-priority
                                            + "\nverifyPolicy=" + root.verify-policy
                                            + "\n\nAnswers and editor notes are local-core only.";
                                    }
                                }
                            }
                            LogPanel { value: root.node-output; panel-height: 54px; }
                        }

                        if root.active-page == 2: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 620px;
                            height: 430px;
                            spacing: 12px;
                            EditableRow { name: "Vault"; value <=> root.vault-name; }
                            EditableRow { name: "Resource ID"; value <=> root.resource-id; }
                            EditableRow { name: "Kind"; value <=> root.resource-kind; }
                            EditableRow { name: "Provider"; value <=> root.provider-id; }
                            EditableRow { name: "Display Name"; value <=> root.display-name; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Save";
                                    primary: true;
                                    clicked => {
                                        root.save-resource();
                                    }
                                }
                                ActionButton {
                                    label: "Refresh";
                                    primary: false;
                                    clicked => {
                                        root.refresh-export();
                                    }
                                }
                            }
                            LogPanel { value: "bindings:\n" + root.resource-bindings + "\n\nobjectMeta:\n" + root.object-meta; panel-height: 170px; }
                        }

                        if root.active-page == 3: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 620px;
                            height: 430px;
                            spacing: 12px;
                            EditableRow { name: "Template File"; value <=> root.template-kind; }
                            EditableRow { name: "Template ID"; value <=> root.template-id; }
                            EditableRow { name: "Display Name"; value <=> root.template-display-name; }
                            EditableRow { name: "Resource ID"; value <=> root.resource-id; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Save";
                                    primary: true;
                                    clicked => {
                                        root.save-template();
                                    }
                                }
                                ActionButton {
                                    label: "Refresh";
                                    primary: false;
                                    clicked => {
                                        root.refresh-export();
                                    }
                                }
                            }
                            LogPanel { value: "template bundles:\n" + root.template-bindings + "\n\nRules: templates describe actions only; they reference resourceId + role and never store secret values."; panel-height: 220px; }
                        }

                        if root.active-page == 4: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 620px;
                            height: 430px;
                            spacing: 12px;
                            StaticRow { name: "Package Kind"; value: "storylock_identity_package"; }
                            StaticRow { name: "Vault File"; value: "vault.stlk"; }
                            StaticRow { name: "Catalog"; value: "resource-catalog.json"; }
                            StaticRow { name: "Manifest"; value: "package-manifest.json"; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Refresh Preview";
                                    primary: true;
                                    clicked => {
                                        root.refresh-export();
                                    }
                                }
                            }
                            LogPanel { value: root.export-preview + "\n\nExport boundary: vault secrets stay encrypted; templates and resource catalog expose structure but no secret values."; panel-height: 220px; }
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
    app.set_call_chain(SharedString::from(
        "verify -> authorize -> execute -> revoke",
    ));
    let core_package_dir = config.data_dir.join("storylock-core");
    ensure_storylock_core_package(&core_package_dir)?;
    app.set_managed_objects(SharedString::from(build_permission_summary_text(
        &core_package_dir,
    )));
    app.set_diagnostics(SharedString::from(format!(
        "Sensitive values are hidden. Question answers, passwords, private keys, signing key bytes, and raw story text are never shown in this UI.\nQuestion bank path: {}\nStoryLock Core package path: {}",
        question_bank_path(&config.data_dir).display(),
        core_package_dir.display()
    )));
    let core_windows: Rc<RefCell<Vec<StoryLockCoreApp>>> = Rc::new(RefCell::new(Vec::new()));
    let core_windows_for_callback = Rc::clone(&core_windows);
    let host_weak_for_core = app.as_weak();
    app.on_open_storylock_core(move || {
        if let Err(error) = ensure_storylock_core_package(&core_package_dir) {
            eprintln!("failed to initialize StoryLock Core package: {error}");
        }
        match StoryLockCoreApp::new() {
            Ok(core) => {
                initialize_storylock_core_window(&core, &core_package_dir);
                wire_storylock_core_callbacks(
                    &core,
                    core_package_dir.clone(),
                    host_weak_for_core.clone(),
                );
                if let Err(error) = core.show() {
                    eprintln!("failed to show StoryLock Core window: {error}");
                    return;
                }
                core_windows_for_callback.borrow_mut().push(core);
            }
            Err(error) => {
                eprintln!("failed to create StoryLock Core window: {error}");
            }
        }
    });
    let weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
    });
    app.run()?;
    Ok(())
}

fn storylock_core_manifest_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("package-manifest.json")
}

fn storylock_core_catalog_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("resource-catalog.json")
}

fn storylock_core_author_draft_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("author-draft.json")
}

fn storylock_core_template_path(package_dir: &Path, file_name: &str) -> std::path::PathBuf {
    package_dir.join("templates").join(file_name)
}

fn ensure_storylock_core_package(package_dir: &Path) -> Result<()> {
    fs::create_dir_all(package_dir.join("templates"))?;
    write_json_if_missing(
        &storylock_core_manifest_path(package_dir),
        &json!({
            "packageId": "windows-storylock-core-local",
            "version": "0.1.0",
            "createdAt": ui_now_timestamp(),
            "description": "Local Windows StoryLock Core package.",
            "files": [
                "package-manifest.json",
                "resource-catalog.json",
                "author-draft.json",
                "templates/login-sites.json",
                "templates/signing-actions.json",
                "templates/agent-tasks.json"
            ]
        }),
    )?;
    write_json_if_missing(
        &storylock_core_author_draft_path(package_dir),
        &default_author_draft_json(),
    )?;
    write_json_if_missing(
        &storylock_core_catalog_path(package_dir),
        &default_resource_catalog_json(),
    )?;
    write_json_if_missing(
        &storylock_core_template_path(package_dir, "login-sites.json"),
        &default_login_templates_json(),
    )?;
    write_json_if_missing(
        &storylock_core_template_path(package_dir, "signing-actions.json"),
        &default_signing_templates_json(),
    )?;
    write_json_if_missing(
        &storylock_core_template_path(package_dir, "agent-tasks.json"),
        &default_agent_templates_json(),
    )?;
    Ok(())
}

fn write_json_if_missing(path: &Path, value: &Value) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
    }
    Ok(())
}

fn ui_now_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
        .to_string()
}

fn read_json_or_default(path: &Path, fallback: Value) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .unwrap_or(fallback)
}

fn initialize_storylock_core_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let draft = read_json_or_default(
        &storylock_core_author_draft_path(package_dir),
        default_author_draft_json(),
    );
    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let template = read_json_or_default(
        &storylock_core_template_path(package_dir, "login-sites.json"),
        default_login_templates_json(),
    );
    core.set_core_data_dir(SharedString::from(package_dir.display().to_string()));
    core.set_story_title(json_string(&draft, &["storyTitle"]));
    core.set_story_summary(json_string(&draft, &["summary"]));
    core.set_memory_anchors(SharedString::from(
        draft
            .get("memoryAnchors")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .unwrap_or_default(),
    ));
    core.set_element_group(SharedString::from(
        draft
            .get("elementGroups")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default(),
    ));
    load_node_into_window(core, package_dir, 0);
    if let Some(resource) = catalog
        .get("resources")
        .and_then(Value::as_array)
        .and_then(|resources| resources.first())
    {
        core.set_resource_id(json_string(resource, &["resourceId"]));
        core.set_resource_kind(json_string(resource, &["resourceKind"]));
        core.set_provider_id(json_string(resource, &["providerId"]));
        core.set_display_name(json_string(resource, &["displayName"]));
        core.set_resource_bindings(SharedString::from(format_bindings(resource)));
        core.set_object_meta(SharedString::from(format_object_meta(resource)));
    }
    if let Some(item) = template
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
    {
        core.set_template_id(json_string(item, &["templateId"]));
        core.set_template_display_name(json_string(item, &["displayName"]));
        core.set_template_bindings(SharedString::from(format_all_template_bundles(package_dir)));
    }
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
}

fn wire_storylock_core_callbacks(
    core: &StoryLockCoreApp,
    package_dir: std::path::PathBuf,
    host_weak: slint::Weak<HostDashboard>,
) {
    let weak = core.as_weak();
    let story_dir = package_dir.clone();
    core.on_save_story(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_story_from_window(&core, &story_dir);
            set_core_status(&core, result, "Story author draft saved locally.");
        }
    });

    let weak = core.as_weak();
    let node_dir = package_dir.clone();
    core.on_save_node(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_current_node_from_window(&core, &node_dir);
            set_core_status(&core, result, "Story node saved locally.");
        }
    });

    let weak = core.as_weak();
    let previous_node_dir = package_dir.clone();
    core.on_previous_node(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_current_node_from_window(&core, &previous_node_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            let next_index = core.get_node_index().saturating_sub(1);
            load_node_into_window(&core, &previous_node_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_node_dir = package_dir.clone();
    core.on_next_node(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_current_node_from_window(&core, &next_node_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            let next_index = (core.get_node_index() + 1).min(23);
            load_node_into_window(&core, &next_node_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let resource_dir = package_dir.clone();
    let resource_host_weak = host_weak.clone();
    core.on_save_resource(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_resource_from_window(&core, &resource_dir);
            set_core_status(&core, result, "Resource catalog saved locally.");
            refresh_host_permission_summary(&resource_host_weak, &resource_dir);
        }
    });

    let weak = core.as_weak();
    let template_dir = package_dir.clone();
    core.on_save_template(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_template_from_window(&core, &template_dir);
            set_core_status(&core, result, "Login template saved locally.");
        }
    });

    let weak = core.as_weak();
    let refresh_host_weak = host_weak.clone();
    core.on_refresh_export(move || {
        if let Some(core) = weak.upgrade() {
            core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
            core.set_config_status(SharedString::from(
                "Export preview refreshed from local StoryLock Core package.",
            ));
            refresh_host_permission_summary(&refresh_host_weak, &package_dir);
        }
    });
}

fn refresh_host_permission_summary(host_weak: &slint::Weak<HostDashboard>, package_dir: &Path) {
    if let Some(host) = host_weak.upgrade() {
        host.set_managed_objects(SharedString::from(build_permission_summary_text(
            package_dir,
        )));
    }
}

fn set_core_status(core: &StoryLockCoreApp, result: Result<()>, success_message: &str) {
    match result {
        Ok(()) => {
            core.set_config_status(SharedString::from(success_message));
            core.set_export_preview(SharedString::from(build_export_preview(Path::new(
                core.get_core_data_dir().as_str(),
            ))));
        }
        Err(error) => core.set_config_status(SharedString::from(format!("Save failed: {error}"))),
    }
}

fn save_story_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let path = storylock_core_author_draft_path(package_dir);
    let mut draft = read_json_or_default(&path, default_author_draft_json());
    draft["storyTitle"] = json!(core.get_story_title().to_string());
    draft["summary"] = json!(core.get_story_summary().to_string());
    draft["memoryAnchors"] = json!(split_list(core.get_memory_anchors().as_str(), "/"));
    draft["elementGroups"] = json!(split_list(core.get_element_group().as_str(), ","));
    write_current_node_to_draft(core, &mut draft);
    fs::write(path, serde_json::to_vec_pretty(&draft)?)?;
    Ok(())
}

fn save_current_node_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let path = storylock_core_author_draft_path(package_dir);
    let mut draft = read_json_or_default(&path, default_author_draft_json());
    write_current_node_to_draft(core, &mut draft);
    fs::write(path, serde_json::to_vec_pretty(&draft)?)?;
    core.set_node_output(SharedString::from(format!(
        "saved node {}\nnodeId={}\ntitle={}\nelementId={}\nquestion={}\n\nAnswers and editor notes are local-core only.",
        core.get_node_position(),
        core.get_node_id(),
        core.get_node_title(),
        core.get_element_id(),
        core.get_question_text()
    )));
    Ok(())
}

fn write_current_node_to_draft(core: &StoryLockCoreApp, draft: &mut Value) {
    let node_index = normalize_node_index(core.get_node_index());
    ensure_draft_nodes(draft);
    if let Some(node) = draft
        .get_mut("nodes")
        .and_then(Value::as_array_mut)
        .and_then(|nodes| nodes.get_mut(node_index))
    {
        node["nodeId"] = json!(core.get_node_id().to_string());
        node["title"] = json!(core.get_node_title().to_string());
        node["elementId"] = json!(core.get_element_id().to_string());
        node["question"] = json!(core.get_question_text().to_string());
        node["recommendedSelectionMode"] = json!(core.get_selection_mode().to_string());
        node["recommendedCorrectCount"] =
            json!(core.get_correct_count().parse::<u32>().unwrap_or(3));
        node["candidatePoolSize"] =
            json!(core.get_candidate_pool_size().parse::<u32>().unwrap_or(9));
        node["recallPriority"] = json!(core.get_recall_priority().to_string());
        node["verifyPolicy"] = json!(core.get_verify_policy().to_string());
        node["editorNotes"] = json!(core.get_editor_notes().to_string());
        node["canonicalAnswerLocalOnly"] = json!(core.get_canonical_answer().to_string());
        node["acceptedAnswersLocalOnly"] =
            json!(split_list(core.get_accepted_answers().as_str(), ";"));
        node["correctOptionsLocalOnly"] =
            json!(split_list(core.get_correct_options().as_str(), ";"));
        node["distractorsLocalOnly"] = json!(split_list(core.get_distractors().as_str(), ";"));
    }
}

fn load_node_into_window(core: &StoryLockCoreApp, package_dir: &Path, requested_index: i32) {
    let node_index = normalize_node_index(requested_index);
    let mut draft = read_json_or_default(
        &storylock_core_author_draft_path(package_dir),
        default_author_draft_json(),
    );
    ensure_draft_nodes(&mut draft);
    let node = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.get(node_index))
        .cloned()
        .unwrap_or_else(|| default_author_draft_json()["nodes"][node_index].clone());
    core.set_node_index(node_index as i32);
    core.set_node_position(SharedString::from(format!("{} / 24", node_index + 1)));
    core.set_node_id(json_string(&node, &["nodeId"]));
    core.set_node_title(json_string(&node, &["title"]));
    core.set_element_id(json_string(&node, &["elementId"]));
    core.set_question_text(json_string(&node, &["question"]));
    core.set_selection_mode(json_string(&node, &["recommendedSelectionMode"]));
    core.set_correct_count(SharedString::from(
        node.get("recommendedCorrectCount")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "3".to_string()),
    ));
    core.set_candidate_pool_size(SharedString::from(
        node.get("candidatePoolSize")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "9".to_string()),
    ));
    core.set_recall_priority(json_string(&node, &["recallPriority"]));
    core.set_verify_policy(json_string(&node, &["verifyPolicy"]));
    core.set_editor_notes(json_string(&node, &["editorNotes"]));
    core.set_canonical_answer(json_string(&node, &["canonicalAnswerLocalOnly"]));
    core.set_accepted_answers(SharedString::from(join_json_string_array(
        node.get("acceptedAnswersLocalOnly"),
        "; ",
    )));
    core.set_correct_options(SharedString::from(join_json_string_array(
        node.get("correctOptionsLocalOnly"),
        "; ",
    )));
    core.set_distractors(SharedString::from(join_json_string_array(
        node.get("distractorsLocalOnly"),
        "; ",
    )));
    core.set_node_output(SharedString::from(format!(
        "loaded node {}\nnodeId={}\ntitle={}\n\nUse Save before closing. Answers and editor notes are local-core only.",
        node_index + 1,
        core.get_node_id(),
        core.get_node_title()
    )));
}

fn normalize_node_index(index: i32) -> usize {
    index.clamp(0, 23) as usize
}

fn ensure_draft_nodes(draft: &mut Value) {
    let needs_reset = draft
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| nodes.len() != 24)
        .unwrap_or(true);
    if needs_reset {
        draft["nodes"] = default_author_draft_json()["nodes"].clone();
    }
}

fn join_json_string_array(value: Option<&Value>, delimiter: &str) -> String {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(delimiter)
        })
        .unwrap_or_default()
}

fn save_resource_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let catalog = json!({
        "version": "1",
        "resources": [{
            "resourceId": core.get_resource_id().to_string(),
            "resourceKind": core.get_resource_kind().to_string(),
            "providerId": core.get_provider_id().to_string(),
            "displayName": core.get_display_name().to_string(),
            "bindings": [
                {
                    "role": "username",
                    "objectId": format!("credential/{}/main/username", sanitize_segment(core.get_provider_id().as_str())),
                    "objectMeta": { "objectKind": "username", "encoding": "text", "sensitivity": "private" }
                },
                {
                    "role": "password",
                    "objectId": format!("credential/{}/main/password", sanitize_segment(core.get_provider_id().as_str())),
                    "objectMeta": { "objectKind": "password", "encoding": "secret", "sensitivity": "secret" }
                }
            ]
        }]
    });
    fs::write(
        storylock_core_catalog_path(package_dir),
        serde_json::to_vec_pretty(&catalog)?,
    )?;
    core.set_resource_bindings(SharedString::from(format_bindings(
        catalog
            .get("resources")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .unwrap_or(&Value::Null),
    )));
    core.set_object_meta(SharedString::from(format_object_meta(
        catalog
            .get("resources")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .unwrap_or(&Value::Null),
    )));
    Ok(())
}

fn save_template_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let resource_id = core.get_resource_id().to_string();
    let template_id = core.get_template_id().to_string();
    let display_name = core.get_template_display_name().to_string();
    let login_template = json!({
        "version": "1",
        "templateType": "login-sites",
        "items": [{
            "templateId": template_id,
            "displayName": display_name,
            "resourceId": resource_id,
            "bindings": [
                { "fieldName": "username", "role": "username" },
                { "fieldName": "password", "role": "password" }
            ]
        }]
    });
    let signing_template = json!({
        "version": "1",
        "templateType": "signing-actions",
        "items": [{
            "templateId": format!("{}-sign", core.get_template_id()),
            "displayName": format!("{} 签名", core.get_template_display_name()),
            "resourceId": core.get_resource_id().to_string(),
            "bindings": [
                { "fieldName": "username", "role": "username" }
            ]
        }]
    });
    let agent_template = json!({
        "version": "1",
        "templateType": "agent-tasks",
        "items": [{
            "templateId": format!("{}-agent", core.get_template_id()),
            "displayName": format!("{} Agent 任务", core.get_template_display_name()),
            "resourceId": core.get_resource_id().to_string(),
            "bindings": [
                { "fieldName": "username", "role": "username" }
            ]
        }]
    });
    fs::write(
        storylock_core_template_path(package_dir, "login-sites.json"),
        serde_json::to_vec_pretty(&login_template)?,
    )?;
    fs::write(
        storylock_core_template_path(package_dir, "signing-actions.json"),
        serde_json::to_vec_pretty(&signing_template)?,
    )?;
    fs::write(
        storylock_core_template_path(package_dir, "agent-tasks.json"),
        serde_json::to_vec_pretty(&agent_template)?,
    )?;
    core.set_template_bindings(SharedString::from(format_all_template_bundles(package_dir)));
    Ok(())
}

fn build_export_preview(package_dir: &Path) -> String {
    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let permission_objects = catalog
        .get("resources")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|resource| {
                    resource
                        .get("bindings")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0)
                })
                .sum::<usize>()
        })
        .unwrap_or(0);
    let preflight = preflight_storylock_core_package(package_dir);
    let status = if preflight.errors.is_empty() {
        "OK"
    } else {
        "FAILED"
    };
    let errors = if preflight.errors.is_empty() {
        "none".to_string()
    } else {
        preflight
            .errors
            .iter()
            .map(|issue| format!("{} {} {}", issue.code, issue.path, issue.message))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "identity-package/\n  vault.stlk\n  package-manifest.json\n  resource-catalog.json\n  author-draft.json\n  templates/login-sites.json\n  templates/signing-actions.json\n  templates/agent-tasks.json\n\nLocal path: {}\nresources={resources}\npermissionObjects={permission_objects}\npreflight={status}\nerrors:\n{errors}\n\nHost-readable permission summary only; raw story, answers, passwords, private keys, and signingKeyBytes remain inside StoryLock Core.",
        package_dir.display()
    )
}

#[derive(Clone, Debug)]
struct PreflightIssue {
    code: &'static str,
    path: String,
    message: String,
}

#[derive(Clone, Debug)]
struct PreflightResult {
    errors: Vec<PreflightIssue>,
}

fn preflight_storylock_core_package(package_dir: &Path) -> PreflightResult {
    let mut errors = Vec::new();
    for required_file in [
        "package-manifest.json",
        "resource-catalog.json",
        "author-draft.json",
        "templates/login-sites.json",
        "templates/signing-actions.json",
        "templates/agent-tasks.json",
    ] {
        if !package_dir.join(required_file).exists() {
            errors.push(PreflightIssue {
                code: "SL_PKG_OPTIONAL_FILE_MISSING",
                path: "$.files".to_string(),
                message: format!("missing required file: {required_file}"),
            });
        }
    }

    let manifest = read_json_or_default(&storylock_core_manifest_path(package_dir), Value::Null);
    if let Some(files) = manifest.get("files").and_then(Value::as_array) {
        for required_file in [
            "package-manifest.json",
            "resource-catalog.json",
            "author-draft.json",
            "templates/login-sites.json",
            "templates/signing-actions.json",
            "templates/agent-tasks.json",
        ] {
            if !files
                .iter()
                .any(|item| item.as_str() == Some(required_file))
            {
                errors.push(PreflightIssue {
                    code: "SL_PKG_OPTIONAL_FILE_MISSING",
                    path: "$.files".to_string(),
                    message: format!("manifest does not list required file: {required_file}"),
                });
            }
        }
    } else {
        errors.push(PreflightIssue {
            code: "SL_MANIFEST_MISSING_CATALOG_FILE",
            path: "$.files".to_string(),
            message: "manifest files must be an array".to_string(),
        });
    }

    let draft = read_json_or_default(
        &storylock_core_author_draft_path(package_dir),
        default_author_draft_json(),
    );
    match draft.get("nodes").and_then(Value::as_array) {
        Some(nodes) if nodes.len() == 24 => {}
        Some(nodes) => errors.push(PreflightIssue {
            code: "SL_PKG_AUTHOR_DRAFT_NODE_COUNT",
            path: "$.nodes".to_string(),
            message: format!(
                "author draft must contain exactly 24 nodes, got {}",
                nodes.len()
            ),
        }),
        None => errors.push(PreflightIssue {
            code: "SL_PKG_AUTHOR_DRAFT_NODE_COUNT",
            path: "$.nodes".to_string(),
            message: "author draft nodes must be an array".to_string(),
        }),
    }

    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let role_index = build_catalog_role_index(&catalog, &mut errors);
    for (file_name, fallback) in [
        ("login-sites.json", default_login_templates_json()),
        ("signing-actions.json", default_signing_templates_json()),
        ("agent-tasks.json", default_agent_templates_json()),
    ] {
        let bundle = read_json_or_default(
            &storylock_core_template_path(package_dir, file_name),
            fallback,
        );
        validate_template_references(file_name, &bundle, &role_index, &mut errors);
    }

    PreflightResult { errors }
}

fn build_catalog_role_index(
    catalog: &Value,
    errors: &mut Vec<PreflightIssue>,
) -> HashMap<String, HashSet<String>> {
    let mut role_index = HashMap::new();
    let Some(resources) = catalog.get("resources").and_then(Value::as_array) else {
        errors.push(PreflightIssue {
            code: "SL_CATALOG_MISSING_RESOURCES",
            path: "$.resources".to_string(),
            message: "resource catalog resources must be an array".to_string(),
        });
        return role_index;
    };
    for (resource_index, resource) in resources.iter().enumerate() {
        let resource_id = resource
            .get("resourceId")
            .and_then(Value::as_str)
            .unwrap_or("");
        if resource_id.is_empty() {
            errors.push(PreflightIssue {
                code: "SL_RESOURCE_MISSING_RESOURCE_ID",
                path: format!("$.resources[{resource_index}].resourceId"),
                message: "resourceId must be a non-empty string".to_string(),
            });
            continue;
        }
        let mut roles = HashSet::new();
        if let Some(bindings) = resource.get("bindings").and_then(Value::as_array) {
            for (binding_index, binding) in bindings.iter().enumerate() {
                let role = binding.get("role").and_then(Value::as_str).unwrap_or("");
                if role.is_empty() {
                    errors.push(PreflightIssue {
                        code: "SL_RESOURCE_MISSING_ROLE",
                        path: format!(
                            "$.resources[{resource_index}].bindings[{binding_index}].role"
                        ),
                        message: "binding role must be a non-empty string".to_string(),
                    });
                } else {
                    roles.insert(role.to_string());
                }
                let object_id = binding
                    .get("objectId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !is_four_segment_object_id(object_id) {
                    errors.push(PreflightIssue {
                        code: "SL_CATALOG_INVALID_OBJECT_ID",
                        path: format!(
                            "$.resources[{resource_index}].bindings[{binding_index}].objectId"
                        ),
                        message: "objectId must use four slash-separated segments".to_string(),
                    });
                }
            }
        }
        role_index.insert(resource_id.to_string(), roles);
    }
    role_index
}

fn validate_template_references(
    file_name: &str,
    bundle: &Value,
    role_index: &HashMap<String, HashSet<String>>,
    errors: &mut Vec<PreflightIssue>,
) {
    let Some(items) = bundle.get("items").and_then(Value::as_array) else {
        errors.push(PreflightIssue {
            code: "SL_TEMPLATE_MISSING_ITEMS",
            path: format!("$.templates.{file_name}.items"),
            message: "template items must be an array".to_string(),
        });
        return;
    };
    for (item_index, item) in items.iter().enumerate() {
        let resource_id = item.get("resourceId").and_then(Value::as_str).unwrap_or("");
        let Some(roles) = role_index.get(resource_id) else {
            errors.push(PreflightIssue {
                code: "SL_TEMPLATE_UNKNOWN_RESOURCE_ID",
                path: format!("$.templates.{file_name}.items[{item_index}].resourceId"),
                message: format!("template references unknown resourceId: {resource_id}"),
            });
            continue;
        };
        if let Some(bindings) = item.get("bindings").and_then(Value::as_array) {
            for (binding_index, binding) in bindings.iter().enumerate() {
                let role = binding.get("role").and_then(Value::as_str).unwrap_or("");
                if !roles.contains(role) {
                    errors.push(PreflightIssue {
                        code: "SL_TEMPLATE_UNKNOWN_ROLE",
                        path: format!("$.templates.{file_name}.items[{item_index}].bindings[{binding_index}].role"),
                        message: format!("template role is not defined under resourceId {resource_id}: {role}"),
                    });
                }
            }
        }
    }
}

fn is_four_segment_object_id(value: &str) -> bool {
    let segments = value.split('/').collect::<Vec<_>>();
    segments.len() == 4
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|ch| {
                    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_'
                })
        })
}

fn build_permission_summary_text(package_dir: &Path) -> String {
    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let summary = build_permission_summary_from_catalog(&catalog);
    let Some(items) = summary.get("items").and_then(Value::as_array) else {
        return "No managed permission objects. Open StoryLock Core to initialize the local package."
            .to_string();
    };
    let mut lines = Vec::new();
    for item in items {
        let resource_id = item
            .get("resourceId")
            .and_then(Value::as_str)
            .unwrap_or("unknown-resource");
        let display_name = item
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or(resource_id);
        let role = item
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown-role");
        let object_id = item
            .get("objectId")
            .and_then(Value::as_str)
            .unwrap_or("unknown-object");
        let object_kind = item
            .get("objectKind")
            .and_then(Value::as_str)
            .unwrap_or("secret");
        let action = item.get("action").and_then(Value::as_str).unwrap_or("read");
        let challenge_policy = item
            .get("challengePolicy")
            .and_then(Value::as_str)
            .unwrap_or("medium");
        let required_grid_count = item
            .get("requiredGridCount")
            .and_then(Value::as_u64)
            .unwrap_or(6);
        lines.push(format!(
            "{display_name} / {role}: objectId={object_id}, objectKind={object_kind}, action={action}, challengePolicy={challenge_policy}, requiredGridCount={required_grid_count}"
        ));
    }
    if lines.is_empty() {
        "No managed permission objects. Configure resources inside StoryLock Core.".to_string()
    } else {
        lines.join("\n")
    }
}

fn build_permission_summary_from_catalog(catalog: &Value) -> Value {
    let mut items = Vec::new();
    for resource in catalog
        .get("resources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let resource_id = resource
            .get("resourceId")
            .and_then(Value::as_str)
            .unwrap_or("unknown-resource");
        let resource_kind = resource
            .get("resourceKind")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let provider_id = resource
            .get("providerId")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let display_name = resource
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or(resource_id);
        for binding in resource
            .get("bindings")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let role = binding
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("unknown-role");
            let object_id = binding
                .get("objectId")
                .and_then(Value::as_str)
                .unwrap_or("unknown-object");
            let meta = binding.get("objectMeta").unwrap_or(&Value::Null);
            let object_kind = meta
                .get("objectKind")
                .and_then(Value::as_str)
                .unwrap_or("secret");
            let sensitivity = meta
                .get("sensitivity")
                .and_then(Value::as_str)
                .unwrap_or("private");
            items.push(json!({
                "resourceId": resource_id,
                "resourceKind": resource_kind,
                "providerId": provider_id,
                "displayName": display_name,
                "role": role,
                "objectId": object_id,
                "objectKind": object_kind,
                "sensitivity": sensitivity,
                "action": permission_action(object_kind),
                "challengePolicy": permission_challenge_policy(sensitivity),
                "requiredGridCount": permission_required_grid_count(sensitivity)
            }));
        }
    }
    json!({ "items": items })
}

fn permission_action(object_kind: &str) -> &'static str {
    match object_kind {
        "private_key" | "signing_key" => "sign",
        "password" => "password_fill",
        _ => "read",
    }
}

fn permission_challenge_policy(sensitivity: &str) -> &'static str {
    match sensitivity {
        "secret" | "high" => "high",
        _ => "medium",
    }
}

fn permission_required_grid_count(sensitivity: &str) -> u8 {
    match sensitivity {
        "secret" | "high" => 12,
        _ => 6,
    }
}

fn default_author_draft_json() -> Value {
    const ELEMENTS: [&str; 8] = [
        "time", "place", "person", "theme", "conflict", "plot", "choice", "ending",
    ];
    let nodes = (1..=24)
        .map(|index| {
            let element_id = ELEMENTS[(index - 1) % ELEMENTS.len()];
            json!({
                "nodeId": format!("node-{index:02}"),
                "title": format!("Node {index:02}"),
                "elementId": element_id,
                "question": format!("Story memory question {index:02}?"),
                "recommendedSelectionMode": "multi_select",
                "recommendedCorrectCount": 3,
                "candidatePoolSize": 9,
                "recallPriority": "medium",
                "verifyPolicy": "caseInsensitive + trim",
                "editorNotes": "Local author draft only."
            })
        })
        .collect::<Vec<_>>();
    json!({
        "version": "1",
        "storyTitle": "梅雨傍晚的旧火车站录音卡",
        "summary": "梅雨季的周三傍晚，林澈在南城旧火车站二号候车厅取到藏有录音卡的蓝色保温杯。",
        "memoryAnchors": ["梅雨季", "周三傍晚", "17号寄存柜", "蓝色保温杯"],
        "elementGroups": ["时间", "地点", "人物", "外部", "起步", "核心", "过程", "收束"],
        "nodes": nodes
    })
}

fn default_resource_catalog_json() -> Value {
    json!({
        "version": "1",
        "resources": [{
            "resourceId": "github-main",
            "resourceKind": "website_account",
            "providerId": "github",
            "displayName": "GitHub 主账号",
            "bindings": [
                {
                    "role": "username",
                    "objectId": "credential/github/main/username",
                    "objectMeta": { "objectKind": "username", "encoding": "text", "sensitivity": "private" }
                },
                {
                    "role": "password",
                    "objectId": "credential/github/main/password",
                    "objectMeta": { "objectKind": "password", "encoding": "secret", "sensitivity": "secret" }
                }
            ]
        }]
    })
}

fn default_login_templates_json() -> Value {
    json!({
        "version": "1",
        "templateType": "login-sites",
        "items": [{
            "templateId": "github.com",
            "displayName": "GitHub 主账号登录",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "username", "role": "username" },
                { "fieldName": "password", "role": "password" }
            ]
        }]
    })
}

fn default_signing_templates_json() -> Value {
    json!({
        "version": "1",
        "templateType": "signing-actions",
        "items": [{
            "templateId": "local-signature-placeholder",
            "displayName": "Local signature placeholder",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "username", "role": "username" }
            ]
        }]
    })
}

fn default_agent_templates_json() -> Value {
    json!({
        "version": "1",
        "templateType": "agent-tasks",
        "items": [{
            "templateId": "local-agent-placeholder",
            "displayName": "Local agent placeholder",
            "resourceId": "github-main",
            "bindings": [
                { "fieldName": "username", "role": "username" }
            ]
        }]
    })
}

fn json_string(value: &Value, path: &[&str]) -> SharedString {
    let mut current = value;
    for key in path {
        current = current.get(*key).unwrap_or(&Value::Null);
    }
    SharedString::from(current.as_str().unwrap_or(""))
}

fn split_list(value: &str, delimiter: &str) -> Vec<String> {
    value
        .split(delimiter)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn sanitize_segment(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if normalized.trim_matches('_').is_empty() {
        "local".to_string()
    } else {
        normalized
    }
}

fn format_bindings(resource: &Value) -> String {
    resource
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .map(|binding| {
                    format!(
                        "{} -> {}",
                        binding.get("role").and_then(Value::as_str).unwrap_or(""),
                        binding
                            .get("objectId")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn format_object_meta(resource: &Value) -> String {
    resource
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .map(|binding| {
                    let meta = binding.get("objectMeta").unwrap_or(&Value::Null);
                    format!(
                        "{}: {} {}",
                        binding.get("role").and_then(Value::as_str).unwrap_or(""),
                        meta.get("objectKind")
                            .and_then(Value::as_str)
                            .unwrap_or("secret"),
                        meta.get("sensitivity")
                            .and_then(Value::as_str)
                            .unwrap_or("private")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn format_template_bindings(template: &Value) -> String {
    template
        .get("bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .map(|binding| {
                    format!(
                        "{} -> {}",
                        binding
                            .get("fieldName")
                            .and_then(Value::as_str)
                            .unwrap_or(""),
                        binding.get("role").and_then(Value::as_str).unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn format_all_template_bundles(package_dir: &Path) -> String {
    [
        ("login-sites.json", default_login_templates_json()),
        ("signing-actions.json", default_signing_templates_json()),
        ("agent-tasks.json", default_agent_templates_json()),
    ]
    .iter()
    .map(|(file_name, fallback)| {
        let bundle = read_json_or_default(
            &storylock_core_template_path(package_dir, file_name),
            fallback.clone(),
        );
        let items = bundle
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        format!(
                            "  {} ({})\n{}",
                            item.get("templateId")
                                .and_then(Value::as_str)
                                .unwrap_or("template"),
                            item.get("resourceId")
                                .and_then(Value::as_str)
                                .unwrap_or("resource"),
                            format_template_bindings(item)
                                .lines()
                                .map(|line| format!("    {line}"))
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        format!("{file_name}\n{items}")
    })
    .collect::<Vec<_>>()
    .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use uuid::Uuid;

    fn temp_core_dir() -> PathBuf {
        std::env::temp_dir().join(format!("storylock_core_ui_test_{}", Uuid::new_v4()))
    }

    #[test]
    fn initializes_storylock_core_package_files() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        assert!(storylock_core_manifest_path(&dir).exists());
        assert!(storylock_core_catalog_path(&dir).exists());
        assert!(storylock_core_author_draft_path(&dir).exists());
        assert!(storylock_core_template_path(&dir, "login-sites.json").exists());
        assert!(storylock_core_template_path(&dir, "signing-actions.json").exists());
        assert!(storylock_core_template_path(&dir, "agent-tasks.json").exists());
    }

    #[test]
    fn export_preview_is_redacted() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let preview = build_export_preview(&dir);
        assert!(preview.contains("permissionObjects=2"));
        assert!(preview.contains("preflight=OK"));
        assert!(preview.contains("Host-readable permission summary only"));
        assert!(!preview.contains("signingKeyBytes="));
        assert!(!preview.contains("privateKey="));
        assert!(!preview.contains("password="));
    }

    #[test]
    fn host_permission_summary_is_derived_and_redacted() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let summary = build_permission_summary_text(&dir);
        assert!(summary.contains("GitHub 主账号 / username"));
        assert!(summary.contains("action=password_fill"));
        assert!(summary.contains("requiredGridCount=12"));
        assert!(!summary.contains("canonicalAnswer"));
        assert!(!summary.contains("acceptedAnswers"));
        assert!(!summary.contains("signingKeyBytes"));
        assert!(!summary.contains("privateKey="));
        assert!(!summary.contains("password="));
    }

    #[test]
    fn windows_permission_summary_matches_shared_js_contract() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let catalog = read_json_or_default(
            &storylock_core_catalog_path(&dir),
            default_resource_catalog_json(),
        );
        let rust_summary = build_permission_summary_from_catalog(&catalog);
        let output = Command::new("node")
            .args([
                "scripts/storylock-package/permission-summary-json.mjs",
                "--input",
            ])
            .arg(storylock_core_catalog_path(&dir))
            .current_dir(
                std::env::current_dir()
                    .expect("current dir")
                    .ancestors()
                    .find(|path| path.join("package.json").exists())
                    .expect("workspace root"),
            )
            .output()
            .expect("run js permission summary");
        assert!(
            output.status.success(),
            "js permission summary failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let js_summary: Value =
            serde_json::from_slice(&output.stdout).expect("parse js permission summary");
        assert_eq!(rust_summary, js_summary);
    }

    #[test]
    fn template_bundle_summary_covers_three_template_files() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let summary = format_all_template_bundles(&dir);
        assert!(summary.contains("login-sites.json"));
        assert!(summary.contains("signing-actions.json"));
        assert!(summary.contains("agent-tasks.json"));
        assert!(summary.contains("username -> username"));
        assert!(!summary.contains("password="));
        assert!(!summary.contains("privateKey="));
    }

    #[test]
    fn preflight_reports_invalid_template_role() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        fs::write(
            storylock_core_template_path(&dir, "agent-tasks.json"),
            serde_json::to_vec_pretty(&json!({
                "version": "1",
                "templateType": "agent-tasks",
                "items": [{
                    "templateId": "broken-agent",
                    "resourceId": "github-main",
                    "bindings": [
                        { "fieldName": "missing", "role": "missing_role" }
                    ]
                }]
            }))
            .expect("serialize broken template"),
        )
        .expect("write broken template");
        let result = preflight_storylock_core_package(&dir);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.code == "SL_TEMPLATE_UNKNOWN_ROLE"));
        let preview = build_export_preview(&dir);
        assert!(preview.contains("preflight=FAILED"));
        assert!(preview.contains("SL_TEMPLATE_UNKNOWN_ROLE"));
    }

    #[test]
    fn writes_all_twenty_four_node_slots() {
        let mut draft = default_author_draft_json();
        let fake_core = StoryLockCoreApp::new().expect("core app");
        fake_core.set_node_index(23);
        fake_core.set_node_id(SharedString::from("node-24-custom"));
        fake_core.set_node_title(SharedString::from("Custom Node 24"));
        fake_core.set_element_id(SharedString::from("ending"));
        fake_core.set_question_text(SharedString::from("Custom question 24?"));
        fake_core.set_selection_mode(SharedString::from("multi_select"));
        fake_core.set_correct_count(SharedString::from("3"));
        fake_core.set_candidate_pool_size(SharedString::from("9"));
        fake_core.set_recall_priority(SharedString::from("high"));
        fake_core.set_verify_policy(SharedString::from("caseInsensitive + trim"));
        fake_core.set_editor_notes(SharedString::from("local only"));
        fake_core.set_canonical_answer(SharedString::from("local answer"));
        fake_core.set_accepted_answers(SharedString::from("local answer; answer alt"));
        fake_core.set_correct_options(SharedString::from("A; B; C"));
        fake_core.set_distractors(SharedString::from("D; E; F"));
        write_current_node_to_draft(&fake_core, &mut draft);
        let nodes = draft.get("nodes").and_then(Value::as_array).expect("nodes");
        assert_eq!(nodes.len(), 24);
        assert_eq!(
            nodes[23].get("nodeId").and_then(Value::as_str),
            Some("node-24-custom")
        );
        assert_eq!(
            nodes[23].get("question").and_then(Value::as_str),
            Some("Custom question 24?")
        );
    }

    #[test]
    fn default_author_draft_has_twenty_four_nodes() {
        let draft = default_author_draft_json();
        assert_eq!(
            draft.get("nodes").and_then(Value::as_array).map(Vec::len),
            Some(24)
        );
    }
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
