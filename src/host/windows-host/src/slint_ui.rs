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
    import { Button, ComboBox, LineEdit, ScrollView, VerticalBox, HorizontalBox } from "std-widgets.slint";

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

    component QuestionTableRow inherits Rectangle {
        in property <string> label;
        in-out property <string> value;
        in-out property <string> state;
        in property <bool> show-state;

        width: 690px;
        height: 32px;
        background: transparent;
        Rectangle {
            x: 0px;
            y: 0px;
            width: 110px;
            height: 32px;
            border-width: 1px;
            border-color: #d7e1e6;
            border-radius: 4px;
            background: #edf3f5;
            Text {
                x: 10px;
                y: 0px;
                width: parent.width - 20px;
                height: parent.height;
                text: root.label;
                color: #62727d;
                font-size: 13px;
                horizontal-alignment: left;
                vertical-alignment: center;
                overflow: elide;
            }
        }
        LineEdit {
            x: 122px;
            y: 0px;
            width: root.show-state ? 408px : 548px;
            height: 32px;
            text <=> root.value;
        }
        if root.show-state: ComboBox {
            x: 540px;
            y: 0px;
            width: 130px;
            height: 32px;
            model: ["wrong", "correct"];
            current-value <=> root.state;
        }
    }

    component QuestionIdTableRow inherits Rectangle {
        in-out property <string> selected-question;
        callback previous-node();
        callback next-node();
        callback select-node(string);

        width: 690px;
        height: 32px;
        background: transparent;
        Rectangle {
            x: 0px;
            y: 0px;
            width: 110px;
            height: 32px;
            border-width: 1px;
            border-color: #d7e1e6;
            border-radius: 4px;
            background: #edf3f5;
            Text {
                x: 10px;
                y: 0px;
                width: parent.width - 20px;
                height: parent.height;
                text: "Question ID";
                color: #62727d;
                font-size: 13px;
                horizontal-alignment: left;
                vertical-alignment: center;
                overflow: elide;
            }
        }
        Button {
            x: 122px;
            y: 0px;
            width: 130px;
            height: 32px;
            text: "Prev";
            clicked => { root.previous-node(); }
        }
        ComboBox {
            x: 262px;
            y: 0px;
            width: 268px;
            height: 32px;
            model: ["1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19","20","21","22","23","24"];
            current-value <=> root.selected-question;
            selected(value) => { root.select-node(value); }
        }
        Button {
            x: 540px;
            y: 0px;
            width: 130px;
            height: 32px;
            text: "Next";
            clicked => { root.next-node(); }
        }
    }

    component LearningAnswerRow inherits Rectangle {
        in property <string> label;
        in property <string> answer;
        in-out property <string> state;

        width: 690px;
        height: 32px;
        background: transparent;
        Rectangle {
            x: 0px;
            y: 0px;
            width: 110px;
            height: 32px;
            border-width: 1px;
            border-color: #d7e1e6;
            border-radius: 4px;
            background: #edf3f5;
            Text {
                x: 10px;
                width: parent.width - 20px;
                height: parent.height;
                text: root.label;
                color: #62727d;
                font-size: 13px;
                vertical-alignment: center;
                overflow: elide;
            }
        }
        Rectangle {
            x: 122px;
            y: 0px;
            width: 408px;
            height: 32px;
            border-width: 1px;
            border-color: #d7e1e6;
            border-radius: 4px;
            background: #f7fafb;
            Text {
                x: 10px;
                width: parent.width - 20px;
                height: parent.height;
                text: root.answer;
                color: #17252f;
                font-size: 13px;
                vertical-alignment: center;
                overflow: elide;
            }
        }
        ComboBox {
            x: 540px;
            y: 0px;
            width: 130px;
            height: 32px;
            model: ["wrong", "correct"];
            current-value <=> root.state;
        }
    }

    component LearningStaticRow inherits HorizontalLayout {
        in property <string> name;
        in property <string> value;
        spacing: 12px;
        Text {
            text: root.name;
            width: 110px;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        Rectangle {
            width: 548px;
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

    component LearningTestPanel inherits Rectangle {
        in property <string> position;
        in property <string> question;
        in property <string> status;
        in property <string> answer-1;
        in-out property <string> answer-1-state;
        in property <string> answer-2;
        in-out property <string> answer-2-state;
        in property <string> answer-3;
        in-out property <string> answer-3-state;
        in property <string> answer-4;
        in-out property <string> answer-4-state;
        in property <string> answer-5;
        in-out property <string> answer-5-state;
        in property <string> answer-6;
        in-out property <string> answer-6-state;
        in property <string> answer-7;
        in-out property <string> answer-7-state;
        in property <string> answer-8;
        in-out property <string> answer-8-state;
        in property <string> answer-9;
        in-out property <string> answer-9-state;
        callback previous();
        callback next();
        callback check-current();

        width: 740px;
        height: 430px;
        background: transparent;
        VerticalBox {
            x: 10px;
            y: 0px;
            width: 690px;
            spacing: 8px;
            LearningStaticRow { name: "Current"; value: root.position; }
            LearningStaticRow { name: "Question"; value: root.question; }
            LearningAnswerRow { label: "Answer 1"; answer: root.answer-1; state <=> root.answer-1-state; }
            LearningAnswerRow { label: "Answer 2"; answer: root.answer-2; state <=> root.answer-2-state; }
            LearningAnswerRow { label: "Answer 3"; answer: root.answer-3; state <=> root.answer-3-state; }
            LearningAnswerRow { label: "Answer 4"; answer: root.answer-4; state <=> root.answer-4-state; }
            LearningAnswerRow { label: "Answer 5"; answer: root.answer-5; state <=> root.answer-5-state; }
            LearningAnswerRow { label: "Answer 6"; answer: root.answer-6; state <=> root.answer-6-state; }
            LearningAnswerRow { label: "Answer 7"; answer: root.answer-7; state <=> root.answer-7-state; }
            LearningAnswerRow { label: "Answer 8"; answer: root.answer-8; state <=> root.answer-8-state; }
            LearningAnswerRow { label: "Answer 9"; answer: root.answer-9; state <=> root.answer-9-state; }
            HorizontalBox {
                spacing: 10px;
                Button { text: "Prev"; clicked => { root.previous(); } }
                Button { text: "Check Current"; clicked => { root.check-current(); } }
                Button { text: "Next"; clicked => { root.next(); } }
            }
            LearningStaticRow { name: "Result"; value: root.status; }
        }
    }

    component QuestionConfigPanel inherits Rectangle {
        in-out property <string> selected-question;
        in-out property <string> question-text;
        in-out property <string> answer-1;
        in-out property <string> answer-1-state;
        in-out property <string> answer-2;
        in-out property <string> answer-2-state;
        in-out property <string> answer-3;
        in-out property <string> answer-3-state;
        in-out property <string> answer-4;
        in-out property <string> answer-4-state;
        in-out property <string> answer-5;
        in-out property <string> answer-5-state;
        in-out property <string> answer-6;
        in-out property <string> answer-6-state;
        in-out property <string> answer-7;
        in-out property <string> answer-7-state;
        in-out property <string> answer-8;
        in-out property <string> answer-8-state;
        in-out property <string> answer-9;
        in-out property <string> answer-9-state;
        callback previous-node();
        callback next-node();
        callback select-node(string);

        width: 740px;
        height: 540px;
        border-radius: 6px;
        border-width: 0px;
        background: transparent;

        ScrollView {
            x: 10px;
            y: 10px;
            width: 720px;
            height: 470px;
            vertical-scrollbar-policy: ScrollBarPolicy.as-needed;
            viewport-width: 720px;
            viewport-height: 520px;
            VerticalBox {
                x: 0px;
                y: 0px;
                width: 690px;
                height: 520px;
                spacing: 8px;
                QuestionIdTableRow {
                    selected-question <=> root.selected-question;
                    previous-node => { root.previous-node(); }
                    next-node => { root.next-node(); }
                    select-node(value) => { root.select-node(value); }
                }
                QuestionTableRow { label: "Name"; value <=> root.question-text; state: ""; show-state: false; }
                QuestionTableRow { label: "Answer 1"; value <=> root.answer-1; state <=> root.answer-1-state; show-state: true; }
                QuestionTableRow { label: "Answer 2"; value <=> root.answer-2; state <=> root.answer-2-state; show-state: true; }
                QuestionTableRow { label: "Answer 3"; value <=> root.answer-3; state <=> root.answer-3-state; show-state: true; }
                QuestionTableRow { label: "Answer 4"; value <=> root.answer-4; state <=> root.answer-4-state; show-state: true; }
                QuestionTableRow { label: "Answer 5"; value <=> root.answer-5; state <=> root.answer-5-state; show-state: true; }
                QuestionTableRow { label: "Answer 6"; value <=> root.answer-6; state <=> root.answer-6-state; show-state: true; }
                QuestionTableRow { label: "Answer 7"; value <=> root.answer-7; state <=> root.answer-7-state; show-state: true; }
                QuestionTableRow { label: "Answer 8"; value <=> root.answer-8; state <=> root.answer-8-state; show-state: true; }
                QuestionTableRow { label: "Answer 9"; value <=> root.answer-9; state <=> root.answer-9-state; show-state: true; }
            }
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
        in property <string> core-package-dir;
        in property <string> core-manifest-path;
        in property <string> core-catalog-path;
        in property <string> core-author-draft-path;
        in property <string> core-temp-draft-path;
        in property <string> core-templates-dir;
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
                            StaticRow { name: "Core Package"; value: root.core-package-dir; }
                            StaticRow { name: "Manifest"; value: root.core-manifest-path; }
                            StaticRow { name: "Catalog"; value: root.core-catalog-path; }
                            StaticRow { name: "Author Draft"; value: root.core-author-draft-path; }
                            StaticRow { name: "Temp Draft"; value: root.core-temp-draft-path; }
                            StaticRow { name: "Templates"; value: root.core-templates-dir; }
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
                            ActionButton {
                                label: "Open StoryLock Core";
                                primary: true;
                                clicked => {
                                    root.core-launch-status = "StoryLock Core opened in a separate local window. Host remains read-only.";
                                    root.open-storylock-core();
                                }
                            }
                            StaticRow { name: "Status"; value: root.core-launch-status; }
                            StaticRow { name: "Mode"; value: "Local editor only"; }
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
        in-out property <string> selected-question: "1";
        in-out property <string> question-text: "故事发生在什么季节和星期几？";
        in-out property <string> canonical-answer: "梅雨季的一个周三傍晚。";
        in-out property <string> accepted-answers: "梅雨季周三; 周三傍晚; 梅雨季";
        in-out property <string> answer-options: "1. 梅雨季 | correct\n2. 周三傍晚 | correct\n3. 发车铃前二十分钟 | correct\n4. 初雪清晨 | wrong\n5. 周日午夜 | wrong\n6. 海边码头 | wrong\n7. 红色背包 | wrong\n8. 3号寄存柜 | wrong\n9. 午夜电台 | wrong";
        in-out property <string> correct-options: "1,2,3";
        in-out property <string> answer-1: "梅雨季";
        in-out property <string> answer-1-state: "correct";
        in-out property <string> answer-2: "周三傍晚";
        in-out property <string> answer-2-state: "correct";
        in-out property <string> answer-3: "发车铃前二十分钟";
        in-out property <string> answer-3-state: "correct";
        in-out property <string> answer-4: "初雪清晨";
        in-out property <string> answer-4-state: "wrong";
        in-out property <string> answer-5: "周日午夜";
        in-out property <string> answer-5-state: "wrong";
        in-out property <string> answer-6: "海边码头";
        in-out property <string> answer-6-state: "wrong";
        in-out property <string> answer-7: "红色背包";
        in-out property <string> answer-7-state: "wrong";
        in-out property <string> answer-8: "3号寄存柜";
        in-out property <string> answer-8-state: "wrong";
        in-out property <string> answer-9: "午夜电台";
        in-out property <string> answer-9-state: "wrong";
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
        in-out property <string> object-id: "wallet/evm/main/signing_key";
        in-out property <string> object-kind: "private_key";
        in-out property <string> required-correct-count: "12";
        in-out property <string> authorization-frequency: "Every high-risk request";
        in-out property <string> secret-reference: "Windows DPAPI / Credential Manager local reference";
        in-out property <string> training-policy: "Complete local learning review before saving.";
        in-out property <string> resource-bindings: "username -> credential/github/main/username\npassword -> credential/github/main/password\ntotp_secret -> credential/github/main/totp_secret";
        in-out property <string> object-meta: "username: reference utf8\npassword: secret utf8\ntotp_secret: secret utf8";
        in-out property <string> template-kind: "login-sites.json";
        in-out property <string> template-id: "github.com";
        in-out property <string> template-display-name: "GitHub 主账号登录";
        in-out property <string> template-bindings: "login-sites.json\n  username -> username\n  password -> password\n\nsigning-actions.json\n  username -> username\n\nagent-tasks.json\n  username -> username";
        in-out property <string> export-preview: "identity-package/\n  vault.stlk\n  resource-catalog.json\n  package-manifest.json\n  templates/login-sites.json\n  templates/signing-actions.json\n  templates/agent-tasks.json";
        in-out property <string> config-status: "All edits stay inside StoryLock Core. Host receives only derived permission metadata.";
        in-out property <string> learning-status: "Learning test is required before export.";
        in-out property <bool> export-ready: false;
        in-out property <int> learning-index: 0;
        in-out property <string> learning-position: "1 / 24";
        in-out property <string> learning-question: "";
        in-out property <string> learning-result: "Select correct/wrong for all 9 answers, then check current question.";
        in-out property <string> learning-answer-1: "";
        in-out property <string> learning-answer-1-state: "wrong";
        in-out property <string> learning-answer-2: "";
        in-out property <string> learning-answer-2-state: "wrong";
        in-out property <string> learning-answer-3: "";
        in-out property <string> learning-answer-3-state: "wrong";
        in-out property <string> learning-answer-4: "";
        in-out property <string> learning-answer-4-state: "wrong";
        in-out property <string> learning-answer-5: "";
        in-out property <string> learning-answer-5-state: "wrong";
        in-out property <string> learning-answer-6: "";
        in-out property <string> learning-answer-6-state: "wrong";
        in-out property <string> learning-answer-7: "";
        in-out property <string> learning-answer-7-state: "wrong";
        in-out property <string> learning-answer-8: "";
        in-out property <string> learning-answer-8-state: "wrong";
        in-out property <string> learning-answer-9: "";
        in-out property <string> learning-answer-9-state: "wrong";
        in-out property <string> core-data-dir: "";
        property <string> current-title: active-page == 0 ? "24 Questions" : active-page == 1 ? "Managed Objects" : active-page == 2 ? "Save Draft" : active-page == 3 ? "Story Aids" : "Export";
        callback close-requested();
        callback save-temp-draft();
        callback previous-node();
        callback next-node();
        callback select-node(string);
        callback save-resource();
        callback save-template();
        callback refresh-export();
        callback run-learning();
        callback learning-previous();
        callback learning-next();
        callback check-learning-current();
        callback export-package();

        title: "StoryLock Core";
        preferred-width: 1080px;
        preferred-height: 780px;
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
                        label: "24 Questions";
                        selected: root.active-page == 0;
                        clicked => { root.active-page = 0; }
                    }
                    MenuButton {
                        label: "Managed Objects";
                        selected: root.active-page == 1;
                        clicked => { root.active-page = 1; }
                    }
                    MenuButton {
                        label: "Save Draft";
                        selected: root.active-page == 2;
                        clicked => { root.active-page = 2; }
                    }
                    MenuButton {
                        label: "Story Aids";
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
                min-width: 820px;
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
                            text: "StoryLock Core - " + root.current-title;
                            font-size: 16px;
                            font-weight: 800;
                            color: #17252f;
                            overflow: elide;
                        }
                        ActionButton {
                            x: 600px;
                            y: 1px;
                            label: "Save Temp Draft";
                            primary: true;
                            clicked => {
                                root.save-temp-draft();
                            }
                        }
                    }

                    Rectangle { height: 1px; background: #d7e1e6; }

                    Rectangle {
                        width: 780px;
                        height: 590px;
                        background: transparent;

                        if root.active-page == 0: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 780px;
                            height: 590px;
                            spacing: 8px;
                            QuestionConfigPanel {
                                selected-question <=> root.selected-question;
                                question-text <=> root.question-text;
                                answer-1 <=> root.answer-1;
                                answer-1-state <=> root.answer-1-state;
                                answer-2 <=> root.answer-2;
                                answer-2-state <=> root.answer-2-state;
                                answer-3 <=> root.answer-3;
                                answer-3-state <=> root.answer-3-state;
                                answer-4 <=> root.answer-4;
                                answer-4-state <=> root.answer-4-state;
                                answer-5 <=> root.answer-5;
                                answer-5-state <=> root.answer-5-state;
                                answer-6 <=> root.answer-6;
                                answer-6-state <=> root.answer-6-state;
                                answer-7 <=> root.answer-7;
                                answer-7-state <=> root.answer-7-state;
                                answer-8 <=> root.answer-8;
                                answer-8-state <=> root.answer-8-state;
                                answer-9 <=> root.answer-9;
                                answer-9-state <=> root.answer-9-state;
                                previous-node => { root.previous-node(); }
                                next-node => { root.next-node(); }
                                select-node(value) => { root.select-node(value); }
                            }
                        }

                        if root.active-page == 1: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 780px;
                            height: 590px;
                            spacing: 12px;
                            EditableRow { name: "Resource ID"; value <=> root.resource-id; }
                            EditableRow { name: "Object ID"; value <=> root.object-id; }
                            EditableRow { name: "Object Kind"; value <=> root.object-kind; }
                            EditableRow { name: "Display Name"; value <=> root.display-name; }
                            EditableRow { name: "Need Correct"; value <=> root.required-correct-count; }
                            EditableRow { name: "Secret Ref"; value <=> root.secret-reference; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Save Object";
                                    primary: true;
                                    clicked => {
                                        root.save-resource();
                                    }
                                }
                            }
                            LogPanel { value: "bindings:\n" + root.resource-bindings + "\n\nobjectMeta:\n" + root.object-meta; panel-height: 150px; }
                        }

                        if root.active-page == 2: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 780px;
                            height: 590px;
                            spacing: 12px;
                            StaticRow { name: "Package Kind"; value: "storylock_identity_package"; }
                            EditableRow { name: "Temp Save Note"; value <=> root.training-policy; }
                            EditableRow { name: "High Auth Freq"; value <=> root.authorization-frequency; }
                            StaticRow { name: "Save Target"; value: ".tmp/author-draft.pending.json only"; }
                            StaticRow { name: "Export Gate"; value: "Use Export after learning test passes."; }
                            StaticRow { name: "Global Save"; value: "Use the top Save Temp Draft button."; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Refresh";
                                    primary: false;
                                    clicked => {
                                        root.refresh-export();
                                    }
                                }
                            }
                            LogPanel { value: root.config-status + "\n\nThe top Save Temp Draft button stores current StoryLock Core memory as a temporary draft. Export promotes it after learning test passes."; panel-height: 230px; }
                        }

                        if root.active-page == 3: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 780px;
                            height: 590px;
                            spacing: 12px;
                            StaticRow { name: "Purpose"; value: "Story and plot templates serve the 24 questions."; }
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
                                    label: "Refresh";
                                    primary: false;
                                    clicked => {
                                        root.refresh-export();
                                    }
                                }
                            }
                            LogPanel { value: root.config-status; panel-height: 100px; }
                        }

                        if root.active-page == 4: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 780px;
                            height: 590px;
                            spacing: 8px;
                            StaticRow { name: "Learning Status"; value: root.learning-status; }
                            LearningTestPanel {
                                position: root.learning-position;
                                question: root.learning-question;
                                status: root.learning-result;
                                answer-1: root.learning-answer-1;
                                answer-1-state <=> root.learning-answer-1-state;
                                answer-2: root.learning-answer-2;
                                answer-2-state <=> root.learning-answer-2-state;
                                answer-3: root.learning-answer-3;
                                answer-3-state <=> root.learning-answer-3-state;
                                answer-4: root.learning-answer-4;
                                answer-4-state <=> root.learning-answer-4-state;
                                answer-5: root.learning-answer-5;
                                answer-5-state <=> root.learning-answer-5-state;
                                answer-6: root.learning-answer-6;
                                answer-6-state <=> root.learning-answer-6-state;
                                answer-7: root.learning-answer-7;
                                answer-7-state <=> root.learning-answer-7-state;
                                answer-8: root.learning-answer-8;
                                answer-8-state <=> root.learning-answer-8-state;
                                answer-9: root.learning-answer-9;
                                answer-9-state <=> root.learning-answer-9-state;
                                previous => { root.learning-previous(); }
                                next => { root.learning-next(); }
                                check-current => { root.check-learning-current(); }
                            }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: "Start Test";
                                    primary: true;
                                    clicked => {
                                        root.run-learning();
                                    }
                                }
                                ActionButton {
                                    label: "Export";
                                    primary: root.export-ready;
                                    clicked => {
                                        root.export-package();
                                    }
                                }
                            }
                            LogPanel { value: root.export-preview + "\n\nExport replaces the external managed key package only after all 24 learning questions pass."; panel-height: 82px; }
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
    app.set_core_package_dir(SharedString::from(core_package_dir.display().to_string()));
    app.set_core_manifest_path(SharedString::from(
        storylock_core_manifest_path(&core_package_dir).display().to_string(),
    ));
    app.set_core_catalog_path(SharedString::from(
        storylock_core_catalog_path(&core_package_dir).display().to_string(),
    ));
    app.set_core_author_draft_path(SharedString::from(
        storylock_core_author_draft_path(&core_package_dir).display().to_string(),
    ));
    app.set_core_temp_draft_path(SharedString::from(
        storylock_core_pending_author_draft_path(&core_package_dir)
            .display()
            .to_string(),
    ));
    app.set_core_templates_dir(SharedString::from(
        core_package_dir.join("templates").display().to_string(),
    ));
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
        if let Some(core) = core_windows_for_callback.borrow().last() {
            initialize_storylock_core_window(core, &core_package_dir);
            core.set_config_status(SharedString::from(
                "StoryLock Core is already open. Existing local window was focused.",
            ));
            if let Err(error) = core.show() {
                eprintln!("failed to show existing StoryLock Core window: {error}");
            }
            return;
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

fn storylock_core_pending_author_draft_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join(".tmp").join("author-draft.pending.json")
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
    replace_legacy_default_author_draft(package_dir)?;
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

fn replace_legacy_default_author_draft(package_dir: &Path) -> Result<()> {
    let path = storylock_core_author_draft_path(package_dir);
    let draft = read_json_or_default(&path, default_author_draft_json());
    let title = draft.get("storyTitle").and_then(Value::as_str).unwrap_or("");
    let first_question = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.first())
        .and_then(|node| node.get("question"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if title.contains("旧火车站") || title.contains("棫鐏") || first_question.starts_with("Story memory question") {
        fs::write(path, serde_json::to_vec_pretty(&default_author_draft_json())?)?;
    }
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

fn read_effective_author_draft(package_dir: &Path) -> Value {
    let pending_path = storylock_core_pending_author_draft_path(package_dir);
    if pending_path.exists() {
        return read_json_or_default(&pending_path, default_author_draft_json());
    }
    read_json_or_default(
        &storylock_core_author_draft_path(package_dir),
        default_author_draft_json(),
    )
}

fn write_pending_author_draft(package_dir: &Path, draft: &Value) -> Result<()> {
    let path = storylock_core_pending_author_draft_path(package_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(draft)?)?;
    Ok(())
}

fn initialize_storylock_core_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let draft = read_effective_author_draft(package_dir);
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
    load_learning_node_into_window(core, package_dir, 0);
}

fn wire_storylock_core_callbacks(
    core: &StoryLockCoreApp,
    package_dir: std::path::PathBuf,
    host_weak: slint::Weak<HostDashboard>,
) {
    let learning_passed = Rc::new(RefCell::new(vec![false; 24]));
    let weak = core.as_weak();
    core.on_close_requested(move || {
        if let Some(core) = weak.upgrade() {
            let _ = core.hide();
        }
    });

    let weak = core.as_weak();
    let temp_draft_dir = package_dir.clone();
    let temp_draft_host_weak = host_weak.clone();
    let temp_draft_learning_passed = Rc::clone(&learning_passed);
    core.on_save_temp_draft(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_temp_draft_from_window(&core, &temp_draft_dir);
            reset_learning_gate(
                &core,
                &temp_draft_learning_passed,
                "Temporary draft saved. Run learning test again before export.",
            );
            set_core_status(
                &core,
                result,
                "Current StoryLock Core memory saved as temporary draft.",
            );
            refresh_host_permission_summary(&temp_draft_host_weak, &temp_draft_dir);
        }
    });

    let weak = core.as_weak();
    let previous_node_dir = package_dir.clone();
    let previous_learning_passed = Rc::clone(&learning_passed);
    core.on_previous_node(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_current_node_from_window(&core, &previous_node_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &previous_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = core.get_node_index().saturating_sub(1);
            load_node_into_window(&core, &previous_node_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_node_dir = package_dir.clone();
    let next_learning_passed = Rc::clone(&learning_passed);
    core.on_next_node(move || {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_current_node_from_window(&core, &next_node_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &next_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = (core.get_node_index() + 1).min(23);
            load_node_into_window(&core, &next_node_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let select_node_dir = package_dir.clone();
    let select_learning_passed = Rc::clone(&learning_passed);
    core.on_select_node(move |value| {
        if let Some(core) = weak.upgrade() {
            if let Err(error) = save_current_node_from_window(&core, &select_node_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &select_learning_passed,
                "Question selection saved a draft. Run learning test again before export.",
            );
            let selected_index = value
                .parse::<i32>()
                .ok()
                .map(|number| number - 1)
                .unwrap_or_else(|| core.get_node_index());
            load_node_into_window(&core, &select_node_dir, selected_index);
        }
    });

    let weak = core.as_weak();
    let resource_dir = package_dir.clone();
    let resource_host_weak = host_weak.clone();
    let resource_learning_passed = Rc::clone(&learning_passed);
    core.on_save_resource(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_resource_from_window(&core, &resource_dir);
            reset_learning_gate(
                &core,
                &resource_learning_passed,
                "Managed object changed. Run learning test again before export.",
            );
            set_core_status(&core, result, "Resource catalog saved locally.");
            refresh_host_permission_summary(&resource_host_weak, &resource_dir);
        }
    });

    let weak = core.as_weak();
    let template_dir = package_dir.clone();
    let template_learning_passed = Rc::clone(&learning_passed);
    core.on_save_template(move || {
        if let Some(core) = weak.upgrade() {
            let result = save_template_from_window(&core, &template_dir);
            reset_learning_gate(
                &core,
                &template_learning_passed,
                "Template changed. Run learning test again before export.",
            );
            set_core_status(&core, result, "Login template saved locally.");
        }
    });

    let weak = core.as_weak();
    let refresh_host_weak = host_weak.clone();
    let refresh_dir = package_dir.clone();
    core.on_refresh_export(move || {
        if let Some(core) = weak.upgrade() {
            core.set_export_preview(SharedString::from(build_export_preview(&refresh_dir)));
            core.set_config_status(SharedString::from(
                "Export preview refreshed from local StoryLock Core package.",
            ));
            refresh_host_permission_summary(&refresh_host_weak, &refresh_dir);
        }
    });

    let weak = core.as_weak();
    let learning_dir = package_dir.clone();
    let run_learning_passed = Rc::clone(&learning_passed);
    core.on_run_learning(move || {
        if let Some(core) = weak.upgrade() {
            match run_export_learning_test(&learning_dir) {
                Ok(_) => {
                    run_learning_passed.borrow_mut().fill(false);
                    core.set_export_ready(false);
                    load_learning_node_into_window(&core, &learning_dir, 0);
                    core.set_learning_status(SharedString::from(
                        "Learning started: 0 / 24 questions passed. Match all 9 answers for each question.",
                    ));
                    core.set_config_status(SharedString::from(
                        "Learning test started. Export remains blocked until all 24 questions pass.",
                    ));
                }
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning test failed: {error}"
                    )));
                    core.set_config_status(SharedString::from(
                        "Export is blocked until learning test passes.",
                    ));
                }
            }
        }
    });

    let weak = core.as_weak();
    let previous_learning_dir = package_dir.clone();
    core.on_learning_previous(move || {
        if let Some(core) = weak.upgrade() {
            let next_index = core.get_learning_index().saturating_sub(1);
            load_learning_node_into_window(&core, &previous_learning_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_learning_dir = package_dir.clone();
    core.on_learning_next(move || {
        if let Some(core) = weak.upgrade() {
            let next_index = (core.get_learning_index() + 1).min(23);
            load_learning_node_into_window(&core, &next_learning_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let check_learning_dir = package_dir.clone();
    let check_learning_passed = Rc::clone(&learning_passed);
    core.on_check_learning_current(move || {
        if let Some(core) = weak.upgrade() {
            match check_learning_current(&core, &check_learning_dir, &check_learning_passed) {
                Ok(report) => {
                    core.set_learning_status(SharedString::from(report.clone()));
                    core.set_learning_result(SharedString::from(report));
                }
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    core.set_learning_result(SharedString::from(
                        "Current answer-state match failed. Review memory and try again.",
                    ));
                }
            }
        }
    });

    let weak = core.as_weak();
    let export_dir = package_dir.clone();
    let export_host_weak = host_weak.clone();
    core.on_export_package(move || {
        if let Some(core) = weak.upgrade() {
            if !core.get_export_ready() {
                core.set_config_status(SharedString::from(
                    "Export blocked. Run Learning Test successfully first.",
                ));
                return;
            }
            match export_storylock_package(&export_dir) {
                Ok(path) => {
                    core.set_export_preview(SharedString::from(build_export_preview(&export_dir)));
                    core.set_config_status(SharedString::from(format!(
                        "Export complete. Managed key package replaced at {}",
                        path.display()
                    )));
                    core.set_learning_status(SharedString::from(
                        "Learning test passed. Export completed.",
                    ));
                    refresh_host_permission_summary(&export_host_weak, &export_dir);
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Export failed: {error}"
                    )));
                }
            }
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

fn reset_learning_gate(
    core: &StoryLockCoreApp,
    learning_passed: &Rc<RefCell<Vec<bool>>>,
    message: &str,
) {
    core.set_export_ready(false);
    learning_passed.borrow_mut().fill(false);
    core.set_learning_status(SharedString::from(message));
    core.set_learning_result(SharedString::from(
        "Learning progress reset because local configuration changed.",
    ));
}

fn save_story_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let mut draft = read_effective_author_draft(package_dir);
    draft["storyTitle"] = json!(core.get_story_title().to_string());
    draft["summary"] = json!(core.get_story_summary().to_string());
    draft["memoryAnchors"] = json!(split_list(core.get_memory_anchors().as_str(), "/"));
    draft["elementGroups"] = json!(split_list(core.get_element_group().as_str(), ","));
    write_current_node_to_draft(core, &mut draft);
    write_pending_author_draft(package_dir, &draft)?;
    Ok(())
}

fn save_temp_draft_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    save_story_from_window(core, package_dir)?;
    save_resource_from_window(core, package_dir)?;
    save_template_from_window(core, package_dir)?;
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    Ok(())
}

fn save_current_node_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let mut draft = read_effective_author_draft(package_dir);
    write_current_node_to_draft(core, &mut draft);
    write_pending_author_draft(package_dir, &draft)?;
    core.set_node_output(SharedString::from(format!(
        "temporary draft saved for node {}\nnodeId={}\ntitle={}\nelementId={}\nquestion={}\n\nSaved to .tmp/author-draft.pending.json. Export promotes it only after learning test passes.",
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
        let answer_options = answer_options_from_window(core);
        node["candidatePoolSize"] = json!(answer_options.len() as u32);
        node["recallPriority"] = json!(core.get_recall_priority().to_string());
        node["verifyPolicy"] = json!(core.get_verify_policy().to_string());
        node["editorNotes"] = json!(core.get_editor_notes().to_string());
        node["canonicalAnswerLocalOnly"] = json!(core.get_canonical_answer().to_string());
        node["acceptedAnswersLocalOnly"] =
            json!(split_list(core.get_accepted_answers().as_str(), ";"));
        node["answerOptionsLocalOnly"] = json!(answer_options);
    }
}

fn load_node_into_window(core: &StoryLockCoreApp, package_dir: &Path, requested_index: i32) {
    let node_index = normalize_node_index(requested_index);
    let mut draft = read_effective_author_draft(package_dir);
    ensure_draft_nodes(&mut draft);
    let node = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.get(node_index))
        .cloned()
        .unwrap_or_else(|| default_author_draft_json()["nodes"][node_index].clone());
    core.set_node_index(node_index as i32);
    core.set_node_position(SharedString::from(format!("{} / 24", node_index + 1)));
    core.set_selected_question(SharedString::from((node_index + 1).to_string()));
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
    let answer_options = node_answer_options(&node);
    core.set_answer_options(SharedString::from(format_answer_options(&answer_options)));
    core.set_correct_options(SharedString::from(format_correct_option_indexes(
        &answer_options,
    )));
    set_answer_options_into_window(core, &answer_options);
    core.set_node_output(SharedString::from(format!(
        "loaded node {}\nnodeId={}\ntitle={}\n\nUse Save before closing. Answers and editor notes are local-core only.",
        node_index + 1,
        core.get_node_id(),
        core.get_node_title()
    )));
}

fn load_learning_node_into_window(core: &StoryLockCoreApp, package_dir: &Path, requested_index: i32) {
    let node_index = normalize_node_index(requested_index);
    let mut draft = read_effective_author_draft(package_dir);
    ensure_draft_nodes(&mut draft);
    let node = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.get(node_index))
        .cloned()
        .unwrap_or_else(|| default_author_draft_json()["nodes"][node_index].clone());
    let options = node_answer_options(&node);
    core.set_learning_index(node_index as i32);
    core.set_learning_position(SharedString::from(format!("{} / 24", node_index + 1)));
    core.set_learning_question(json_string(&node, &["question"]));
    set_learning_answers_into_window(core, &options);
    core.set_learning_result(SharedString::from(format!(
        "Question {} loaded. Mark each visible answer as correct or wrong from memory, then check current.",
        node_index + 1
    )));
}

fn set_learning_answers_into_window(core: &StoryLockCoreApp, options: &[Value]) {
    let answer_text = |index: usize| -> SharedString {
        SharedString::from(
            options
                .get(index)
                .and_then(|option| option.get("text"))
                .and_then(Value::as_str)
                .unwrap_or(""),
        )
    };
    core.set_learning_answer_1(answer_text(0));
    core.set_learning_answer_1_state(SharedString::from("wrong"));
    core.set_learning_answer_2(answer_text(1));
    core.set_learning_answer_2_state(SharedString::from("wrong"));
    core.set_learning_answer_3(answer_text(2));
    core.set_learning_answer_3_state(SharedString::from("wrong"));
    core.set_learning_answer_4(answer_text(3));
    core.set_learning_answer_4_state(SharedString::from("wrong"));
    core.set_learning_answer_5(answer_text(4));
    core.set_learning_answer_5_state(SharedString::from("wrong"));
    core.set_learning_answer_6(answer_text(5));
    core.set_learning_answer_6_state(SharedString::from("wrong"));
    core.set_learning_answer_7(answer_text(6));
    core.set_learning_answer_7_state(SharedString::from("wrong"));
    core.set_learning_answer_8(answer_text(7));
    core.set_learning_answer_8_state(SharedString::from("wrong"));
    core.set_learning_answer_9(answer_text(8));
    core.set_learning_answer_9_state(SharedString::from("wrong"));
}

fn learning_answer_states_from_window(core: &StoryLockCoreApp) -> Vec<bool> {
    [
        core.get_learning_answer_1_state(),
        core.get_learning_answer_2_state(),
        core.get_learning_answer_3_state(),
        core.get_learning_answer_4_state(),
        core.get_learning_answer_5_state(),
        core.get_learning_answer_6_state(),
        core.get_learning_answer_7_state(),
        core.get_learning_answer_8_state(),
        core.get_learning_answer_9_state(),
    ]
    .into_iter()
    .map(|state| state.as_str().eq_ignore_ascii_case("correct"))
    .collect()
}

fn check_learning_current(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    learning_passed: &Rc<RefCell<Vec<bool>>>,
) -> Result<String> {
    let node_index = normalize_node_index(core.get_learning_index());
    let mut draft = read_effective_author_draft(package_dir);
    ensure_draft_nodes(&mut draft);
    let node = draft
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.get(node_index))
        .ok_or_else(|| anyhow::anyhow!("question {} is missing", node_index + 1))?;
    let expected = node_answer_options(node)
        .iter()
        .map(|option| {
            option
                .get("isCorrect")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let actual = learning_answer_states_from_window(core);
    if expected.len() != actual.len() {
        anyhow::bail!("question {} answer count mismatch", node_index + 1);
    }
    let matched = expected.iter().zip(actual.iter()).all(|(expected, actual)| expected == actual);
    let mut passed = learning_passed.borrow_mut();
    passed[node_index] = matched;
    let passed_count = passed.iter().filter(|passed| **passed).count();
    if matched && passed_count == 24 {
        let preflight = preflight_storylock_core_package(package_dir);
        if !preflight.errors.is_empty() {
            core.set_export_ready(false);
            anyhow::bail!(
                "all learning questions passed, but package preflight failed: {}",
                preflight
                    .errors
                    .iter()
                    .map(|issue| issue.code)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        core.set_export_ready(true);
        return Ok("Learning complete: 24 / 24 questions passed. Export is enabled.".to_string());
    }
    core.set_export_ready(false);
    if matched {
        Ok(format!(
            "Question {} passed. Learning progress: {} / 24. Continue until all questions pass.",
            node_index + 1,
            passed_count
        ))
    } else {
        Ok(format!(
            "Question {} did not match. Learning progress: {} / 24. Re-check the 9 correct/wrong selections.",
            node_index + 1,
            passed_count
        ))
    }
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
    let object_id = if core.get_object_id().trim().is_empty() {
        format!(
            "credential/{}/main/secret",
            sanitize_segment(core.get_provider_id().as_str())
        )
    } else {
        core.get_object_id().to_string()
    };
    let object_kind = if core.get_object_kind().trim().is_empty() {
        "secret".to_string()
    } else {
        core.get_object_kind().to_string()
    };
    let required_grid_count = core
        .get_required_correct_count()
        .parse::<u64>()
        .unwrap_or(12)
        .clamp(1, 24);
    let catalog = json!({
        "version": "1",
        "resources": [{
            "resourceId": core.get_resource_id().to_string(),
            "resourceKind": core.get_resource_kind().to_string(),
            "providerId": core.get_provider_id().to_string(),
            "displayName": core.get_display_name().to_string(),
            "bindings": [
                {
                    "role": "protected_object",
                    "objectId": object_id,
                    "objectMeta": {
                        "objectKind": object_kind,
                        "encoding": "secret",
                        "sensitivity": "secret",
                        "requiredGridCount": required_grid_count,
                        "authorizationFrequency": core.get_authorization_frequency().to_string(),
                        "secretRef": core.get_secret_reference().to_string()
                    }
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
    let pending_state = if storylock_core_pending_author_draft_path(package_dir).exists() {
        "pending temporary draft exists; export will promote it and clear .tmp"
    } else {
        "no pending temporary draft"
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
        "identity-package/\n  vault.stlk\n  package-manifest.json\n  resource-catalog.json\n  author-draft.json\n  templates/login-sites.json\n  templates/signing-actions.json\n  templates/agent-tasks.json\n\nLocal path: {}\ntemporaryDraft={pending_state}\nresources={resources}\npermissionObjects={permission_objects}\npreflight={status}\nerrors:\n{errors}\n\nHost-readable permission summary only; raw story, answers, passwords, private keys, and signingKeyBytes remain inside StoryLock Core.",
        package_dir.display()
    )
}

fn run_export_learning_test(package_dir: &Path) -> Result<String> {
    let draft = read_effective_author_draft(package_dir);
    let nodes = draft
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("author draft nodes must be an array"))?;
    if nodes.len() != 24 {
        anyhow::bail!("author draft must contain exactly 24 questions, got {}", nodes.len());
    }
    let mut total_correct = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        let question = node.get("question").and_then(Value::as_str).unwrap_or("");
        if question.trim().is_empty() {
            anyhow::bail!("question {} is empty", index + 1);
        }
        let options = node_answer_options(node);
        if options.len() != 9 {
            anyhow::bail!("question {} must contain 9 answer options", index + 1);
        }
        let correct_count = options
            .iter()
            .filter(|option| {
                option
                    .get("isCorrect")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .count();
        if correct_count == 0 {
            anyhow::bail!("question {} must contain at least one correct answer", index + 1);
        }
        total_correct += correct_count;
    }
    let preflight = preflight_storylock_core_package(package_dir);
    if !preflight.errors.is_empty() {
        anyhow::bail!(
            "package preflight failed: {}",
            preflight
                .errors
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(format!(
        "Learning test passed: 24 questions, 216 answer options, {total_correct} correct options checked. Export is enabled."
    ))
}

fn export_storylock_package(package_dir: &Path) -> Result<std::path::PathBuf> {
    let preflight = preflight_storylock_core_package(package_dir);
    if !preflight.errors.is_empty() {
        anyhow::bail!(
            "package preflight failed: {}",
            preflight
                .errors
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    promote_pending_author_draft(package_dir)?;
    let Some(parent) = package_dir.parent() else {
        anyhow::bail!("StoryLock Core package directory has no parent");
    };
    let export_dir = parent.join("storylock-managed-key-package");
    if export_dir.exists() {
        fs::remove_dir_all(&export_dir)?;
    }
    copy_dir_recursive(package_dir, &export_dir)?;
    fs::write(
        export_dir.join("EXPORT_STATUS.txt"),
        format!(
            "Exported from StoryLock Core after learning test.\nSource: {}\nExportedAt: {}\nTemporaryDraftCleared: true\n",
            package_dir.display(),
            ui_now_timestamp()
        ),
    )?;
    remove_pending_author_draft(package_dir)?;
    Ok(export_dir)
}

fn promote_pending_author_draft(package_dir: &Path) -> Result<()> {
    let pending_path = storylock_core_pending_author_draft_path(package_dir);
    if pending_path.exists() {
        let draft = read_json_or_default(&pending_path, default_author_draft_json());
        fs::write(
            storylock_core_author_draft_path(package_dir),
            serde_json::to_vec_pretty(&draft)?,
        )?;
    }
    Ok(())
}

fn remove_pending_author_draft(package_dir: &Path) -> Result<()> {
    let pending_path = storylock_core_pending_author_draft_path(package_dir);
    if pending_path.exists() {
        fs::remove_file(&pending_path)?;
    }
    if let Some(parent) = pending_path.parent() {
        if parent.exists() && fs::read_dir(parent)?.next().is_none() {
            fs::remove_dir(parent)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        if source_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == ".tmp")
        {
            continue;
        }
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
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

    let draft = read_effective_author_draft(package_dir);
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
            let required_grid_count = meta
                .get("requiredGridCount")
                .and_then(Value::as_u64)
                .unwrap_or_else(|| permission_required_grid_count(sensitivity) as u64);
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
                "requiredGridCount": required_grid_count
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
    return default_shouzhudaitu_author_draft_json();
}

fn default_shouzhudaitu_author_draft_json() -> Value {
    let questions = vec![
        ("time", "故事发生在什么季节？", ["春天", "春耕时节", "万物生长的时候"], ["秋天", "冬天", "雨夜", "腊月", "盛夏", "元宵"]),
        ("place", "宋人主要在哪里劳作？", ["田地里", "庄稼地旁", "自己耕作的田间"], ["集市", "河边", "山洞", "城门", "书院", "码头"]),
        ("person", "故事中的主要人物是谁？", ["宋国农夫", "种田人", "守株的人"], ["渔夫", "樵夫", "商人", "将军", "书生", "猎户"]),
        ("object", "兔子撞到了什么地方？", ["树桩", "田边的树桩", "断树根"], ["石碑", "井沿", "木门", "车轮", "竹篱", "桥柱"]),
        ("event", "兔子为什么被农夫得到？", ["撞树而死", "奔跑时撞上树桩", "意外撞死"], ["被网捕住", "被箭射中", "自己睡着", "掉进井里", "被狗追到家", "被雨淋倒"]),
        ("reaction", "农夫看到兔子后的反应是什么？", ["非常高兴", "觉得捡到便宜", "以为好运会再来"], ["立刻继续耕田", "把树砍掉", "马上搬家", "责怪邻居", "放走兔子", "写信报官"]),
        ("choice", "农夫后来做出了什么选择？", ["放下农具", "不再认真耕作", "守在树桩旁"], ["扩大田地", "学习打猎", "卖掉农具", "修水渠", "去城里经商", "种更多庄稼"]),
        ("goal", "农夫守着树桩想等什么？", ["再有兔子撞来", "再次白得兔子", "重复上次的好运"], ["等人买田", "等雨停", "等官府赏钱", "等树结果", "等牛回来", "等邻居道歉"]),
        ("result", "农夫的田地后来怎样了？", ["田地荒芜", "庄稼长不好", "农事被耽误"], ["粮食大丰收", "田地变成花园", "长出金子", "变成池塘", "被别人偷走", "马上卖高价"]),
        ("lesson", "这个故事主要讽刺什么？", ["侥幸心理", "不劳而获", "死守偶然经验"], ["勤劳致富", "诚实守信", "尊老爱幼", "团结合作", "知错能改", "乐于助人"]),
        ("logic", "兔子撞树这件事在故事里属于什么？", ["偶然事件", "不可重复的巧合", "意外收获"], ["日常规律", "农夫计划", "官府安排", "自然法则", "交易结果", "长期经验"]),
        ("contrast", "农夫应该依靠什么获得收成？", ["耕作", "持续劳动", "按时种田"], ["守树桩", "等兔子", "占卜", "睡觉", "赶集", "听传闻"]),
        ("symbol", "树桩在故事中象征什么？", ["偶然机会", "死守的经验", "错误依赖"], ["丰收秘诀", "官府权力", "家族荣耀", "市场价格", "远方道路", "善良品质"]),
        ("risk", "农夫把一次偶然当成规律会导致什么？", ["耽误生产", "失去收成", "越来越被动"], ["马上发财", "得到更多田", "学会医术", "成为官员", "找到宝藏", "获得名声"]),
        ("memory", "故事中最关键的画面是什么？", ["兔子撞树", "农夫守株", "田地荒芜"], ["老人钓鱼", "孩子读书", "商人赶路", "将军练兵", "船夫摆渡", "木匠造屋"]),
        ("order", "事件顺序应当怎样理解？", ["耕田时见兔", "捡到兔子", "放下农具守株"], ["先卖兔子", "先砍树", "先搬家", "先捕鱼", "先下雪", "先修桥"]),
        ("identity", "故事里的人为什么被称为宋人？", ["来自宋国", "宋国的农夫", "寓言中的宋国人"], ["姓宋", "住在宋山", "会写宋体字", "来自楚国", "来自齐国", "官名叫宋"]),
        ("action", "守株这个动作具体指什么？", ["守着树桩", "等待兔子再撞来", "停止劳动等好运"], ["守城门", "守仓库", "守桥头", "保护树苗", "看守羊群", "守夜巡逻"]),
        ("failure", "农夫失败的根本原因是什么？", ["误判偶然", "放弃劳动", "把巧合当方法"], ["不会识字", "天气太冷", "田太小", "兔子太多", "邻居阻拦", "工具太新"]),
        ("training", "学习这个故事时最应该记住哪三点？", ["兔子撞树", "农夫守株", "田地荒芜"], ["金斧银斧", "狼来了", "刻舟求剑", "井底之蛙", "狐假虎威", "画蛇添足"]),
        ("policy", "用于授权训练时，这个故事适合作为什么提示？", ["反侥幸提示", "坚持主动验证", "不要依赖偶然"], ["公开密码", "远程私钥", "自动放行", "无需确认", "删除题库", "跳过训练"]),
        ("review", "如果用户只记得兔子撞树，还需要补充记住什么？", ["农夫停止耕作", "守着树桩等待", "最后田地荒芜"], ["兔子会说话", "农夫成了国王", "树桩开花", "田里有井", "邻居送粮", "天上下金"]),
        ("export", "导出前学习训练要确认什么？", ["24个问题已配置", "每题9个候选答案", "正确错误已标记"], ["直接暴露私钥", "跳过本地确认", "上传答案原文", "删除故事", "关闭题库", "开放远程写入"]),
        ("ending", "守株待兔最终告诉我们什么？", ["不能坐等侥幸", "要靠持续行动", "不能把偶然当规律"], ["等着就会成功", "兔子每天会来", "树桩能带来财富", "田地不用管理", "好运一定重复", "农具没有用"]),
    ];
    let nodes = questions
        .into_iter()
        .enumerate()
        .map(|(offset, (element_id, question, correct, wrong))| {
            let index = offset + 1;
            let answer_options = correct
                .into_iter()
                .map(|text| json!({ "text": text, "isCorrect": true }))
                .chain(wrong.into_iter().map(|text| json!({ "text": text, "isCorrect": false })))
                .collect::<Vec<_>>();
            json!({
                "nodeId": format!("node-{index:02}"),
                "title": format!("守株待兔问题 {index:02}"),
                "elementId": element_id,
                "question": question,
                "recommendedSelectionMode": "multi_select",
                "recommendedCorrectCount": 3,
                "candidatePoolSize": 9,
                "recallPriority": "high",
                "verifyPolicy": "caseInsensitive + trim",
                "editorNotes": "守株待兔默认模板，仅保存在 StoryLock Core 本地草稿中。",
                "canonicalAnswerLocalOnly": correct[0],
                "acceptedAnswersLocalOnly": correct,
                "answerOptionsLocalOnly": answer_options
            })
        })
        .collect::<Vec<_>>();
    json!({
        "version": "1",
        "storyTitle": "守株待兔",
        "summary": "宋国有个农夫在田里耕作时，看见一只兔子奔跑撞到树桩上死了。农夫因此放下农具，天天守着树桩等待下一只兔子，结果兔子没有再来，田地也荒芜了。",
        "memoryAnchors": ["宋国农夫", "田地", "兔子撞树", "树桩", "放下农具", "田地荒芜"],
        "elementGroups": ["时间", "地点", "人物", "物件", "事件", "反应", "选择", "结果"],
        "nodes": nodes
    })
}

#[allow(dead_code)]
fn legacy_default_author_draft_json() -> Value {
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
                "editorNotes": "Local author draft only.",
                "answerOptionsLocalOnly": default_answer_options(index)
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

fn answer_options_from_window(core: &StoryLockCoreApp) -> Vec<Value> {
    [
        (core.get_answer_1(), core.get_answer_1_state()),
        (core.get_answer_2(), core.get_answer_2_state()),
        (core.get_answer_3(), core.get_answer_3_state()),
        (core.get_answer_4(), core.get_answer_4_state()),
        (core.get_answer_5(), core.get_answer_5_state()),
        (core.get_answer_6(), core.get_answer_6_state()),
        (core.get_answer_7(), core.get_answer_7_state()),
        (core.get_answer_8(), core.get_answer_8_state()),
        (core.get_answer_9(), core.get_answer_9_state()),
    ]
    .into_iter()
    .map(|(text, state)| {
        json!({
            "text": text.to_string(),
            "isCorrect": state.as_str().eq_ignore_ascii_case("correct")
        })
    })
    .collect()
}

fn set_answer_options_into_window(core: &StoryLockCoreApp, options: &[Value]) {
    let get = |index: usize| -> (SharedString, SharedString) {
        let option = options.get(index).unwrap_or(&Value::Null);
        let text = option.get("text").and_then(Value::as_str).unwrap_or("");
        let state = if option
            .get("isCorrect")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            "correct"
        } else {
            "wrong"
        };
        (SharedString::from(text), SharedString::from(state))
    };
    let (text, state) = get(0);
    core.set_answer_1(text);
    core.set_answer_1_state(state);
    let (text, state) = get(1);
    core.set_answer_2(text);
    core.set_answer_2_state(state);
    let (text, state) = get(2);
    core.set_answer_3(text);
    core.set_answer_3_state(state);
    let (text, state) = get(3);
    core.set_answer_4(text);
    core.set_answer_4_state(state);
    let (text, state) = get(4);
    core.set_answer_5(text);
    core.set_answer_5_state(state);
    let (text, state) = get(5);
    core.set_answer_6(text);
    core.set_answer_6_state(state);
    let (text, state) = get(6);
    core.set_answer_7(text);
    core.set_answer_7_state(state);
    let (text, state) = get(7);
    core.set_answer_8(text);
    core.set_answer_8_state(state);
    let (text, state) = get(8);
    core.set_answer_9(text);
    core.set_answer_9_state(state);
}

fn node_answer_options(node: &Value) -> Vec<Value> {
    if let Some(options) = node.get("answerOptionsLocalOnly").and_then(Value::as_array) {
        let mut normalized = options
            .iter()
            .take(9)
            .map(|option| {
                json!({
                    "text": option.get("text").and_then(Value::as_str).unwrap_or(""),
                    "isCorrect": option.get("isCorrect").and_then(Value::as_bool).unwrap_or(false)
                })
            })
            .collect::<Vec<_>>();
        while normalized.len() < 9 {
            let index = normalized.len() + 1;
            normalized.push(json!({ "text": format!("候选答案 {index}"), "isCorrect": false }));
        }
        return normalized;
    }
    let correct = node
        .get("correctOptionsLocalOnly")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let distractors = node
        .get("distractorsLocalOnly")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    correct
        .into_iter()
        .map(|item| json!({ "text": item.as_str().unwrap_or(""), "isCorrect": true }))
        .chain(
            distractors
                .into_iter()
                .map(|item| json!({ "text": item.as_str().unwrap_or(""), "isCorrect": false })),
        )
        .chain((1..=9).map(|index| json!({ "text": format!("候选答案 {index}"), "isCorrect": false })))
        .take(9)
        .collect()
}

fn format_answer_options(options: &[Value]) -> String {
    options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let text = option.get("text").and_then(Value::as_str).unwrap_or("");
            let marker = if option
                .get("isCorrect")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                "correct"
            } else {
                "wrong"
            };
            format!("{}. {} | {}", index + 1, text, marker)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_correct_option_indexes(options: &[Value]) -> String {
    options
        .iter()
        .enumerate()
        .filter_map(|(index, option)| {
            option
                .get("isCorrect")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                .then(|| (index + 1).to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
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

fn default_answer_options(index: usize) -> Vec<Value> {
    (1..=9)
        .map(|option_index| {
            json!({
                "text": format!("Node {index:02} answer option {option_index}"),
                "isCorrect": option_index <= 3
            })
        })
        .collect()
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
    fn effective_author_draft_prefers_pending_temp_file() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let mut pending = read_json_or_default(
            &storylock_core_author_draft_path(&dir),
            default_author_draft_json(),
        );
        pending["storyTitle"] = json!("pending temp title");
        write_pending_author_draft(&dir, &pending).expect("write pending draft");
        let effective = read_effective_author_draft(&dir);
        assert_eq!(
            effective.get("storyTitle").and_then(Value::as_str),
            Some("pending temp title")
        );
    }

    #[test]
    fn export_promotes_and_clears_pending_temp_draft() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let mut pending = read_json_or_default(
            &storylock_core_author_draft_path(&dir),
            default_author_draft_json(),
        );
        pending["storyTitle"] = json!("promoted title");
        write_pending_author_draft(&dir, &pending).expect("write pending draft");

        let export_dir = export_storylock_package(&dir).expect("export package");
        assert!(!storylock_core_pending_author_draft_path(&dir).exists());
        assert!(!export_dir.join(".tmp").exists());
        let promoted = read_json_or_default(
            &storylock_core_author_draft_path(&dir),
            default_author_draft_json(),
        );
        assert_eq!(
            promoted.get("storyTitle").and_then(Value::as_str),
            Some("promoted title")
        );
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
        fake_core.set_answer_options(SharedString::from(
            "1. A | correct\n2. B | correct\n3. C | correct\n4. D | wrong\n5. E | wrong\n6. F | wrong\n7. G | wrong\n8. H | wrong\n9. I | wrong",
        ));
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
