use crate::WindowsHostConfig;
use crate::ProtectedEnvelope;
use crate::dpapi_protect_to_base64;
use crate::dpapi_unprotect_from_base64;
use anyhow::Result;
use reqwest::blocking::Client;
use serde_json::json;
use serde_json::Value;
use slint::SharedString;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

slint::slint! {
    import { Button, ComboBox, LineEdit, ScrollView, TextEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";

    component MenuButton inherits Rectangle {
        in property <string> label;
        in property <bool> selected;
        in property <bool> enabled: true;
        callback clicked();

        width: 144px;
        height: 36px;
        border-radius: 6px;
        background: !enabled ? #d8e0e4 : (selected ? #b8c8cd : #e2e9ec);
        TouchArea {
            clicked => {
                if root.enabled {
                    root.clicked();
                }
            }
        }
        Rectangle {
            x: 0px;
            y: 0px;
            width: root.enabled && root.selected ? 4px : 0px;
            height: parent.height;
            border-radius: 4px;
            background: #45606b;
        }
        Text {
            text: root.label;
            color: enabled ? #17252f : #8a98a0;
            font-size: 14px;
            font-weight: selected ? 700 : 500;
            vertical-alignment: center;
            x: 10px;
            width: parent.width - 20px;
            height: parent.height;
        }
    }

    component SubMenuButton inherits Rectangle {
        in property <string> label;
        in property <bool> selected;
        callback clicked();

        width: 124px;
        height: 32px;
        background: transparent;

        Rectangle {
            x: 10px;
            y: -8px;
            width: 1px;
            height: 48px;
            background: #9fb0b8;
        }
        Rectangle {
            x: 10px;
            y: 16px;
            width: 12px;
            height: 1px;
            background: #9fb0b8;
        }
        Rectangle {
            x: 22px;
            y: 0px;
            width: 102px;
            height: 32px;
            border-radius: 6px;
            background: selected ? #b8c8cd : #e2e9ec;
            Rectangle {
                x: 0px;
                y: 0px;
                width: root.selected ? 4px : 0px;
                height: parent.height;
                border-radius: 4px;
                background: #45606b;
            }
            Text {
                text: root.label;
                color: #17252f;
                font-size: 12px;
                font-weight: selected ? 700 : 500;
                vertical-alignment: center;
                x: 8px;
                width: parent.width - 16px;
                height: parent.height;
                overflow: elide;
            }
            TouchArea {
                clicked => { root.clicked(); }
            }
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
            width: 612px;
            height: 32px;
        }
    }

    component PathBrowseRow inherits HorizontalLayout {
        in property <string> name;
        in-out property <string> value;
        in property <bool> is-zh: true;
        callback browse();

        spacing: 10px;
        Text {
            text: root.name;
            width: 150px;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        LineEdit {
            text <=> root.value;
            width: 450px;
            height: 32px;
        }
        Rectangle {
            width: 74px;
            height: 32px;
            border-radius: 6px;
            background: #d7e1e6;
            TouchArea {
                clicked => { root.browse(); }
            }
            Text {
                text: root.is-zh ? "浏览" : "Browse";
                color: #17252f;
                font-size: 13px;
                font-weight: 700;
                horizontal-alignment: center;
                vertical-alignment: center;
                width: parent.width;
                height: parent.height;
            }
        }
    }

    component LargeEditableText inherits Rectangle {
        in property <string> name;
        in-out property <string> value;

        width: 720px;
        height: 250px;
        background: #eef3f5;
        Text {
            x: 0px;
            y: 0px;
            width: 150px;
            height: 32px;
            text: root.name;
            color: #62727d;
            font-size: 13px;
            vertical-alignment: center;
        }
        Rectangle {
            x: 164px;
            y: 0px;
            width: 556px;
            height: 250px;
            border-radius: 6px;
            background: #eef3f5;
            TextEdit {
                x: 0px;
                y: 0px;
                width: parent.width;
                height: parent.height;
                text <=> root.value;
            }
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
        if root.show-state: Rectangle {
            x: 540px;
            y: 0px;
            width: 130px;
            height: 32px;
            border-width: 1px;
            border-color: #d7e1e6;
            border-radius: 4px;
            background: #f7fafb;
            TouchArea {
                clicked => {
                    root.state = root.state == "correct" ? "wrong" : "correct";
                }
            }
            HorizontalBox {
                x: 8px;
                y: 0px;
                width: parent.width - 20px;
                height: parent.height;
                spacing: 8px;
                Rectangle {
                    width: 20px;
                    height: 20px;
                    y: 6px;
                    border-width: 0px;
                    background: transparent;
                    Text {
                        text: root.state == "correct" ? "✓" : "×";
                        width: parent.width;
                        height: parent.height;
                        color: root.state == "correct" ? #246b3d : #8d3333;
                        font-family: "Segoe UI Symbol";
                        font-size: 15px;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }
                Text {
                    text: root.state == "correct" ? "正确" : "错误";
                    color: #17252f;
                    font-size: 13px;
                    vertical-alignment: center;
                }
            }
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

    component QuestionTile inherits Rectangle {
        in property <string> label;
        in property <bool> selected;
        callback clicked();

        width: 171px;
        height: 56px;
        border-width: 1px;
        border-color: selected ? #45606b : #d7e1e6;
        border-radius: 6px;
        background: selected ? #dbe8eb : #edf3f5;
        TouchArea {
            clicked => { root.clicked(); }
        }
        Text {
            x: 10px;
            y: 8px;
            width: parent.width - 20px;
            height: parent.height - 16px;
            text: root.label;
            color: #17252f;
            font-size: 12px;
            font-weight: selected ? 700 : 500;
            wrap: word-wrap;
            overflow: elide;
        }
    }

    component QuestionOverviewGrid inherits Rectangle {
        in-out property <string> selected-question;
        in property <string> question-1;
        in property <string> question-2;
        in property <string> question-3;
        in property <string> question-4;
        in property <string> question-5;
        in property <string> question-6;
        in property <string> question-7;
        in property <string> question-8;
        in property <string> question-9;
        in property <string> question-10;
        in property <string> question-11;
        in property <string> question-12;
        in property <string> question-13;
        in property <string> question-14;
        in property <string> question-15;
        in property <string> question-16;
        in property <string> question-17;
        in property <string> question-18;
        in property <string> question-19;
        in property <string> question-20;
        in property <string> question-21;
        in property <string> question-22;
        in property <string> question-23;
        in property <string> question-24;
        callback select-node(string);

        width: 720px;
        height: 374px;
        background: transparent;
        GridLayout {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            spacing: 8px;
            Row {
                QuestionTile { label: root.question-1; selected: root.selected-question == "1"; clicked => { root.select-node("1"); } }
                QuestionTile { label: root.question-2; selected: root.selected-question == "2"; clicked => { root.select-node("2"); } }
                QuestionTile { label: root.question-3; selected: root.selected-question == "3"; clicked => { root.select-node("3"); } }
                QuestionTile { label: root.question-4; selected: root.selected-question == "4"; clicked => { root.select-node("4"); } }
            }
            Row {
                QuestionTile { label: root.question-5; selected: root.selected-question == "5"; clicked => { root.select-node("5"); } }
                QuestionTile { label: root.question-6; selected: root.selected-question == "6"; clicked => { root.select-node("6"); } }
                QuestionTile { label: root.question-7; selected: root.selected-question == "7"; clicked => { root.select-node("7"); } }
                QuestionTile { label: root.question-8; selected: root.selected-question == "8"; clicked => { root.select-node("8"); } }
            }
            Row {
                QuestionTile { label: root.question-9; selected: root.selected-question == "9"; clicked => { root.select-node("9"); } }
                QuestionTile { label: root.question-10; selected: root.selected-question == "10"; clicked => { root.select-node("10"); } }
                QuestionTile { label: root.question-11; selected: root.selected-question == "11"; clicked => { root.select-node("11"); } }
                QuestionTile { label: root.question-12; selected: root.selected-question == "12"; clicked => { root.select-node("12"); } }
            }
            Row {
                QuestionTile { label: root.question-13; selected: root.selected-question == "13"; clicked => { root.select-node("13"); } }
                QuestionTile { label: root.question-14; selected: root.selected-question == "14"; clicked => { root.select-node("14"); } }
                QuestionTile { label: root.question-15; selected: root.selected-question == "15"; clicked => { root.select-node("15"); } }
                QuestionTile { label: root.question-16; selected: root.selected-question == "16"; clicked => { root.select-node("16"); } }
            }
            Row {
                QuestionTile { label: root.question-17; selected: root.selected-question == "17"; clicked => { root.select-node("17"); } }
                QuestionTile { label: root.question-18; selected: root.selected-question == "18"; clicked => { root.select-node("18"); } }
                QuestionTile { label: root.question-19; selected: root.selected-question == "19"; clicked => { root.select-node("19"); } }
                QuestionTile { label: root.question-20; selected: root.selected-question == "20"; clicked => { root.select-node("20"); } }
            }
            Row {
                QuestionTile { label: root.question-21; selected: root.selected-question == "21"; clicked => { root.select-node("21"); } }
                QuestionTile { label: root.question-22; selected: root.selected-question == "22"; clicked => { root.select-node("22"); } }
                QuestionTile { label: root.question-23; selected: root.selected-question == "23"; clicked => { root.select-node("23"); } }
                QuestionTile { label: root.question-24; selected: root.selected-question == "24"; clicked => { root.select-node("24"); } }
            }
        }
    }

    component QuestionConfigPanel inherits Rectangle {
        in property <length> panel-height: 374px;
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
        height: root.panel-height;
        border-radius: 6px;
        border-width: 0px;
        background: #eef3f5;

        ScrollView {
            x: 10px;
            y: 0px;
            width: 720px;
            height: root.panel-height;
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
        in property <length> button-width: 150px;
        callback clicked();

        width: root.button-width;
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

    component SettingsIconButton inherits Rectangle {
        in property <bool> selected;
        callback clicked();

        width: 32px;
        height: 32px;
        border-radius: 6px;
        border-width: 1px;
        border-color: selected ? #45606b : #c9d5da;
        background: selected ? #d7e1e6 : #f7fafb;
        TouchArea {
            clicked => { root.clicked(); }
        }
        Text {
            text: "⚙";
            width: parent.width;
            height: parent.height;
            color: #17252f;
            font-size: 18px;
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
        in-out property <int> active-page: 1;
        in-out property <string> language: "zh";
        property <bool> is-zh: language == "zh";
        in-out property <bool> settings-open: false;
        in-out property <string> nine-grid-status: "九宫格测试尚未开始";
        in-out property <string> nine-grid-summary: "对象: 未选择";
        in property <string> product;
        in property <string> version;
        in property <string> mode;
        in property <string> identity-id;
        in property <string> device-id;
        in property <string> local-api;
        in property <string> capabilities;
        in property <string> call-chain;
        in property <string> management-stats;
        in property <string> diagnostics;
        in-out property <string> connection-test-status: "No connection test has run yet.";
        property <string> core-launch-status: "StoryLock: closed | launch: none | language: zh";
        property <string> current-title: active-page == 0 ? (is-zh ? "状态" : "Status") : active-page == 1 ? (is-zh ? "本地主机" : "Local Host") : active-page == 2 ? (is-zh ? "管理" : "Management") : (is-zh ? "诊断" : "Diagnostics");
        callback close-requested();
        callback open-storylock-core();
        callback open-settings();
        callback test-local-host() -> string;
        callback test-remote-connection() -> string;
        callback test-managed-object-nine-grid(string) -> string;

        title: "Yian Windows Host";
        preferred-width: 960px;
        preferred-height: 540px;
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
                    y: 48px;
                    width: 144px;
                    height: 330px;
                    spacing: 14px;

                    MenuButton {
                        label: root.is-zh ? "状态" : "Status";
                        selected: root.active-page == 0;
                        clicked => { root.active-page = 0; }
                    }
                    MenuButton {
                        label: root.is-zh ? "本地主机" : "Local Host";
                        selected: root.active-page == 1;
                        clicked => { root.active-page = 1; }
                    }
                    MenuButton {
                        label: root.is-zh ? "管理" : "Management";
                        selected: root.active-page == 2;
                        clicked => { root.active-page = 2; }
                    }
                    MenuButton {
                        label: root.is-zh ? "诊断" : "Diagnostics";
                        selected: root.active-page == 3;
                        clicked => { root.active-page = 3; }
                    }
                    MenuButton {
                        label: root.is-zh ? "打开 StoryLock 配置" : "Open StoryLock";
                        selected: false;
                        clicked => { root.open-storylock-core(); }
                    }
                }
            }

            Rectangle {
                min-width: 640px;
                background: #eef3f5;
                Rectangle {
                    width: 640px;
                    height: 540px;
                    background: transparent;
                    VerticalBox {
                        padding: 38px;
                        spacing: 16px;

                        Rectangle {
                            height: 48px;
                            background: transparent;
                            Image {
                                x: 0px;
                                y: 12px;
                                source: @image-url("assets/lock.png");
                                width: 28px;
                                height: 28px;
                            }
                            Text {
                                x: 38px;
                                y: 12px;
                                width: 480px;
                                height: 28px;
                                text: "Yian: StoryLock - " + root.current-title;
                                font-size: 16px;
                                font-weight: 800;
                                color: #17252f;
                                overflow: elide;
                            }
                            SettingsIconButton {
                                x: 556px;
                                y: 2px;
                                selected: false;
                                clicked => { root.open-settings(); }
                            }
                        }

                        Rectangle {
                            height: 1px;
                            background: #d7e1e6;
                        }

                        Rectangle {
                            width: 600px;
                            height: 360px;
                            background: transparent;

                            if root.active-page == 0: VerticalBox {
                                x: 0px;
                                y: 0px;
                                width: 600px;
                                height: 360px;
                                spacing: 12px;
                                FormRow { name: root.is-zh ? "身份" : "Identity"; value: root.identity-id; }
                                FormRow { name: root.is-zh ? "设备" : "Device"; value: root.device-id; }
                                FormRow { name: root.is-zh ? "本地 API" : "Local API"; value: root.local-api; }
                                StaticRow { name: root.is-zh ? "模式" : "Mode"; value: root.mode; }
                                StaticRow { name: root.is-zh ? "连接测试" : "Connection Test"; value: root.connection-test-status; }
                                HorizontalBox {
                                    spacing: 12px;
                                    ActionButton {
                                        label: root.is-zh ? "测试本地主机" : "Test Local Host";
                                        primary: true;
                                        clicked => {
                                            root.connection-test-status = root.test-local-host();
                                        }
                                    }
                                    ActionButton {
                                        label: root.is-zh ? "测试远程连接" : "Test Remote";
                                        primary: false;
                                        clicked => {
                                            root.connection-test-status = root.test-remote-connection();
                                        }
                                    }
                                }
                            }

                            if root.active-page == 1: VerticalBox {
                                x: 0px;
                                y: 0px;
                                width: 600px;
                                height: 360px;
                                spacing: 12px;
                                FormRow { name: root.is-zh ? "能力" : "Capabilities"; value: root.capabilities; }
                                FormRow { name: root.is-zh ? "调用链" : "Call Chain"; value: root.call-chain; }
                                StaticRow { name: root.is-zh ? "边界" : "Boundary"; value: root.is-zh ? "仅 Relay、本地 API 与授权代理" : "Relay, localhost API, and approval broker only"; }
                                StaticRow { name: root.is-zh ? "远程访问" : "Remote Access"; value: root.is-zh ? "默认关闭" : "Disabled by default"; }
                                StaticRow { name: root.is-zh ? "StoryLock 数据" : "StoryLock Data"; value: root.is-zh ? "Yian Host 不可读取" : "Not readable from Yian Host"; }
                            }

                            if root.active-page == 2: VerticalBox {
                                x: 0px;
                                y: 0px;
                                width: 600px;
                                height: 360px;
                                spacing: 12px;
                                StaticRow { name: root.is-zh ? "单项读取" : "Single Read"; value: root.is-zh ? "解锁 9 格中的 6 格，中等强度，允许远程" : "6 of 9 cells, medium strength, remote allowed"; }
                                StaticRow { name: root.is-zh ? "批量读取" : "Batch Read"; value: root.is-zh ? "解锁 12 格中的 12 格，高强度，允许远程" : "12 of 12 cells, high strength, remote allowed"; }
                                StaticRow { name: root.is-zh ? "故事编辑" : "Story Edit"; value: root.is-zh ? "解锁 24 格中的 22 格，仅本地 StoryLock UI" : "22 of 24 cells, local StoryLock UI only"; }
                                StaticRow { name: root.is-zh ? "九宫格测试" : "Nine-Grid Test"; value: root.nine-grid-status; }
                                StaticRow { name: root.is-zh ? "测试对象" : "Test Object"; value: root.nine-grid-summary; }
                                HorizontalBox {
                                    spacing: 8px;
                                    ActionButton {
                                        label: root.is-zh ? "6格普通对象" : "6-cell Normal";
                                        primary: true;
                                        button-width: 128px;
                                        clicked => {
                                            root.nine-grid-status = root.test-managed-object-nine-grid("normal");
                                            root.nine-grid-summary = root.nine-grid-status;
                                        }
                                    }
                                    ActionButton {
                                        label: root.is-zh ? "12格保密对象" : "12-cell Confidential";
                                        primary: false;
                                        button-width: 136px;
                                        clicked => {
                                            root.nine-grid-status = root.test-managed-object-nine-grid("confidential");
                                            root.nine-grid-summary = root.nine-grid-status;
                                        }
                                    }
                                    ActionButton {
                                        label: root.is-zh ? "22格高机密对象" : "22-cell Top Secret";
                                        primary: false;
                                        button-width: 136px;
                                        clicked => {
                                            root.nine-grid-status = root.test-managed-object-nine-grid("top-secret");
                                            root.nine-grid-summary = root.nine-grid-status;
                                        }
                                    }
                                }
                                LogPanel { value: root.management-stats; panel-height: 150px; }
                            }

                            if root.active-page == 3: VerticalBox {
                                x: 0px;
                                y: 0px;
                                width: 600px;
                                height: 360px;
                                spacing: 12px;
                                LogPanel { value: root.diagnostics; }
                            }
                        }
                    }
                    if root.settings-open: Rectangle {
                        width: 640px;
                        height: 540px;
                        background: #00000022;
                        TouchArea { }
                    }
                }
            }
        }
    }

    export component SettingsDialog inherits Window {
        in-out property <string> language: "zh";
        in-out property <string> core-launch-status: "StoryLock: closed | launch: none | language: zh";
        property <bool> is-zh: language == "zh";
        callback close-requested();
        callback open-storylock-core();
        callback language-changed(string);

        title: is-zh ? "设置" : "Settings";
        preferred-width: 640px;
        preferred-height: 360px;
        background: #eef3f5;

        VerticalBox {
            padding: 24px;
            spacing: 16px;
            Rectangle {
                height: 36px;
                background: transparent;
                Text {
                    x: 0px;
                    y: 4px;
                    width: 400px;
                    height: 28px;
                    text: is-zh ? "设置" : "Settings";
                    font-size: 16px;
                    font-weight: 800;
                    color: #17252f;
                    overflow: elide;
                }
            }
                HorizontalBox {
                    spacing: 14px;
                    Text {
                        text: is-zh ? "界面语言" : "UI Language";
                        width: 140px;
                    color: #62727d;
                    font-size: 13px;
                    vertical-alignment: center;
                }
                ComboBox {
                    width: 294px;
                    height: 32px;
                    model: ["中文", "English"];
                    current-value: is-zh ? "中文" : "English";
                    selected(value) => {
                        root.language = value == "中文" ? "zh" : "en";
                        root.language-changed(value == "中文" ? "zh" : "en");
                    }
                }
                    ActionButton {
                        label: is-zh ? "打开" : "Open";
                        primary: true;
                        button-width: 112px;
                        clicked => { root.open-storylock-core(); }
                    }
                }
            StaticRow { name: is-zh ? "StoryLock 状态" : "StoryLock Status"; value: root.core-launch-status; }
            StaticRow { name: is-zh ? "触发方式" : "Trigger"; value: is-zh ? "只能从设置弹窗手动打开" : "Manual launch from the Settings dialog only"; }
            StaticRow { name: is-zh ? "说明" : "Note"; value: is-zh ? "语言切换会立即作用于主界面与 StoryLock Core。" : "Language changes apply to the Host UI and StoryLock Core immediately."; }
        }
    }

    export component StoryLockCoreApp inherits Window {
        in-out property <int> active-page: 1;
        in-out property <string> language: "zh";
        property <bool> is-zh: language == "zh";
        in-out property <string> story-title: "守株待兔";
        in-out property <string> story-summary: "一名农夫在田里偶然撞见兔子撞树而死，从此放弃耕作，天天守在树桩旁等待下一只兔子。";
        in-out property <string> story-plot: "宋国有个农夫，正在田里劳作。忽然一只兔子慌慌张张地奔跑，撞上田边的树桩死了。农夫捡到兔子后，觉得不必再辛苦耕作，只要守着树桩就能得到兔子。于是他把锄头丢在一边，天天坐在树桩旁等待，结果田地荒芜，始终没有再等到兔子。这个故事用来演示如何把 24 个问题串成一个可以反复回忆的本地故事模板。";
        in-out property <string> memory-anchors: "spring / station / blue cup / recorder card / departure bell";
        in-out property <string> element-group: "time,place,person,object,event,reaction,choice,result";
        in-out property <int> node-index: 0;
        in-out property <string> node-position: "1 / 24";
        in-out property <string> node-id: "node-01";
        in-out property <string> node-title: "Question 01";
        in-out property <string> element-id: "time";
        in-out property <string> selected-question: "1";
        in-out property <string> question-text: "Which season appears in the memory story?";
        in-out property <string> question-1: "Q1";
        in-out property <string> question-2: "Q2";
        in-out property <string> question-3: "Q3";
        in-out property <string> question-4: "Q4";
        in-out property <string> question-5: "Q5";
        in-out property <string> question-6: "Q6";
        in-out property <string> question-7: "Q7";
        in-out property <string> question-8: "Q8";
        in-out property <string> question-9: "Q9";
        in-out property <string> question-10: "Q10";
        in-out property <string> question-11: "Q11";
        in-out property <string> question-12: "Q12";
        in-out property <string> question-13: "Q13";
        in-out property <string> question-14: "Q14";
        in-out property <string> question-15: "Q15";
        in-out property <string> question-16: "Q16";
        in-out property <string> question-17: "Q17";
        in-out property <string> question-18: "Q18";
        in-out property <string> question-19: "Q19";
        in-out property <string> question-20: "Q20";
        in-out property <string> question-21: "Q21";
        in-out property <string> question-22: "Q22";
        in-out property <string> question-23: "Q23";
        in-out property <string> question-24: "Q24";
        in-out property <string> canonical-answer: "spring";
        in-out property <string> accepted-answers: "spring; rainy spring";
        in-out property <string> answer-options: "1. spring | correct\n2. rainy spring | correct\n3. departure bell | correct\n4. winter | wrong\n5. noon | wrong\n6. harbor | wrong\n7. red bag | wrong\n8. locker 3 | wrong\n9. radio tower | wrong";
        in-out property <string> correct-options: "1,2,3";
        in-out property <string> answer-1: "spring";
        in-out property <string> answer-1-state: "correct";
        in-out property <string> answer-2: "rainy spring";
        in-out property <string> answer-2-state: "correct";
        in-out property <string> answer-3: "departure bell";
        in-out property <string> answer-3-state: "correct";
        in-out property <string> answer-4: "winter";
        in-out property <string> answer-4-state: "wrong";
        in-out property <string> answer-5: "noon";
        in-out property <string> answer-5-state: "wrong";
        in-out property <string> answer-6: "harbor";
        in-out property <string> answer-6-state: "wrong";
        in-out property <string> answer-7: "red bag";
        in-out property <string> answer-7-state: "wrong";
        in-out property <string> answer-8: "locker 3";
        in-out property <string> answer-8-state: "wrong";
        in-out property <string> answer-9: "radio tower";
        in-out property <string> answer-9-state: "wrong";
        in-out property <string> selection-mode: "multi_select";
        in-out property <string> correct-count: "3";
        in-out property <string> candidate-pool-size: "9";
        in-out property <string> recall-priority: "high";
        in-out property <string> verify-policy: "caseInsensitive + trim";
        in-out property <string> editor-notes: "StoryLock UI local draft only.";
        in-out property <string> node-overview: "24 question overview is loaded from the local author draft.";
        in-out property <string> node-output: "Configure 24 local questions before export.";
        in-out property <string> vault-name: "storylock-local-vault";
        in-out property <string> resource-group: "normal";
        in-out property <string> resource-id: "github-main";
        in-out property <string> resource-kind: "website_account";
        in-out property <string> provider-id: "github";
        in-out property <string> display-name: "GitHub main account";
        in-out property <string> object-id: "wallet/evm/main/signing_key";
        in-out property <string> object-kind: "private_key";
        in-out property <string> required-correct-count: "12";
        in-out property <string> authorization-frequency: "Every high-risk request";
        in-out property <string> secret-reference: "StoryLock local secret reference";
        in-out property <string> training-policy: "Complete local learning review before saving.";
        in-out property <string> pre-learning-error-tolerance: "2";
        in-out property <string> weak-item-limit: "3";
        in-out property <string> initial-days: "3";
        in-out property <string> initial-frequency-days: "1";
        in-out property <string> consolidation-days: "4";
        in-out property <string> consolidation-frequency-days: "2";
        in-out property <string> adaptation-weeks: "3";
        in-out property <string> adaptation-frequency-weeks: "1";
        in-out property <string> stable-months: "4";
        in-out property <string> stable-frequency-months: "1";
        in-out property <string> long-term-years: "1";
        in-out property <string> long-term-frequency-years: "1";
        in-out property <string> learning-plan-summary: "Pre-learning: 48 prompts, 24 questions x 2. Retention: 22 questions by phase.";
        in-out property <string> protected-object-list: "Protected objects are loaded from resource-catalog.json.";
        in-out property <string> resource-bindings: "username -> credential/github/main/username\npassword -> credential/github/main/password\ntotp_secret -> credential/github/main/totp_secret";
        in-out property <string> object-meta: "username: reference utf8\npassword: secret utf8\ntotp_secret: secret utf8";
        in-out property <string> template-kind: "vault.stlk";
        in-out property <string> template-id: "github.com";
        in-out property <string> template-display-name: "GitHub main login";
        in-out property <string> template-bindings: "vault.stlk\n  loginSites\n    username -> username\n    password -> password\n\n  signingActions\n    username -> username\n\n  agentTasks\n    username -> username";
        in-out property <string> candidate-template-status: "";
        in-out property <string> export-preview: "identity-package/\n  vault.stlk\n  package-manifest.json\n  resource-catalog.json";
        in-out property <string> config-status: "All edits stay inside StoryLock UI. Yian Host receives no draft, vault, catalog, template, or package path.";
        in-out property <string> learning-status: "Pre-export test is required before export.";
        in-out property <bool> export-ready: false;
        in-out property <int> learning-index: 0;
        in-out property <string> learning-position: "1 / 24";
        in-out property <string> learning-question: "";
        in-out property <string> learning-result: "Toggle the 9 answer states, then check current question.";
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
        in-out property <string> draft-file-path: "vault.stlk";
        in-out property <string> manifest-file-path: "package-manifest.json";
        in-out property <string> encrypted-vault-path: "vault.stlk";
        in-out property <string> resource-catalog-path: "resource-catalog.json";
        in-out property <string> learning-policy-path: "learning-policy.json";
        in-out property <string> export-package-dir: "";
        in-out property <string> temp-draft-label: is-zh ? "暂存草稿" : "Save Draft";
        in-out property <bool> temp-draft-cooling: false;
        property <string> current-title: active-page == 0 ? (is-zh ? "故事草稿" : "Story Draft") : active-page == 1 ? (is-zh ? "24 个问题" : "24 Questions") : active-page == 2 ? (is-zh ? "保护对象" : "Protected Objects") : active-page == 3 ? (is-zh ? "学习设置" : "Learning Settings") : active-page == 4 ? (is-zh ? "导出" : "Export") : active-page == 6 ? (is-zh ? "答案配置" : "Answer Editor") : active-page == 7 ? (is-zh ? "对象编辑" : "Object Editor") : (is-zh ? "设置" : "Settings");
        callback close-requested();
        callback save-temp-draft();
        callback previous-node();
        callback next-node();
        callback select-node(string);
        callback select-resource-group(string);
        callback save-resource();
        callback save-template();
        callback apply-template();
        callback pull-template-candidates();
        callback refresh-export();
        callback save-learning-policy();
        callback run-learning();
        callback learning-previous();
        callback learning-next();
        callback check-learning-current();
        callback export-package();
        callback browse-core-data-dir();
        callback browse-export-package-dir();
        callback open-core-settings();

        title: "StoryLock Core";
        preferred-width: 960px;
        preferred-height: 540px;
        min-width: 960px;
        max-width: 960px;
        min-height: 540px;
        max-height: 540px;
        background: #eef3f5;

        HorizontalBox {
            padding: 0px;
            spacing: 0px;

            Rectangle {
                min-width: 160px;
                max-width: 160px;
                background: #eef3f5;
                VerticalBox {
                    x: 18px;
                    y: 12px;
                    width: 124px;
                    height: 528px;
                    spacing: 7px;
                    MenuButton {
                        label: root.is-zh ? "24 个问题" : "24 Questions";
                        selected: root.active-page == 1;
                        clicked => { root.active-page = 1; }
                    }
                    MenuButton {
                        label: root.is-zh ? "故事草稿" : "Story Draft";
                        selected: root.active-page == 0;
                        clicked => { root.active-page = 0; }
                    }
                    MenuButton {
                        label: root.is-zh ? "保护对象" : "Protected Objects";
                        selected: root.active-page == 2;
                        clicked => { root.active-page = 2; }
                    }
                    SubMenuButton {
                        label: root.is-zh ? "普通授权对象" : "Normal Objects";
                        selected: root.active-page == 2 && root.resource-group == "normal";
                        clicked => {
                            root.select-resource-group("normal");
                        }
                    }
                    SubMenuButton {
                        label: root.is-zh ? "私密对象" : "Private Objects";
                        selected: root.active-page == 2 && root.resource-group == "private";
                        clicked => {
                            root.select-resource-group("private");
                        }
                    }
                    SubMenuButton {
                        label: root.is-zh ? "机密对象" : "Secret Objects";
                        selected: root.active-page == 2 && root.resource-group == "secret";
                        clicked => {
                            root.select-resource-group("secret");
                        }
                    }
                    MenuButton {
                        label: root.temp-draft-label;
                        selected: false;
                        enabled: !root.temp-draft-cooling;
                        clicked => { root.save-temp-draft(); }
                    }
                    MenuButton {
                        label: root.is-zh ? "学习设置" : "Learning";
                        selected: root.active-page == 3;
                        clicked => { root.active-page = 3; }
                    }
                    MenuButton {
                        label: root.is-zh ? "导出" : "Export";
                        selected: root.active-page == 4;
                        clicked => { root.active-page = 4; }
                    }
                }
            }

            Rectangle {
                min-width: 800px;
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
                            width: 470px;
                            height: 28px;
                            text: "StoryLock Core - " + root.current-title;
                            font-size: 16px;
                            font-weight: 800;
                            color: #17252f;
                            overflow: elide;
                        }
                        SettingsIconButton {
                            x: 688px;
                            y: 2px;
                            selected: false;
                            clicked => { root.open-core-settings(); }
                        }
                    }

                    Rectangle { height: 1px; background: #d7e1e6; }

                    Rectangle {
                        width: 720px;
                        height: 374px;
                        background: #eef3f5;

                        if root.active-page == 0: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 12px;
                            EditableRow { name: root.is-zh ? "故事标题" : "Story Title"; value <=> root.story-title; }
                            EditableRow { name: root.is-zh ? "故事摘要" : "Summary"; value <=> root.story-summary; }
                            LargeEditableText { name: root.is-zh ? "完整故事情节" : "Full Story Plot"; value <=> root.story-plot; }
                        }

                        if root.active-page == 1: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 12px;
                            QuestionOverviewGrid {
                                selected-question <=> root.selected-question;
                                question-1: root.question-1;
                                question-2: root.question-2;
                                question-3: root.question-3;
                                question-4: root.question-4;
                                question-5: root.question-5;
                                question-6: root.question-6;
                                question-7: root.question-7;
                                question-8: root.question-8;
                                question-9: root.question-9;
                                question-10: root.question-10;
                                question-11: root.question-11;
                                question-12: root.question-12;
                                question-13: root.question-13;
                                question-14: root.question-14;
                                question-15: root.question-15;
                                question-16: root.question-16;
                                question-17: root.question-17;
                                question-18: root.question-18;
                                question-19: root.question-19;
                                question-20: root.question-20;
                                question-21: root.question-21;
                                question-22: root.question-22;
                                question-23: root.question-23;
                                question-24: root.question-24;
                                select-node(value) => {
                                    root.select-node(value);
                                }
                            }
                        }

                        if root.active-page == 6: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 8px;
                            QuestionConfigPanel {
                                panel-height: 330px;
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
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: root.is-zh ? "返回问题概览" : "Back to Overview";
                                    primary: false;
                                    button-width: 150px;
                                    clicked => { root.active-page = 1; }
                                }
                            }
                        }

                        if root.active-page == 2: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 12px;
                            StaticRow { name: root.is-zh ? "页面目的" : "Purpose"; value: root.is-zh ? "在左侧菜单选择对象级别，本页以普通列表展示当前级别下的受保护对象。" : "Choose an object level from the left menu; this page shows a plain list for that level."; }
                            StaticRow { name: root.is-zh ? "当前级别" : "Current Level"; value: root.resource-group == "normal" ? (root.is-zh ? "普通授权对象" : "Normal Objects") : root.resource-group == "private" ? (root.is-zh ? "私密对象" : "Private Objects") : (root.is-zh ? "机密对象" : "Secret Objects"); }
                            StaticRow { name: root.is-zh ? "对象名称" : "Object Name"; value: root.display-name; }
                            StaticRow { name: root.is-zh ? "资源 ID" : "Resource ID"; value: root.resource-id; }
                            StaticRow { name: root.is-zh ? "对象 ID" : "Object ID"; value: root.object-id; }
                            StaticRow { name: root.is-zh ? "对象类型" : "Object Kind"; value: root.object-kind; }
                            StaticRow { name: root.is-zh ? "授权边界" : "Authorization Boundary"; value: root.authorization-frequency; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: root.is-zh ? "管理对象" : "Manage Object";
                                    primary: true;
                                    clicked => { root.active-page = 7; }
                                }
                            }
                            LogPanel { value: root.protected-object-list; panel-height: 120px; }
                        }

                        if root.active-page == 7: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 12px;
                            EditableRow { name: root.is-zh ? "资源 ID" : "Resource ID"; value <=> root.resource-id; }
                            EditableRow { name: root.is-zh ? "对象 ID" : "Object ID"; value <=> root.object-id; }
                            EditableRow { name: root.is-zh ? "对象类型" : "Object Kind"; value <=> root.object-kind; }
                            EditableRow { name: root.is-zh ? "显示名称" : "Display Name"; value <=> root.display-name; }
                            EditableRow { name: root.is-zh ? "保护级别" : "Protection Level"; value <=> root.resource-group; }
                            EditableRow { name: root.is-zh ? "正确数" : "Need Correct"; value <=> root.required-correct-count; }
                            EditableRow { name: root.is-zh ? "密钥引用" : "Secret Ref"; value <=> root.secret-reference; }
                            EditableRow { name: root.is-zh ? "授权频率" : "Auth Freq"; value <=> root.authorization-frequency; }
                            HorizontalBox {
                                spacing: 10px;
                                Rectangle { width: 164px; height: 1px; background: transparent; }
                                ActionButton {
                                    label: root.is-zh ? "保存对象" : "Save Object";
                                    primary: true;
                                    clicked => {
                                        root.save-resource();
                                        root.active-page = 2;
                                    }
                                }
                            }
                            LogPanel { value: root.config-status; panel-height: 120px; }
                        }

                        if root.active-page == 3: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 12px;
                            ScrollView {
                                width: 720px;
                                height: 374px;
                                vertical-scrollbar-policy: ScrollBarPolicy.as-needed;
                                viewport-width: 700px;
                                viewport-height: 900px;
                                VerticalBox {
                                    x: 0px;
                                    y: 0px;
                                    width: 690px;
                                    height: 900px;
                                    spacing: 10px;
                                    StaticRow { name: root.is-zh ? "计划文件" : "Plan File"; value: "learning-policy.json"; }
                                    StaticRow { name: root.is-zh ? "设计目的" : "Design Goal"; value: root.is-zh ? "防止用户长期不用后忘记问题答案；导出后由本地 Host 按计划强制复习。" : "Prevent forgotten answers after long gaps; after export, the local Host schedules mandatory review."; }
                                    StaticRow { name: root.is-zh ? "强制内容" : "Required Review"; value: root.is-zh ? "每次保留学习固定回答 22 个问题，用来确认用户仍然记得自己的故事锁。" : "Each retention review requires 22 fixed questions to confirm the user still remembers the StoryLock."; }
                                    StaticRow { name: root.is-zh ? "频率变化" : "Frequency Shape"; value: root.is-zh ? "先按天复习，再逐步降到每周、每月、每年；时间越久，强制学习频率越低。" : "Review starts by day, then lowers to weekly, monthly, and yearly cycles over time."; }
                                    StaticRow { name: root.is-zh ? "参数含义" : "Parameter Meaning"; value: root.is-zh ? "天数/周数/月数/年数表示阶段持续多久，频率表示该阶段隔多久触发一次复习。" : "Days/weeks/months/years set phase length; frequency sets how often that phase triggers review."; }
                                    EditableRow { name: root.is-zh ? "预学习错数" : "Pre Error Max"; value <=> root.pre-learning-error-tolerance; }
                                    EditableRow { name: root.is-zh ? "薄弱项上限" : "Weak Item Max"; value <=> root.weak-item-limit; }
                                    EditableRow { name: root.is-zh ? "初始期天数" : "Initial Days"; value <=> root.initial-days; }
                                    EditableRow { name: root.is-zh ? "初始期频率" : "Initial Freq"; value <=> root.initial-frequency-days; }
                                    EditableRow { name: root.is-zh ? "巩固期天数" : "Consolid. Days"; value <=> root.consolidation-days; }
                                    EditableRow { name: root.is-zh ? "巩固期频率" : "Consolid. Freq"; value <=> root.consolidation-frequency-days; }
                                    EditableRow { name: root.is-zh ? "适应期周数" : "Adapt Weeks"; value <=> root.adaptation-weeks; }
                                    EditableRow { name: root.is-zh ? "适应期频率" : "Adapt Freq"; value <=> root.adaptation-frequency-weeks; }
                                    EditableRow { name: root.is-zh ? "稳定期月数" : "Stable Months"; value <=> root.stable-months; }
                                    EditableRow { name: root.is-zh ? "稳定期频率" : "Stable Freq"; value <=> root.stable-frequency-months; }
                                    EditableRow { name: root.is-zh ? "长期期年数" : "Long Years"; value <=> root.long-term-years; }
                                    EditableRow { name: root.is-zh ? "长期期频率" : "Long Freq"; value <=> root.long-term-frequency-years; }
                                    StaticRow { name: root.is-zh ? "执行计划" : "Plan Summary"; value: root.learning-plan-summary; }
                                    StaticRow { name: root.is-zh ? "学习状态" : "Learning Status"; value: root.learning-status; }
                                    StaticRow { name: root.is-zh ? "当前题目" : "Current Question"; value: root.learning-position + "  " + root.learning-question; }
                                    QuestionTableRow { label: "Answer 1"; value <=> root.learning-answer-1; state <=> root.learning-answer-1-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 2"; value <=> root.learning-answer-2; state <=> root.learning-answer-2-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 3"; value <=> root.learning-answer-3; state <=> root.learning-answer-3-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 4"; value <=> root.learning-answer-4; state <=> root.learning-answer-4-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 5"; value <=> root.learning-answer-5; state <=> root.learning-answer-5-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 6"; value <=> root.learning-answer-6; state <=> root.learning-answer-6-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 7"; value <=> root.learning-answer-7; state <=> root.learning-answer-7-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 8"; value <=> root.learning-answer-8; state <=> root.learning-answer-8-state; show-state: true; }
                                    QuestionTableRow { label: "Answer 9"; value <=> root.learning-answer-9; state <=> root.learning-answer-9-state; show-state: true; }
                                    HorizontalBox {
                                        spacing: 10px;
                                        Rectangle { width: 164px; height: 1px; background: transparent; }
                                        ActionButton {
                                            label: root.is-zh ? "保存计划" : "Save Plan";
                                            primary: false;
                                            clicked => { root.save-learning-policy(); }
                                        }
                                        ActionButton {
                                            label: root.is-zh ? "开始测试" : "Start Test";
                                            primary: true;
                                            clicked => { root.run-learning(); }
                                        }
                                        ActionButton {
                                            label: root.is-zh ? "上一题" : "Previous";
                                            primary: false;
                                            button-width: 110px;
                                            clicked => { root.learning-previous(); }
                                        }
                                        ActionButton {
                                            label: root.is-zh ? "下一题" : "Next";
                                            primary: false;
                                            button-width: 110px;
                                            clicked => { root.learning-next(); }
                                        }
                                        ActionButton {
                                            label: root.is-zh ? "检查本题" : "Check";
                                            primary: true;
                                            button-width: 110px;
                                            clicked => { root.check-learning-current(); }
                                        }
                                    }
                                    LogPanel { value: root.learning-result; panel-height: 90px; }
                                }
                            }
                        }

                        if root.active-page == 4: VerticalBox {
                            x: 0px;
                            y: 0px;
                            width: 720px;
                            height: 374px;
                            spacing: 12px;
                            ScrollView {
                                width: 720px;
                                height: 374px;
                                vertical-scrollbar-policy: ScrollBarPolicy.as-needed;
                                viewport-width: 700px;
                                viewport-height: 760px;
                                VerticalBox {
                                    x: 0px;
                                    y: 0px;
                                    width: 690px;
                                    height: 760px;
                                    spacing: 10px;
                                    PathBrowseRow {
                                        name: root.is-zh ? "导出目录" : "Export Dir";
                                        value <=> root.export-package-dir;
                                        is-zh: root.is-zh;
                                        browse => { root.browse-export-package-dir(); }
                                    }
                                    StaticRow { name: root.is-zh ? "学习状态" : "Learning Status"; value: root.learning-status; }
                                    StaticRow { name: root.is-zh ? "测试结果" : "Test Result"; value: root.learning-result; }
                                    StaticRow { name: root.is-zh ? "加密数据" : "Encrypted Data"; value: root.is-zh ? "通过测试后导出加密 Vault 与相关包数据。" : "After the test passes, export the encrypted vault and package data."; }
                                    HorizontalBox {
                                        spacing: 10px;
                                        Rectangle { width: 164px; height: 1px; background: transparent; }
                                        ActionButton {
                                            label: root.is-zh ? "学习设置" : "Learning";
                                            primary: false;
                                            clicked => {
                                                root.active-page = 3;
                                            }
                                        }
                                        ActionButton {
                                            label: root.is-zh ? "导出" : "Export";
                                            primary: root.export-ready;
                                            clicked => {
                                                root.export-package();
                                            }
                                        }
                                    }
                                    LogPanel { value: root.export-preview + "\n\nExport writes the encrypted StoryLock vault and related package data only after the pre-export test passes. Host reads learning-policy.json and schedules retention checks from it."; panel-height: 150px; }
                                }
                            }
                        }

                    }
                }
            }
        }
    }

    export component StoryLockCoreSettingsDialog inherits Window {
        in-out property <string> language: "zh";
        property <bool> is-zh: language == "zh";
        in-out property <string> core-data-dir: "";
        callback close-requested();
        callback language-changed(string);
        callback browse-core-data-dir();

        title: is-zh ? "StoryLock Core 设置" : "StoryLock Core Settings";
        preferred-width: 800px;
        preferred-height: 450px;
        min-width: 800px;
        max-width: 800px;
        min-height: 450px;
        max-height: 450px;
        background: #eef3f5;

        VerticalBox {
            padding: 20px;
            spacing: 12px;
            Rectangle {
                height: 36px;
                background: transparent;
                Text {
                    x: 0px;
                    y: 4px;
                    width: 720px;
                    height: 28px;
                    text: root.is-zh ? "StoryLock Core 设置" : "StoryLock Core Settings";
                    color: #17252f;
                    font-size: 16px;
                    font-weight: 800;
                    overflow: elide;
                }
            }
            VerticalBox {
                spacing: 14px;
                HorizontalBox {
                    x: 10px;
                    width: 574px;
                    spacing: 14px;
                    Text {
                        text: root.is-zh ? "界面语言" : "UI Language";
                        width: 140px;
                        color: #62727d;
                        font-size: 13px;
                        vertical-alignment: center;
                    }
                    ComboBox {
                        width: 420px;
                        height: 32px;
                        model: ["中文", "English"];
                        current-value: root.is-zh ? "中文" : "English";
                        selected(value) => {
                            root.language = value == "中文" ? "zh" : "en";
                            root.language-changed(root.language);
                        }
                    }
                }
                PathBrowseRow {
                    name: root.is-zh ? "工作目录" : "Workspace Dir";
                    value <=> root.core-data-dir;
                    is-zh: root.is-zh;
                    browse => { root.browse-core-data-dir(); }
                }
                StaticRow { name: root.is-zh ? "目录内容" : "Directory Files"; value: root.is-zh ? "工作目录中包含 vault.stlk、learning-policy.json、package-manifest.json、resource-catalog.json 等文件。" : "The workspace contains vault.stlk, learning-policy.json, package-manifest.json, resource-catalog.json, and related files."; }
            }
        }
    }

    export component AnswerEditorDialog inherits Window {
        in-out property <string> language: "zh";
        property <bool> is-zh: language == "zh";
        in-out property <string> selected-question: "1";
        in-out property <string> question-text: "";
        in-out property <string> answer-1: "";
        in-out property <string> answer-1-state: "wrong";
        in-out property <string> answer-2: "";
        in-out property <string> answer-2-state: "wrong";
        in-out property <string> answer-3: "";
        in-out property <string> answer-3-state: "wrong";
        in-out property <string> answer-4: "";
        in-out property <string> answer-4-state: "wrong";
        in-out property <string> answer-5: "";
        in-out property <string> answer-5-state: "wrong";
        in-out property <string> answer-6: "";
        in-out property <string> answer-6-state: "wrong";
        in-out property <string> answer-7: "";
        in-out property <string> answer-7-state: "wrong";
        in-out property <string> answer-8: "";
        in-out property <string> answer-8-state: "wrong";
        in-out property <string> answer-9: "";
        in-out property <string> answer-9-state: "wrong";
        callback close-requested();
        callback save-requested();
        callback previous-node();
        callback next-node();
        callback select-node(string);

        title: is-zh ? "答案配置" : "Answer Editor";
        preferred-width: 960px;
        preferred-height: 540px;
        min-width: 960px;
        max-width: 960px;
        min-height: 540px;
        max-height: 540px;
        background: #eef3f5;

        VerticalBox {
            padding: 20px;
            spacing: 12px;
            Rectangle {
                height: 36px;
                background: transparent;
                Text {
                    x: 0px;
                    y: 4px;
                    width: 700px;
                    height: 28px;
                    text: is-zh ? "答案配置 - " + root.selected-question : "Answer Editor - " + root.selected-question;
                    color: #17252f;
                    font-size: 16px;
                    font-weight: 800;
                    overflow: elide;
                }
                ActionButton {
                    x: 740px;
                    y: 1px;
                    label: is-zh ? "保存" : "Save";
                    primary: true;
                    button-width: 96px;
                    clicked => { root.save-requested(); }
                }
                ActionButton {
                    x: 846px;
                    y: 1px;
                    label: is-zh ? "关闭" : "Close";
                    primary: false;
                    button-width: 96px;
                    clicked => { root.close-requested(); }
                }
            }
            QuestionConfigPanel {
                panel-height: 422px;
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
        preferred-width: 640px;
        preferred-height: 360px;
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
    app.set_capabilities(SharedString::from(if config.remote_enabled {
        "health, verify, authorize, revoke, execute, relay_poll"
    } else {
        "health, verify, authorize, revoke, execute"
    }));
    app.set_call_chain(SharedString::from(
        "verify -> authorize -> execute -> revoke",
    ));
    app.set_management_stats(SharedString::from(format!(
        "Live redacted statistics are available at http://127.0.0.1:{}/ui and /ui/status.\n\nYian Host may show authorization modes, required grid cells, managed-object call counts, agent/requester counts, remote-interface access counts, and error-call totals.\n\nStory template candidates can be generated by Host and queued at /story-template/generate; StoryLock must explicitly pull them from /story-template/candidates. Host never invokes StoryLock.\n\nLLM keys are direct-access generator config. Host may show configured/missing, but must not display key values.\n\nIt must not display StoryLock drafts, vault files, package paths, question answers, passwords, private keys, signing key bytes, shared secrets, or raw story text.",
        config.host_port
    )));
    app.set_diagnostics(SharedString::from(
        "Yian Host is storage-blind. It does not read or display StoryLock drafts, vault files, manifests, catalogs, templates, package paths, question answers, passwords, private keys, signing key bytes, shared secrets, or raw story text.",
    ));
    app.set_language(SharedString::from("zh"));
    let local_health_url = config.health_url.clone();
    app.on_test_local_host(move || SharedString::from(test_http_endpoint("Local Host", &local_health_url)));
    let remote_gateway_url = config.gateway_base_url.clone();
    app.on_test_remote_connection(move || {
        SharedString::from(test_http_endpoint("Remote Gateway", &remote_gateway_url))
    });
    let host_port_for_nine_grid = config.host_port;
    app.on_test_managed_object_nine_grid(move |tier| {
        SharedString::from(test_managed_object_nine_grid(host_port_for_nine_grid, tier.to_string()))
    });
    let host_language = Rc::new(RefCell::new(String::from("zh")));
    let core_package_dir = storylock_core_package_dir();
    let _ = ensure_storylock_core_package(&core_package_dir);
    app.set_management_stats(SharedString::from(format!(
        "Live redacted statistics are available at http://127.0.0.1:{}/ui and /ui/status.\n\n{}\n\nYian Host reads learning-policy.json for retention-learning scheduling, but does not read StoryLock drafts, vault files, question text, answers, passwords, private keys, signing key bytes, shared secrets, or raw story text.",
        config.host_port,
        host_learning_plan_status(&core_package_dir)
    )));
    let core_window: Rc<RefCell<Option<StoryLockCoreApp>>> = Rc::new(RefCell::new(None));
    let settings_window: Rc<RefCell<Option<SettingsDialog>>> = Rc::new(RefCell::new(None));
    let settings_window_for_open = Rc::clone(&settings_window);
    let host_for_settings = app.as_weak();
    let host_language_for_settings = Rc::clone(&host_language);
    let core_window_for_settings = Rc::clone(&core_window);
    let host_for_settings_lock = app.as_weak();
    let shared_status = Rc::new(RefCell::new(String::from("")));
    let shared_status_for_settings = Rc::clone(&shared_status);
    let core_package_dir_for_settings = core_package_dir.clone();
    app.on_open_settings(move || {
        if let Some(existing) = settings_window_for_open.borrow().as_ref() {
            if let Err(error) = existing.show() {
                eprintln!("failed to show settings window: {error}");
            }
            if let Some(host) = host_for_settings.upgrade() {
                host.set_settings_open(true);
            }
            return;
        }
        match SettingsDialog::new() {
            Ok(settings) => {
                settings.set_language(SharedString::from(host_language_for_settings.borrow().clone()));
                settings.set_core_launch_status(SharedString::from(shared_status_for_settings.borrow().clone()));
                if let Some(host) = host_for_settings.upgrade() {
                    let status = if host_language_for_settings.borrow().as_str() == "zh" {
                        "设置已打开".to_string()
                    } else {
                        "Settings opened".to_string()
                    };
                    host.set_connection_test_status(SharedString::from(status));
                    host.set_settings_open(true);
                }
                let settings_weak = settings.as_weak();
                let settings_weak_for_close = settings_weak.clone();
                let host_for_language = host_for_settings.clone();
                let host_language_for_language = Rc::clone(&host_language_for_settings);
                let host_language_for_open_storylock = Rc::clone(&host_language_for_settings);
                let core_window_for_language = Rc::clone(&core_window_for_settings);
                let core_package_dir_for_open_storylock = core_package_dir_for_settings.clone();
                let core_window_for_open_storylock = Rc::clone(&core_window_for_settings);
                let host_language_for_storylock_core = Rc::clone(&host_language_for_settings);
                let settings_weak_for_storylock_close = settings.as_weak();
                settings.on_language_changed(move |language| {
                    let language_string = language.to_string();
                    *host_language_for_language.borrow_mut() = language_string.clone();
                    if let Some(settings) = settings_weak.upgrade() {
                        settings.set_language(SharedString::from(language_string.clone()));
                    }
                    if let Some(host) = host_for_language.upgrade() {
                        host.set_language(SharedString::from(language_string.clone()));
                    }
                    if let Some(core) = core_window_for_language.borrow().as_ref() {
                        core.set_language(SharedString::from(language_string.clone()));
                    }
                    if let Some(host) = host_for_language.upgrade() {
                        let text = if language_string == "zh" {
                            "语言已切换，设置窗保持打开".to_string()
                        } else {
                            "Language changed, settings stay open".to_string()
                        };
                        host.set_connection_test_status(SharedString::from(text));
                    }
                });
                let host_for_open_storylock = host_for_settings.clone();
                let shared_status_for_storylock = Rc::clone(&shared_status_for_settings);
                let host_for_settings_lock_close = host_for_settings_lock.clone();
                let host_language_for_close = Rc::clone(&host_language_for_settings);
                settings.on_open_storylock_core(move || {
                    if let Some(host) = host_for_open_storylock.upgrade() {
                        let status = if host_language_for_open_storylock.borrow().as_str() == "zh" {
                            "StoryLock 将从设置弹窗打开".to_string()
                        } else {
                            "StoryLock will open from Settings".to_string()
                        };
                        *shared_status_for_storylock.borrow_mut() = status.clone();
                        host.set_connection_test_status(SharedString::from(status));
                    }
                    if let Err(error) = ensure_storylock_core_package(&core_package_dir_for_open_storylock) {
                        eprintln!("failed to initialize StoryLock Core package: {error}");
                    }
                    if let Some(host) = host_for_open_storylock.upgrade() {
                        let storylock_open = if host_language_for_open_storylock.borrow().as_str() == "zh" {
                            "StoryLock 已打开".to_string()
                        } else {
                            "StoryLock open".to_string()
                        };
                        host.set_connection_test_status(SharedString::from(storylock_open));
                    }
                    if let Some(core) = core_window_for_open_storylock.borrow().as_ref() {
                        initialize_storylock_core_window(core, &core_package_dir_for_open_storylock);
                        core.set_language(SharedString::from(host_language_for_storylock_core.borrow().clone()));
                        match core.show() {
                            Ok(()) => {
                                return;
                            }
                            Err(error) => {
                                eprintln!("failed to show existing StoryLock Core window: {error}");
                            }
                        }
                    }
                    *core_window_for_open_storylock.borrow_mut() = None;
                    match StoryLockCoreApp::new() {
                        Ok(core) => {
                            core.set_language(SharedString::from(host_language_for_storylock_core.borrow().clone()));
                            initialize_storylock_core_window(&core, &core_package_dir_for_open_storylock);
                            let host_for_storylock_close = host_for_open_storylock.clone();
                            let settings_for_storylock_close = settings_weak_for_storylock_close.clone();
                            let shared_status_for_storylock_close = Rc::clone(&shared_status_for_storylock);
                            let host_language_for_storylock_close =
                                Rc::clone(&host_language_for_open_storylock);
                            let notify_storylock_closed: Rc<dyn Fn()> = Rc::new(move || {
                                let status =
                                    if host_language_for_storylock_close.borrow().as_str() == "zh" {
                                        "StoryLock 已关闭".to_string()
                                    } else {
                                        "StoryLock closed".to_string()
                                    };
                                *shared_status_for_storylock_close.borrow_mut() = status.clone();
                                if let Some(host) = host_for_storylock_close.upgrade() {
                                    host.set_connection_test_status(SharedString::from(status.clone()));
                                }
                                if let Some(settings) = settings_for_storylock_close.upgrade() {
                                    settings.set_core_launch_status(SharedString::from(status));
                                }
                            });
                            wire_storylock_core_callbacks(
                                &core,
                                core_package_dir_for_open_storylock.clone(),
                                Rc::clone(&core_window_for_open_storylock),
                                notify_storylock_closed,
                                config.host_port,
                            );
                            if let Err(error) = core.show() {
                                eprintln!("failed to show StoryLock Core window: {error}");
                                return;
                            }
                            *core_window_for_open_storylock.borrow_mut() = Some(core);
                        }
                        Err(error) => eprintln!("failed to create StoryLock Core window: {error}"),
                    }
                });
                settings.on_close_requested(move || {
                    if let Some(settings) = settings_weak_for_close.upgrade() {
                        let _ = settings.hide();
                    }
                    if let Some(host) = host_for_settings_lock_close.upgrade() {
                        host.set_settings_open(false);
                        let text = if host_language_for_close.borrow().as_str() == "zh" {
                            "设置已关闭".to_string()
                        } else {
                            "Settings closed".to_string()
                        };
                        host.set_connection_test_status(SharedString::from(text));
                    }
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
    let core_package_dir_for_callback = core_package_dir.clone();
    let host_for_storylock_close = app.as_weak();
    let settings_window_for_storylock_close = Rc::clone(&settings_window);
    let host_language_for_storylock_close = Rc::clone(&host_language);
    let shared_status_for_storylock_close = Rc::clone(&shared_status);
    app.on_open_storylock_core(move || {
        if let Err(error) = ensure_storylock_core_package(&core_package_dir_for_callback) {
            eprintln!("failed to initialize StoryLock Core package: {error}");
        }
        if let Some(core) = core_window_for_callback.borrow().as_ref() {
            initialize_storylock_core_window(core, &core_package_dir_for_callback);
            core.set_config_status(SharedString::from(
                "StoryLock Core is already open. Existing local window was focused.",
            ));
            match core.show() {
                Ok(()) => {
                    return;
                }
                Err(error) => {
                    eprintln!("failed to show existing StoryLock Core window: {error}");
                }
            }
        }
        *core_window_for_callback.borrow_mut() = None;
        match StoryLockCoreApp::new() {
            Ok(core) => {
                initialize_storylock_core_window(&core, &core_package_dir_for_callback);
                let host_for_closed = host_for_storylock_close.clone();
                let settings_window_for_closed = Rc::clone(&settings_window_for_storylock_close);
                let host_language_for_closed = Rc::clone(&host_language_for_storylock_close);
                let shared_status_for_closed = Rc::clone(&shared_status_for_storylock_close);
                let notify_storylock_closed: Rc<dyn Fn()> = Rc::new(move || {
                    let status = if host_language_for_closed.borrow().as_str() == "zh" {
                        "StoryLock 已关闭".to_string()
                    } else {
                        "StoryLock closed".to_string()
                    };
                    *shared_status_for_closed.borrow_mut() = status.clone();
                    if let Some(host) = host_for_closed.upgrade() {
                        host.set_connection_test_status(SharedString::from(status.clone()));
                    }
                    if let Some(settings) = settings_window_for_closed.borrow().as_ref() {
                        settings.set_core_launch_status(SharedString::from(status));
                    }
                });
                wire_storylock_core_callbacks(
                    &core,
                    core_package_dir_for_callback.clone(),
                    Rc::clone(&core_window_for_callback),
                    notify_storylock_closed,
                    config.host_port,
                );
                if let Err(error) = core.show() {
                    eprintln!("failed to show StoryLock Core window: {error}");
                    return;
                }
                *core_window_for_callback.borrow_mut() = Some(core);
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

fn test_managed_object_nine_grid(host_port: u16, tier: String) -> String {
    let client = match Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(client) => client,
        Err(error) => return format!("Nine-grid test failed: client setup failed: {error}"),
    };
    let status_url = format!("http://127.0.0.1:{host_port}/ui/status");
    let status = match client.get(&status_url).send() {
        Ok(response) => match response.json::<Value>() {
            Ok(value) => value,
            Err(error) => return format!("Nine-grid test failed: could not parse ui status: {error}"),
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
            "保密级别对象",
        ),
        "top-secret" => (
            "story_edit",
            "story_edit",
            "requestPasswordFill",
            "高机密对象",
        ),
        _ => (
            "single_read",
            "password_fill",
            "requestPasswordFill",
            "普通对象",
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
            Err(error) => return format!("Nine-grid test failed: could not parse verification response: {error}"),
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
    let grid_size = grid
        .get("gridSize")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    format!(
        "Nine-grid ready: {label}, object={object_ref}, verificationId={verification_id}, {required_cells}/{grid_size} cells"
    )
}

fn host_learning_plan_status(package_dir: &Path) -> String {
    let policy_path = storylock_core_learning_policy_path(package_dir);
    if !policy_path.exists() {
        return "Learning plan: not configured. Open StoryLock Core Export and save learning-policy.json.".to_string();
    }
    let policy = read_learning_policy(package_dir);
    let summary = learning_policy_summary(&policy);
    let current_phase = policy
        .get("execution")
        .and_then(|value| value.get("currentPhase"))
        .and_then(Value::as_str)
        .unwrap_or("initial");
    let status = policy
        .get("execution")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("pending_export");
    format!("Learning plan: {status}, current phase={current_phase}\n{summary}")
}

fn storylock_core_manifest_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("package-manifest.json")
}

fn storylock_core_package_dir() -> std::path::PathBuf {
    if let Ok(configured) = std::env::var("STORYLOCK_CORE_DATA_DIR") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            return std::path::PathBuf::from(trimmed).join("identity-package");
        }
    }
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        return std::path::PathBuf::from(appdata)
            .join("StoryLock")
            .join("core")
            .join("identity-package");
    }
    std::path::PathBuf::from(".")
        .join(".storylock-core-data")
        .join("identity-package")
}

fn storylock_core_package_dir_from_window(core: &StoryLockCoreApp, fallback: &Path) -> std::path::PathBuf {
    let configured = core.get_core_data_dir();
    let trimmed = configured.as_str().trim();
    if trimmed.is_empty() {
        fallback.to_path_buf()
    } else {
        std::path::PathBuf::from(trimmed)
    }
}

fn ensure_storylock_core_package_dir_from_window(
    core: &StoryLockCoreApp,
    fallback: &Path,
) -> Result<std::path::PathBuf> {
    let package_dir = storylock_core_package_dir_from_window(core, fallback);
    ensure_storylock_core_package(&package_dir)?;
    Ok(package_dir)
}

fn storylock_core_catalog_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("resource-catalog.json")
}

fn storylock_core_vault_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("vault.stlk")
}

fn storylock_core_learning_policy_path(package_dir: &Path) -> std::path::PathBuf {
    package_dir.join("learning-policy.json")
}

fn required_storylock_package_files() -> [&'static str; 4] {
    [
        "package-manifest.json",
        "resource-catalog.json",
        "vault.stlk",
        "learning-policy.json",
    ]
}

fn default_storylock_vault_json() -> Value {
    json!({
        "schemaVersion": "1",
        "authorDraft": default_author_draft_json(),
        "pendingAuthorDraft": Value::Null,
        "storyDraftTemplates": default_story_draft_templates_json(),
        "templates": default_storylock_templates_json()
    })
}

fn default_story_draft_templates_json() -> Value {
    json!({
        "schemaVersion": "storylock-story-draft-templates-v1",
        "defaultTemplateId": "dongguo-wolf",
        "items": [
            dongguo_wolf_author_draft_json(),
            zhizi_yilin_author_draft_json(),
            shouzhudaitu_author_draft_json()
        ]
    })
}

fn default_storylock_templates_json() -> Value {
    json!({
        "loginSites": default_login_templates_json(),
        "signingActions": default_signing_templates_json(),
        "agentTasks": default_agent_templates_json()
    })
}

fn default_learning_policy_json() -> Value {
    json!({
        "schemaVersion": "1",
        "policyId": "storylock-default-learning-policy",
        "updatedAt": ui_now_timestamp(),
        "hostReadable": true,
        "preLearning": {
            "questionCount": 24,
            "promptsPerQuestion": 2,
            "totalPrompts": 48,
            "minRepeatGap": 12,
            "errorTolerance": 2,
            "weakItemLimit": 3
        },
        "retentionLearning": {
            "description": "Prevents users from forgetting StoryLock answers by forcing periodic review after export.",
            "questionCount": 22,
            "questionCountMeaning": "Each retention review requires 22 fixed questions.",
            "frequencyDesign": "Review frequency decreases over time: daily, weekly, monthly, then yearly.",
            "phaseParameterMeaning": "Duration sets how long a phase lasts; frequency sets how often review is triggered in that phase.",
            "phases": [
                { "phase": "initial", "duration": { "unit": "day", "value": 3 }, "frequency": { "unit": "day", "value": 1 } },
                { "phase": "consolidation", "duration": { "unit": "day", "value": 4 }, "frequency": { "unit": "day", "value": 2 } },
                { "phase": "adaptation", "duration": { "unit": "week", "value": 3 }, "frequency": { "unit": "week", "value": 1 } },
                { "phase": "stable", "duration": { "unit": "month", "value": 4 }, "frequency": { "unit": "month", "value": 1 } },
                { "phase": "long_term", "duration": { "unit": "year", "value": 1 }, "frequency": { "unit": "year", "value": 1 } }
            ]
        },
        "execution": {
            "status": "pending_export",
            "currentPhase": "initial",
            "nextCheckAfter": { "unit": "day", "value": 1 },
            "lastResult": "not_started"
        }
    })
}

fn cleanup_legacy_storylock_package_files(package_dir: &Path) -> Result<()> {
    for path in [
        package_dir.join("author-draft.json"),
        package_dir.join(".tmp").join("author-draft.pending.json"),
        package_dir.join("templates").join("login-sites.json"),
        package_dir.join("templates").join("signing-actions.json"),
        package_dir.join("templates").join("agent-tasks.json"),
    ] {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    let templates_dir = package_dir.join("templates");
    if templates_dir.exists()
        && templates_dir.is_dir()
        && fs::read_dir(&templates_dir)?.next().is_none()
    {
        fs::remove_dir(&templates_dir)?;
    }
    let tmp_dir = package_dir.join(".tmp");
    if tmp_dir.exists() && tmp_dir.is_dir() && fs::read_dir(&tmp_dir)?.next().is_none() {
        fs::remove_dir(&tmp_dir)?;
    }
    Ok(())
}

fn ensure_storylock_core_package(package_dir: &Path) -> Result<()> {
    fs::create_dir_all(package_dir)?;
    write_json_if_missing(
        &storylock_core_manifest_path(package_dir),
        &json!({
            "packageId": "windows-storylock-core-local",
            "version": "0.1.0",
            "createdAt": ui_now_timestamp(),
            "description": "Local Windows StoryLock Core package.",
            "files": required_storylock_package_files()
        }),
    )?;
    ensure_manifest_lists_required_files(package_dir)?;
    write_json_if_missing(
        &storylock_core_catalog_path(package_dir),
        &default_resource_catalog_json(),
    )?;
    write_json_if_missing(
        &storylock_core_learning_policy_path(package_dir),
        &default_learning_policy_json(),
    )?;
    ensure_storylock_vault(package_dir)?;
    cleanup_legacy_storylock_package_files(package_dir)?;
    Ok(())
}

fn ensure_manifest_lists_required_files(package_dir: &Path) -> Result<()> {
    let path = storylock_core_manifest_path(package_dir);
    let mut manifest = read_json_or_default(&path, json!({}));
    if !manifest.is_object() {
        manifest = json!({});
    }
    if manifest.get("packageId").is_none() {
        manifest["packageId"] = json!("windows-storylock-core-local");
    }
    if manifest.get("version").is_none() {
        manifest["version"] = json!("0.1.0");
    }
    if manifest.get("createdAt").is_none() {
        manifest["createdAt"] = json!(ui_now_timestamp());
    }
    let mut files = manifest
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for required_file in required_storylock_package_files() {
        if !files.iter().any(|item| item.as_str() == Some(required_file)) {
            files.push(json!(required_file));
        }
    }
    manifest["files"] = Value::Array(files);
    fs::write(path, serde_json::to_vec_pretty(&manifest)?)?;
    Ok(())
}

fn ensure_storylock_vault(package_dir: &Path) -> Result<()> {
    if storylock_core_vault_path(package_dir).exists() {
        let mut vault = read_storylock_vault_payload(package_dir);
        let before = vault.clone();
        if vault.get("storyDraftTemplates").is_none() {
            let draft = storylock_author_draft_from_vault(&vault);
            vault["storyDraftTemplates"] = story_draft_templates_from_draft(&draft);
        }
        merge_builtin_story_draft_templates(&mut vault);
        if vault != before {
            save_storylock_vault_payload(package_dir, vault)?;
        }
        return Ok(());
    }
    let legacy_draft = read_json_or_default(
        &package_dir.join("author-draft.json"),
        default_author_draft_json(),
    );
    let legacy_templates = json!({
        "loginSites": read_json_or_default(
            &package_dir.join("templates").join("login-sites.json"),
            default_login_templates_json(),
        ),
        "signingActions": read_json_or_default(
            &package_dir.join("templates").join("signing-actions.json"),
            default_signing_templates_json(),
        ),
        "agentTasks": read_json_or_default(
            &package_dir.join("templates").join("agent-tasks.json"),
            default_agent_templates_json(),
        )
    });
    let vault = json!({
        "schemaVersion": "1",
        "authorDraft": legacy_draft,
        "pendingAuthorDraft": Value::Null,
        "storyDraftTemplates": default_story_draft_templates_json(),
        "templates": legacy_templates,
    });
    write_storylock_vault(package_dir, &vault)
}

fn story_draft_templates_from_draft(draft: &Value) -> Value {
    let template_id = draft
        .get("templateId")
        .and_then(Value::as_str)
        .unwrap_or("current-author-draft");
    json!({
        "schemaVersion": "storylock-story-draft-templates-v1",
        "defaultTemplateId": template_id,
        "items": [draft.clone()]
    })
}

fn merge_builtin_story_draft_templates(vault: &mut Value) {
    let mut templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    if !templates.is_object() {
        templates = default_story_draft_templates_json();
    }
    let mut items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for builtin in [
        dongguo_wolf_author_draft_json(),
        zhizi_yilin_author_draft_json(),
        shouzhudaitu_author_draft_json(),
    ] {
        let template_id = builtin
            .get("templateId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !items
            .iter()
            .any(|item| item.get("templateId").and_then(Value::as_str) == Some(template_id))
        {
            items.push(builtin);
        }
    }
    templates["schemaVersion"] = json!("storylock-story-draft-templates-v1");
    templates["defaultTemplateId"] = json!("dongguo-wolf");
    templates["items"] = Value::Array(items);
    vault["storyDraftTemplates"] = templates;
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

fn read_learning_policy(package_dir: &Path) -> Value {
    read_json_or_default(
        &storylock_core_learning_policy_path(package_dir),
        default_learning_policy_json(),
    )
}

fn write_learning_policy(package_dir: &Path, policy: &Value) -> Result<()> {
    let path = storylock_core_learning_policy_path(package_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(policy)?)?;
    ensure_manifest_lists_required_files(package_dir)
}

fn bounded_policy_int(value: &str, field_name: &str) -> Result<i64> {
    let parsed = value
        .trim()
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("{field_name} must be a number from 1 to 9"))?;
    if !(1..=9).contains(&parsed) {
        anyhow::bail!("{field_name} must be between 1 and 9");
    }
    Ok(parsed)
}

fn learning_policy_from_window(core: &StoryLockCoreApp) -> Result<Value> {
    Ok(json!({
        "schemaVersion": "1",
        "policyId": "storylock-core-learning-policy",
        "updatedAt": ui_now_timestamp(),
        "hostReadable": true,
        "preLearning": {
            "questionCount": 24,
            "promptsPerQuestion": 2,
            "totalPrompts": 48,
            "minRepeatGap": 12,
            "errorTolerance": bounded_policy_int(core.get_pre_learning_error_tolerance().as_str(), "pre-learning error tolerance")?,
            "weakItemLimit": bounded_policy_int(core.get_weak_item_limit().as_str(), "weak item limit")?
        },
        "retentionLearning": {
            "description": "Prevents users from forgetting StoryLock answers by forcing periodic review after export.",
            "questionCount": 22,
            "questionCountMeaning": "Each retention review requires 22 fixed questions.",
            "frequencyDesign": "Review frequency decreases over time: daily, weekly, monthly, then yearly.",
            "phaseParameterMeaning": "Duration sets how long a phase lasts; frequency sets how often review is triggered in that phase.",
            "phases": [
                {
                    "phase": "initial",
                    "duration": { "unit": "day", "value": bounded_policy_int(core.get_initial_days().as_str(), "initial days")? },
                    "frequency": { "unit": "day", "value": bounded_policy_int(core.get_initial_frequency_days().as_str(), "initial frequency")? }
                },
                {
                    "phase": "consolidation",
                    "duration": { "unit": "day", "value": bounded_policy_int(core.get_consolidation_days().as_str(), "consolidation days")? },
                    "frequency": { "unit": "day", "value": bounded_policy_int(core.get_consolidation_frequency_days().as_str(), "consolidation frequency")? }
                },
                {
                    "phase": "adaptation",
                    "duration": { "unit": "week", "value": bounded_policy_int(core.get_adaptation_weeks().as_str(), "adaptation weeks")? },
                    "frequency": { "unit": "week", "value": bounded_policy_int(core.get_adaptation_frequency_weeks().as_str(), "adaptation frequency")? }
                },
                {
                    "phase": "stable",
                    "duration": { "unit": "month", "value": bounded_policy_int(core.get_stable_months().as_str(), "stable months")? },
                    "frequency": { "unit": "month", "value": bounded_policy_int(core.get_stable_frequency_months().as_str(), "stable frequency")? }
                },
                {
                    "phase": "long_term",
                    "duration": { "unit": "year", "value": bounded_policy_int(core.get_long_term_years().as_str(), "long-term years")? },
                    "frequency": { "unit": "year", "value": bounded_policy_int(core.get_long_term_frequency_years().as_str(), "long-term frequency")? }
                }
            ]
        },
        "execution": {
            "status": "active_after_export",
            "currentPhase": "initial",
            "nextCheckAfter": {
                "unit": "day",
                "value": bounded_policy_int(core.get_initial_frequency_days().as_str(), "initial frequency")?
            },
            "lastResult": "not_started"
        }
    }))
}

fn policy_number(policy: &Value, path: &[&str], fallback: i64) -> String {
    let mut current = policy;
    for key in path {
        current = current.get(*key).unwrap_or(&Value::Null);
    }
    current.as_i64().unwrap_or(fallback).to_string()
}

fn phase_number(policy: &Value, phase: &str, section: &str, fallback: i64) -> String {
    policy
        .get("retentionLearning")
        .and_then(|value| value.get("phases"))
        .and_then(Value::as_array)
        .and_then(|phases| {
            phases
                .iter()
                .find(|item| item.get("phase").and_then(Value::as_str) == Some(phase))
        })
        .and_then(|item| item.get(section))
        .and_then(|value| value.get("value"))
        .and_then(Value::as_i64)
        .unwrap_or(fallback)
        .to_string()
}

fn learning_policy_summary(policy: &Value) -> String {
    let pre_errors = policy_number(policy, &["preLearning", "errorTolerance"], 2);
    let weak_limit = policy_number(policy, &["preLearning", "weakItemLimit"], 3);
    let initial_frequency = phase_number(policy, "initial", "frequency", 1);
    let consolidation_frequency = phase_number(policy, "consolidation", "frequency", 2);
    let adaptation_frequency = phase_number(policy, "adaptation", "frequency", 1);
    let stable_frequency = phase_number(policy, "stable", "frequency", 1);
    let long_frequency = phase_number(policy, "long_term", "frequency", 1);
    format!(
        "Pre-learning: 48 prompts, max errors {pre_errors}, weak items <= {weak_limit}. Retention: 22 questions; initial every {initial_frequency} day(s), consolidation every {consolidation_frequency} day(s), adaptation every {adaptation_frequency} week(s), stable every {stable_frequency} month(s), long-term every {long_frequency} year(s)."
    )
}

fn load_learning_policy_into_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let policy = read_learning_policy(package_dir);
    core.set_pre_learning_error_tolerance(SharedString::from(policy_number(
        &policy,
        &["preLearning", "errorTolerance"],
        2,
    )));
    core.set_weak_item_limit(SharedString::from(policy_number(
        &policy,
        &["preLearning", "weakItemLimit"],
        3,
    )));
    core.set_initial_days(SharedString::from(phase_number(&policy, "initial", "duration", 3)));
    core.set_initial_frequency_days(SharedString::from(phase_number(&policy, "initial", "frequency", 1)));
    core.set_consolidation_days(SharedString::from(phase_number(&policy, "consolidation", "duration", 4)));
    core.set_consolidation_frequency_days(SharedString::from(phase_number(&policy, "consolidation", "frequency", 2)));
    core.set_adaptation_weeks(SharedString::from(phase_number(&policy, "adaptation", "duration", 3)));
    core.set_adaptation_frequency_weeks(SharedString::from(phase_number(&policy, "adaptation", "frequency", 1)));
    core.set_stable_months(SharedString::from(phase_number(&policy, "stable", "duration", 4)));
    core.set_stable_frequency_months(SharedString::from(phase_number(&policy, "stable", "frequency", 1)));
    core.set_long_term_years(SharedString::from(phase_number(&policy, "long_term", "duration", 1)));
    core.set_long_term_frequency_years(SharedString::from(phase_number(&policy, "long_term", "frequency", 1)));
    core.set_learning_plan_summary(SharedString::from(learning_policy_summary(&policy)));
}

fn save_learning_policy_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let policy = learning_policy_from_window(core)?;
    write_learning_policy(package_dir, &policy)?;
    core.set_learning_plan_summary(SharedString::from(learning_policy_summary(&policy)));
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    Ok(())
}

fn read_storylock_vault(package_dir: &Path) -> Value {
    let path = storylock_core_vault_path(package_dir);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(envelope) = serde_json::from_str::<ProtectedEnvelope>(&content) {
                if let Ok(bytes) = dpapi_unprotect_from_base64(&envelope.cipher_text) {
                    if let Ok(vault) = serde_json::from_slice::<Value>(&bytes) {
                        return vault;
                    }
                }
            }
        }
    }
    default_storylock_vault_json()
}

fn write_storylock_vault(package_dir: &Path, vault: &Value) -> Result<()> {
    let path = storylock_core_vault_path(package_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_vec(vault)?;
    let envelope = ProtectedEnvelope {
        schema_version: "dpapi-protected-v1".to_string(),
        protected_by: "windows-dpapi".to_string(),
        created_at: ui_now_timestamp(),
        cipher_text: dpapi_protect_to_base64(&serialized)?,
    };
    fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;
    Ok(())
}

fn read_storylock_vault_payload(package_dir: &Path) -> Value {
    read_storylock_vault(package_dir)
}

fn save_storylock_vault_payload(package_dir: &Path, mut vault: Value) -> Result<()> {
    if vault.get("schemaVersion").is_none() {
        vault["schemaVersion"] = json!("1");
    }
    write_storylock_vault(package_dir, &vault)
}

fn storylock_author_draft_from_vault(vault: &Value) -> Value {
    vault
        .get("pendingAuthorDraft")
        .cloned()
        .filter(|value| !value.is_null())
        .or_else(|| vault.get("authorDraft").cloned())
        .unwrap_or_else(default_author_draft_json)
}

fn storylock_templates_from_vault(vault: &Value) -> Value {
    vault
        .get("templates")
        .and_then(Value::as_object)
        .map(|templates| Value::Object(templates.clone()))
        .unwrap_or_else(default_storylock_templates_json)
}

fn read_effective_author_draft(package_dir: &Path) -> Value {
    let vault = read_storylock_vault_payload(package_dir);
    storylock_author_draft_from_vault(&vault)
}

fn write_pending_author_draft(package_dir: &Path, draft: &Value) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    let mut normalized = draft.clone();
    normalize_author_draft_schema(&mut normalized);
    vault["pendingAuthorDraft"] = normalized;
    save_storylock_vault_payload(package_dir, vault)
}

fn normalize_author_draft_schema(draft: &mut Value) {
    if draft.get("version").is_none() {
        draft["version"] = json!("1");
    }
    for key in ["storyTitle", "summary", "storyPlot"] {
        if draft.get(key).and_then(Value::as_str).is_none() {
            draft[key] = json!("");
        }
    }
    if draft.get("memoryAnchors").and_then(Value::as_array).is_none() {
        draft["memoryAnchors"] = json!([]);
    }
    if draft.get("elementGroups").and_then(Value::as_array).is_none() {
        draft["elementGroups"] = json!([]);
    }
    ensure_draft_nodes(draft);
}

fn initialize_storylock_core_window(core: &StoryLockCoreApp, package_dir: &Path) {
    let vault = read_storylock_vault_payload(package_dir);
    let draft = storylock_author_draft_from_vault(&vault);
    let templates = storylock_templates_from_vault(&vault);
    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    core.set_core_data_dir(SharedString::from(package_dir.display().to_string()));
    core.set_draft_file_path(SharedString::from("vault.stlk"));
    core.set_manifest_file_path(SharedString::from("package-manifest.json"));
    core.set_encrypted_vault_path(SharedString::from("vault.stlk"));
    core.set_resource_catalog_path(SharedString::from("resource-catalog.json"));
    core.set_learning_policy_path(SharedString::from("learning-policy.json"));
    core.set_export_package_dir(SharedString::from(default_storylock_export_dir(package_dir).display().to_string()));
    core.set_story_title(json_string(&draft, &["storyTitle"]));
    core.set_story_summary(json_string(&draft, &["summary"]));
    core.set_story_plot(json_string(&draft, &["storyPlot"]));
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
    core.set_node_overview(SharedString::from(format_node_overview(&draft)));
    set_question_overview_titles(core, &draft);
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
        core.set_resource_group(resource_group_from_catalog_resource(resource));
        core.set_resource_bindings(SharedString::from(format_bindings(resource)));
        core.set_object_meta(SharedString::from(format_object_meta(resource)));
    }
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        core.get_resource_group().as_str(),
    )));
    if let Some(item) = templates
        .get("loginSites")
        .and_then(|value| value.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
    {
        core.set_template_id(json_string(item, &["templateId"]));
        core.set_template_display_name(json_string(item, &["displayName"]));
    }
    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(package_dir)));
    core.set_export_preview(SharedString::from(build_export_preview(package_dir)));
    load_learning_policy_into_window(core, package_dir);
    load_learning_node_into_window(core, package_dir, 0);
}

fn wire_storylock_core_callbacks(
    core: &StoryLockCoreApp,
    package_dir: std::path::PathBuf,
    core_window_slot: Rc<RefCell<Option<StoryLockCoreApp>>>,
    on_closed: Rc<dyn Fn()>,
    host_port: u16,
) {
    let learning_passed = Rc::new(RefCell::new(vec![false; 24]));
    let answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>> = Rc::new(RefCell::new(None));
    let settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>> =
        Rc::new(RefCell::new(None));
    let weak = core.as_weak();
    let close_slot = Rc::clone(&core_window_slot);
    let on_button_closed = Rc::clone(&on_closed);
    core.on_close_requested(move || {
        if let Some(core) = weak.upgrade() {
            let _ = core.hide();
        }
        *close_slot.borrow_mut() = None;
        on_button_closed();
    });

    let weak = core.as_weak();
    let window_close_slot = Rc::clone(&core_window_slot);
    let on_window_closed = Rc::clone(&on_closed);
    core.window().on_close_requested(move || {
        if let Some(core) = weak.upgrade() {
            let _ = core.hide();
        }
        *window_close_slot.borrow_mut() = None;
        on_window_closed();
        slint::CloseRequestResponse::HideWindow
    });

    let weak = core.as_weak();
    let settings_dir = package_dir.clone();
    let settings_dialog_for_open = Rc::clone(&settings_dialog);
    core.on_open_core_settings(move || {
        if let Some(core) = weak.upgrade() {
            open_storylock_core_settings_dialog(
                &core,
                &settings_dir,
                Rc::clone(&settings_dialog_for_open),
            );
        }
    });

    let weak = core.as_weak();
    let browse_fallback_dir = package_dir.clone();
    core.on_browse_core_data_dir(move || {
        if let Some(core) = weak.upgrade() {
            let current_dir = storylock_core_package_dir_from_window(&core, &browse_fallback_dir);
            let mut dialog = rfd::FileDialog::new();
            if current_dir.exists() {
                dialog = dialog.set_directory(&current_dir);
            }
            if let Some(selected_dir) = dialog.pick_folder() {
                match ensure_storylock_core_package(&selected_dir) {
                    Ok(()) => {
                        initialize_storylock_core_window(&core, &selected_dir);
                        core.set_config_status(SharedString::from(
                            "StoryLock Core workspace loaded from selected directory.",
                        ));
                    }
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!(
                            "Workspace load failed: {error}"
                        )));
                    }
                }
            }
        }
    });

    let weak = core.as_weak();
    let export_browse_fallback_dir = package_dir.clone();
    core.on_browse_export_package_dir(move || {
        if let Some(core) = weak.upgrade() {
            let current = core.get_export_package_dir();
            let current_trimmed = current.as_str().trim();
            let mut dialog = rfd::FileDialog::new();
            if !current_trimmed.is_empty() {
                let current_path = std::path::PathBuf::from(current_trimmed);
                if current_path.exists() {
                    dialog = dialog.set_directory(current_path);
                }
            } else {
                dialog = dialog.set_directory(default_storylock_export_dir(&export_browse_fallback_dir));
            }
            if let Some(selected_dir) = dialog.pick_folder() {
                core.set_export_package_dir(SharedString::from(selected_dir.display().to_string()));
                core.set_config_status(SharedString::from(
                    "Export directory selected for the next package export.",
                ));
            }
        }
    });

    let weak = core.as_weak();
    let temp_draft_dir = package_dir.clone();
    let temp_draft_learning_passed = Rc::clone(&learning_passed);
    core.on_save_temp_draft(move || {
        if let Some(core) = weak.upgrade() {
            if core.get_temp_draft_cooling() {
                return;
            }
            core.set_temp_draft_cooling(true);
            core.set_temp_draft_label(SharedString::from(if core.get_language().as_str() == "zh" {
                "已暂存"
            } else {
                "Saved"
            }));
            let result = ensure_storylock_core_package_dir_from_window(&core, &temp_draft_dir)
                .and_then(|package_dir| save_temp_draft_from_window(&core, &package_dir));
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
            let weak_for_timer = core.as_weak();
            slint::Timer::single_shot(Duration::from_millis(900), move || {
                if let Some(core) = weak_for_timer.upgrade() {
                    core.set_temp_draft_cooling(false);
                    core.set_temp_draft_label(SharedString::from(
                        if core.get_language().as_str() == "zh" {
                            "暂存草稿"
                        } else {
                            "Save Draft"
                        },
                    ));
                }
            });
        }
    });

    let weak = core.as_weak();
    let previous_node_dir = package_dir.clone();
    let previous_learning_passed = Rc::clone(&learning_passed);
    core.on_previous_node(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &previous_node_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                        return;
                    }
                };
            if let Err(error) = save_current_node_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &previous_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = core.get_node_index().saturating_sub(1);
            load_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_node_dir = package_dir.clone();
    let next_learning_passed = Rc::clone(&learning_passed);
    core.on_next_node(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &next_node_dir)
            {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                    return;
                }
            };
            if let Err(error) = save_current_node_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                return;
            }
            reset_learning_gate(
                &core,
                &next_learning_passed,
                "Question navigation saved a draft. Run learning test again before export.",
            );
            let next_index = (core.get_node_index() + 1).min(23);
            load_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let select_node_dir = package_dir.clone();
    let select_learning_passed = Rc::clone(&learning_passed);
    let answer_editor_for_select = Rc::clone(&answer_editor);
    core.on_select_node(move |value| {
        if let Some(core) = weak.upgrade() {
            let package_dir =
                match ensure_storylock_core_package_dir_from_window(&core, &select_node_dir) {
                    Ok(package_dir) => package_dir,
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!("Save failed: {error}")));
                        return;
                    }
                };
            if let Err(error) = save_current_node_from_window(&core, &package_dir) {
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
            load_node_into_window(&core, &package_dir, selected_index);
            open_answer_editor_dialog(&core, &package_dir, Rc::clone(&answer_editor_for_select));
        }
    });

    let weak = core.as_weak();
    let group_dir = package_dir.clone();
    core.on_select_resource_group(move |value| {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &group_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Workspace load failed: {error}"
                    )));
                    return;
                }
            };
            let group = normalize_resource_group(value.as_str());
            core.set_resource_group(SharedString::from(group.clone()));
            let catalog = read_json_or_default(
                &storylock_core_catalog_path(&package_dir),
                default_resource_catalog_json(),
            );
            if let Some(resource) = first_resource_for_group(&catalog, &group) {
                core.set_resource_id(json_string(resource, &["resourceId"]));
                core.set_resource_kind(json_string(resource, &["resourceKind"]));
                core.set_provider_id(json_string(resource, &["providerId"]));
                core.set_display_name(json_string(resource, &["displayName"]));
                core.set_resource_bindings(SharedString::from(format_bindings(resource)));
                core.set_object_meta(SharedString::from(format_object_meta(resource)));
            }
            core.set_protected_object_list(SharedString::from(format_protected_object_list(
                &catalog,
                &group,
            )));
            core.set_active_page(2);
        }
    });

    let weak = core.as_weak();
    let resource_dir = package_dir.clone();
    let resource_learning_passed = Rc::clone(&learning_passed);
    core.on_save_resource(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &resource_dir)
                .and_then(|package_dir| save_resource_from_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &resource_learning_passed,
                "Managed object changed. Run learning test again before export.",
            );
            set_core_status(&core, result, "Resource catalog saved locally.");
        }
    });

    let weak = core.as_weak();
    let template_dir = package_dir.clone();
    let template_learning_passed = Rc::clone(&learning_passed);
    core.on_save_template(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &template_dir)
                .and_then(|package_dir| save_template_from_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &template_learning_passed,
                "Template changed. Run learning test again before export.",
            );
            set_core_status(&core, result, "Story draft template saved locally.");
        }
    });

    let weak = core.as_weak();
    let apply_template_dir = package_dir.clone();
    let apply_template_learning_passed = Rc::clone(&learning_passed);
    core.on_apply_template(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &apply_template_dir)
                .and_then(|package_dir| apply_story_draft_template_to_window(&core, &package_dir));
            reset_learning_gate(
                &core,
                &apply_template_learning_passed,
                "Story template loaded. Run learning test again before export.",
            );
            set_core_status(&core, result, "Story draft template loaded into current UI.");
        }
    });

    let weak = core.as_weak();
    let candidate_dir = package_dir.clone();
    core.on_pull_template_candidates(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &candidate_dir)
                .and_then(|package_dir| pull_story_template_candidates_into_vault(&core, &package_dir, host_port));
            match result {
                Ok(message) => {
                    core.set_candidate_template_status(SharedString::from(message));
                    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(
                        &storylock_core_package_dir_from_window(&core, &candidate_dir),
                    )));
                    core.set_config_status(SharedString::from(
                        "Story template candidates pulled into local StoryLock templates.",
                    ));
                }
                Err(error) => {
                    core.set_candidate_template_status(SharedString::from(format!(
                        "Candidate pull failed: {error}"
                    )));
                    core.set_config_status(SharedString::from(format!(
                        "Candidate pull failed: {error}"
                    )));
                }
            }
        }
    });

    let weak = core.as_weak();
    let refresh_dir = package_dir.clone();
    core.on_refresh_export(move || {
        if let Some(core) = weak.upgrade() {
            match ensure_storylock_core_package_dir_from_window(&core, &refresh_dir) {
                Ok(package_dir) => {
                    core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
                    core.set_config_status(SharedString::from(
                        "Export preview refreshed from local StoryLock Core package.",
                    ));
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Export preview failed: {error}"
                    )));
                }
            }
        }
    });

    let weak = core.as_weak();
    let learning_policy_dir = package_dir.clone();
    core.on_save_learning_policy(move || {
        if let Some(core) = weak.upgrade() {
            let result = ensure_storylock_core_package_dir_from_window(&core, &learning_policy_dir)
                .and_then(|package_dir| save_learning_policy_from_window(&core, &package_dir));
            match result {
                Ok(()) => {
                    core.set_config_status(SharedString::from(
                        "Learning policy saved to learning-policy.json for Host execution.",
                    ));
                }
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Learning policy save failed: {error}"
                    )));
                }
            }
        }
    });

    let weak = core.as_weak();
    let learning_dir = package_dir.clone();
    let run_learning_passed = Rc::clone(&learning_passed);
    core.on_run_learning(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &learning_dir)
            {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_result(SharedString::from(
                        "Pre-export test blocked because the workspace is invalid.",
                    ));
                    core.set_config_status(SharedString::from(format!(
                        "Workspace load failed: {error}"
                    )));
                    return;
                }
            };
            if let Err(error) = save_learning_policy_from_window(&core, &package_dir) {
                core.set_export_ready(false);
                core.set_learning_result(SharedString::from(
                    "Pre-export test blocked because the learning policy is invalid.",
                ));
                core.set_config_status(SharedString::from(format!(
                    "Learning policy save failed: {error}"
                )));
                return;
            }
            match run_export_learning_test(&package_dir) {
                Ok(report) => {
                    run_learning_passed.borrow_mut().fill(true);
                    core.set_export_ready(true);
                    core.set_learning_result(SharedString::from(report.clone()));
                    core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
                    core.set_learning_status(SharedString::from(
                        "Pre-export test passed. Export is enabled.",
                    ));
                    core.set_config_status(SharedString::from(report));
                }
                Err(error) => {
                    run_learning_passed.borrow_mut().fill(false);
                    core.set_export_ready(false);
                    core.set_learning_result(SharedString::from(
                        "Pre-export test failed. Fix the local StoryLock data and run the test again.",
                    ));
                    core.set_learning_status(SharedString::from(format!(
                        "Pre-export test failed: {error}"
                    )));
                    core.set_config_status(SharedString::from(
                        "Export is blocked until the pre-export test passes.",
                    ));
                }
            }
        }
    });

    let weak = core.as_weak();
    let previous_learning_dir = package_dir.clone();
    core.on_learning_previous(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &previous_learning_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_learning_status(SharedString::from(format!(
                        "Learning load failed: {error}"
                    )));
                    return;
                }
            };
            let next_index = core.get_learning_index().saturating_sub(1);
            load_learning_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let next_learning_dir = package_dir.clone();
    core.on_learning_next(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &next_learning_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_learning_status(SharedString::from(format!(
                        "Learning load failed: {error}"
                    )));
                    return;
                }
            };
            let next_index = (core.get_learning_index() + 1).min(23);
            load_learning_node_into_window(&core, &package_dir, next_index);
        }
    });

    let weak = core.as_weak();
    let check_learning_dir = package_dir.clone();
    let check_learning_passed = Rc::clone(&learning_passed);
    core.on_check_learning_current(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &check_learning_dir) {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_export_ready(false);
                    core.set_learning_status(SharedString::from(format!(
                        "Learning check failed: {error}"
                    )));
                    return;
                }
            };
            match check_learning_current(&core, &package_dir, &check_learning_passed) {
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
    core.on_export_package(move || {
        if let Some(core) = weak.upgrade() {
            let package_dir = match ensure_storylock_core_package_dir_from_window(&core, &export_dir)
            {
                Ok(package_dir) => package_dir,
                Err(error) => {
                    core.set_config_status(SharedString::from(format!(
                        "Export blocked. Workspace is invalid: {error}"
                    )));
                    return;
                }
            };
            if let Err(error) = save_learning_policy_from_window(&core, &package_dir) {
                core.set_config_status(SharedString::from(format!(
                    "Export blocked. Learning policy is invalid: {error}"
                )));
                return;
            }
            if !core.get_export_ready() {
                core.set_config_status(SharedString::from(
                    "Export blocked. Run the pre-export test successfully first.",
                ));
                return;
            }
            let export_dir = storylock_export_dir_from_window(&core, &package_dir);
            match export_storylock_package_to(&package_dir, &export_dir) {
                Ok(path) => {
                    core.set_export_preview(SharedString::from(build_export_preview(&package_dir)));
                    core.set_config_status(SharedString::from(format!(
                        "Export complete. Managed key package replaced at {}",
                        path.display()
                    )));
                    core.set_learning_status(SharedString::from(
                        "Pre-export test passed. Encrypted export completed.",
                    ));
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

fn open_answer_editor_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>>,
) {
    if answer_editor.borrow().is_none() {
        match AnswerEditorDialog::new() {
            Ok(dialog) => {
                wire_answer_editor_callbacks(&dialog, core.as_weak(), package_dir.to_path_buf());
                *answer_editor.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Answer editor failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = answer_editor.borrow().as_ref() {
        copy_core_question_to_answer_editor(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Answer editor failed to show: {error}"
            )));
        }
    }
}

fn open_storylock_core_settings_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    if settings_dialog.borrow().is_none() {
        match StoryLockCoreSettingsDialog::new() {
            Ok(dialog) => {
                wire_storylock_core_settings_callbacks(
                    &dialog,
                    core.as_weak(),
                    package_dir.to_path_buf(),
                    Rc::clone(&settings_dialog),
                );
                *settings_dialog.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Settings failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = settings_dialog.borrow().as_ref() {
        copy_core_settings_to_dialog(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Settings failed to show: {error}"
            )));
        }
    }
}

fn wire_storylock_core_settings_callbacks(
    dialog: &StoryLockCoreSettingsDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    let weak = dialog.as_weak();
    let close_slot = Rc::clone(&settings_dialog);
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        *close_slot.borrow_mut() = None;
    });

    let weak = dialog.as_weak();
    let core_for_language = core_weak.clone();
    dialog.on_language_changed(move |language| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_language.upgrade()) {
            core.set_language(language);
            copy_core_settings_to_dialog(&core, &dialog);
        }
    });

    let weak = dialog.as_weak();
    let core_for_browse = core_weak.clone();
    let browse_fallback_dir = package_dir.clone();
    dialog.on_browse_core_data_dir(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_browse.upgrade()) {
            copy_dialog_settings_to_core(&dialog, &core);
            let current_dir = storylock_core_package_dir_from_window(&core, &browse_fallback_dir);
            let mut file_dialog = rfd::FileDialog::new();
            if current_dir.exists() {
                file_dialog = file_dialog.set_directory(&current_dir);
            }
            if let Some(selected_dir) = file_dialog.pick_folder() {
                match ensure_storylock_core_package(&selected_dir) {
                    Ok(()) => {
                        initialize_storylock_core_window(&core, &selected_dir);
                        core.set_config_status(SharedString::from(
                            "StoryLock Core workspace loaded from selected directory.",
                        ));
                        copy_core_settings_to_dialog(&core, &dialog);
                    }
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!(
                            "Workspace load failed: {error}"
                        )));
                    }
                }
            }
        }
    });

}

fn wire_answer_editor_callbacks(
    dialog: &AnswerEditorDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
) {
    let weak = dialog.as_weak();
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
    });

    let weak = dialog.as_weak();
    let core_for_save = core_weak.clone();
    let save_dir = package_dir.clone();
    dialog.on_save_requested(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_save.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            match save_current_node_from_window(&core, &save_dir) {
                Ok(()) => core.set_config_status(SharedString::from(
                    "Answer editor saved current question.",
                )),
                Err(error) => core.set_config_status(SharedString::from(format!(
                    "Answer editor save failed: {error}"
                ))),
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_previous = core_weak.clone();
    let previous_dir = package_dir.clone();
    dialog.on_previous_node(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_previous.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &previous_dir).is_ok() {
                let next_index = core.get_node_index().saturating_sub(1);
                load_node_into_window(&core, &previous_dir, next_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_next = core_weak.clone();
    let next_dir = package_dir.clone();
    dialog.on_next_node(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_next.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &next_dir).is_ok() {
                let next_index = (core.get_node_index() + 1).min(23);
                load_node_into_window(&core, &next_dir, next_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_select = core_weak;
    let select_dir = package_dir;
    dialog.on_select_node(move |value| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_select.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &select_dir).is_ok() {
                let selected_index = value
                    .parse::<i32>()
                    .ok()
                    .map(|number| number - 1)
                    .unwrap_or_else(|| core.get_node_index());
                load_node_into_window(&core, &select_dir, selected_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });
}

fn copy_core_settings_to_dialog(core: &StoryLockCoreApp, dialog: &StoryLockCoreSettingsDialog) {
    dialog.set_language(core.get_language());
    dialog.set_core_data_dir(core.get_core_data_dir());
}

fn copy_dialog_settings_to_core(dialog: &StoryLockCoreSettingsDialog, core: &StoryLockCoreApp) {
    core.set_language(dialog.get_language());
    core.set_core_data_dir(dialog.get_core_data_dir());
}

fn copy_core_question_to_answer_editor(core: &StoryLockCoreApp, dialog: &AnswerEditorDialog) {
    dialog.set_language(core.get_language());
    dialog.set_selected_question(core.get_selected_question());
    dialog.set_question_text(core.get_question_text());
    dialog.set_answer_1(core.get_answer_1());
    dialog.set_answer_1_state(core.get_answer_1_state());
    dialog.set_answer_2(core.get_answer_2());
    dialog.set_answer_2_state(core.get_answer_2_state());
    dialog.set_answer_3(core.get_answer_3());
    dialog.set_answer_3_state(core.get_answer_3_state());
    dialog.set_answer_4(core.get_answer_4());
    dialog.set_answer_4_state(core.get_answer_4_state());
    dialog.set_answer_5(core.get_answer_5());
    dialog.set_answer_5_state(core.get_answer_5_state());
    dialog.set_answer_6(core.get_answer_6());
    dialog.set_answer_6_state(core.get_answer_6_state());
    dialog.set_answer_7(core.get_answer_7());
    dialog.set_answer_7_state(core.get_answer_7_state());
    dialog.set_answer_8(core.get_answer_8());
    dialog.set_answer_8_state(core.get_answer_8_state());
    dialog.set_answer_9(core.get_answer_9());
    dialog.set_answer_9_state(core.get_answer_9_state());
}

fn copy_answer_editor_to_core(dialog: &AnswerEditorDialog, core: &StoryLockCoreApp) {
    core.set_selected_question(dialog.get_selected_question());
    core.set_question_text(dialog.get_question_text());
    core.set_answer_1(dialog.get_answer_1());
    core.set_answer_1_state(dialog.get_answer_1_state());
    core.set_answer_2(dialog.get_answer_2());
    core.set_answer_2_state(dialog.get_answer_2_state());
    core.set_answer_3(dialog.get_answer_3());
    core.set_answer_3_state(dialog.get_answer_3_state());
    core.set_answer_4(dialog.get_answer_4());
    core.set_answer_4_state(dialog.get_answer_4_state());
    core.set_answer_5(dialog.get_answer_5());
    core.set_answer_5_state(dialog.get_answer_5_state());
    core.set_answer_6(dialog.get_answer_6());
    core.set_answer_6_state(dialog.get_answer_6_state());
    core.set_answer_7(dialog.get_answer_7());
    core.set_answer_7_state(dialog.get_answer_7_state());
    core.set_answer_8(dialog.get_answer_8());
    core.set_answer_8_state(dialog.get_answer_8_state());
    core.set_answer_9(dialog.get_answer_9());
    core.set_answer_9_state(dialog.get_answer_9_state());
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
    draft["storyPlot"] = json!(core.get_story_plot().to_string());
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
    core.set_node_overview(SharedString::from(format_node_overview(&draft)));
    set_question_overview_titles(core, &draft);
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

fn set_question_overview_titles(core: &StoryLockCoreApp, draft: &Value) {
    let nodes = draft.get("nodes").and_then(Value::as_array);
    let title = |index: usize| -> SharedString {
        let label = nodes
            .and_then(|items| items.get(index))
            .map(|node| {
                let question = node
                    .get("question")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if question.is_empty() {
                    json_string(node, &["title"]).to_string()
                } else {
                    question.to_string()
                }
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("Q{}", index + 1));
        SharedString::from(format!("{}. {}", index + 1, label))
    };

    core.set_question_1(title(0));
    core.set_question_2(title(1));
    core.set_question_3(title(2));
    core.set_question_4(title(3));
    core.set_question_5(title(4));
    core.set_question_6(title(5));
    core.set_question_7(title(6));
    core.set_question_8(title(7));
    core.set_question_9(title(8));
    core.set_question_10(title(9));
    core.set_question_11(title(10));
    core.set_question_12(title(11));
    core.set_question_13(title(12));
    core.set_question_14(title(13));
    core.set_question_15(title(14));
    core.set_question_16(title(15));
    core.set_question_17(title(16));
    core.set_question_18(title(17));
    core.set_question_19(title(18));
    core.set_question_20(title(19));
    core.set_question_21(title(20));
    core.set_question_22(title(21));
    core.set_question_23(title(22));
    core.set_question_24(title(23));
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
            "Question {} did not match. Learning progress: {} / 24. Re-check the 9 check boxes.",
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

fn format_node_overview(draft: &Value) -> String {
    draft
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    let title = node.get("title").and_then(Value::as_str).unwrap_or("Question");
                    let question = node.get("question").and_then(Value::as_str).unwrap_or("");
                    let answer_count = node
                        .get("answerOptionsLocalOnly")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0);
                    format!("{:02}. {} | {} | {} answers", index + 1, title, question, answer_count)
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|| "No question overview is available.".to_string())
}

fn normalize_resource_group(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" | "普通授权对象" | "普通" => "normal".to_string(),
        "private" | "私密对象" | "私密" => "private".to_string(),
        "secret" | "机密对象" | "机密" => "secret".to_string(),
        _ => "normal".to_string(),
    }
}

fn resource_group_from_catalog_resource(resource: &Value) -> SharedString {
    let group = resource
        .get("resourceGroup")
        .and_then(Value::as_str)
        .or_else(|| {
            resource
                .get("bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| bindings.first())
                .and_then(|binding| binding.get("objectMeta"))
                .and_then(|meta| meta.get("sensitivity"))
                .and_then(Value::as_str)
        })
        .unwrap_or("normal");
    SharedString::from(normalize_resource_group(group))
}

fn format_protected_object_list(catalog: &Value, selected_group: &str) -> String {
    let selected_group = normalize_resource_group(selected_group);
    let mut items = Vec::new();
    if let Some(resources) = catalog.get("resources").and_then(Value::as_array) {
        for resource in resources {
            let resource_id = resource
                .get("resourceId")
                .and_then(Value::as_str)
                .unwrap_or("resource");
            let display_name = resource
                .get("displayName")
                .and_then(Value::as_str)
                .unwrap_or(resource_id);
            let resource_group = resource
                .get("resourceGroup")
                .and_then(Value::as_str)
                .unwrap_or("normal");
            let Some(bindings) = resource.get("bindings").and_then(Value::as_array) else {
                continue;
            };
            for binding in bindings {
                let meta = binding.get("objectMeta").unwrap_or(&Value::Null);
                let group = normalize_resource_group(
                    meta.get("sensitivity")
                        .and_then(Value::as_str)
                        .unwrap_or(resource_group),
                );
                if group != selected_group {
                    continue;
                }
                items.push(format!(
                    "{}. {} | resource={} | object={} | kind={} | level={}",
                    items.len() + 1,
                    display_name,
                    resource_id,
                    binding.get("objectId").and_then(Value::as_str).unwrap_or(""),
                    meta.get("objectKind").and_then(Value::as_str).unwrap_or("secret"),
                    group
                ));
            }
        }
    }
    if items.is_empty() {
        "No protected objects in this level yet.".to_string()
    } else {
        items.join("\n")
    }
}

fn first_resource_for_group<'a>(catalog: &'a Value, selected_group: &str) -> Option<&'a Value> {
    let selected_group = normalize_resource_group(selected_group);
    catalog
        .get("resources")
        .and_then(Value::as_array)?
        .iter()
        .find(|resource| {
            let resource_group = resource_group_from_catalog_resource(resource);
            resource_group.as_str() == selected_group
        })
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
    let sensitivity = normalize_resource_group(core.get_resource_group().as_str());
    let catalog = json!({
        "version": "1",
        "resources": [{
            "resourceId": core.get_resource_id().to_string(),
            "resourceKind": core.get_resource_kind().to_string(),
            "providerId": core.get_provider_id().to_string(),
            "displayName": core.get_display_name().to_string(),
            "resourceGroup": sensitivity.clone(),
            "bindings": [
                {
                    "role": "protected_object",
                    "objectId": object_id,
                    "objectMeta": {
                        "objectKind": object_kind,
                        "encoding": "secret",
                        "sensitivity": sensitivity.clone(),
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
    core.set_protected_object_list(SharedString::from(format_protected_object_list(
        &catalog,
        core.get_resource_group().as_str(),
    )));
    Ok(())
}

fn save_template_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    save_story_draft_template_from_window(core, package_dir)
}

fn save_story_draft_template_from_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let mut draft = read_effective_author_draft(package_dir);
    draft["version"] = json!("1");
    draft["templateId"] = json!(core.get_template_id().to_string());
    draft["storyTitle"] = json!(core.get_story_title().to_string());
    draft["summary"] = json!(core.get_story_summary().to_string());
    draft["storyPlot"] = json!(core.get_story_plot().to_string());
    draft["memoryAnchors"] = json!(split_list(core.get_memory_anchors().as_str(), "/"));
    draft["elementGroups"] = json!(split_list(core.get_element_group().as_str(), ","));
    write_current_node_to_draft(core, &mut draft);
    normalize_author_draft_schema(&mut draft);

    let mut vault = read_storylock_vault_payload(package_dir);
    let mut templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    if !templates.is_object() {
        templates = default_story_draft_templates_json();
    }
    let draft_template_id = draft
        .get("templateId")
        .and_then(Value::as_str)
        .unwrap_or("current-author-draft")
        .to_string();
    let mut items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    items.retain(|item| {
        item.get("templateId").and_then(Value::as_str) != Some(draft_template_id.as_str())
    });
    items.insert(0, draft.clone());
    templates["schemaVersion"] = json!("storylock-story-draft-templates-v1");
    templates["defaultTemplateId"] = json!(draft_template_id);
    templates["items"] = Value::Array(items);
    vault["storyDraftTemplates"] = templates;
    vault["pendingAuthorDraft"] = draft;
    save_storylock_vault_payload(package_dir, vault)?;
    core.set_template_display_name(json_string(&read_effective_author_draft(package_dir), &["storyTitle"]));
    core.set_template_bindings(SharedString::from(format_story_draft_template_summary(package_dir)));
    Ok(())
}

fn apply_story_draft_template_to_window(core: &StoryLockCoreApp, package_dir: &Path) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    let mut draft = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or_else(default_author_draft_json);
    normalize_author_draft_schema(&mut draft);
    vault["pendingAuthorDraft"] = draft;
    save_storylock_vault_payload(package_dir, vault)?;
    initialize_storylock_core_window(core, package_dir);
    Ok(())
}

fn pull_story_template_candidates_into_vault(
    _core: &StoryLockCoreApp,
    package_dir: &Path,
    host_port: u16,
) -> Result<String> {
    let url = format!("http://127.0.0.1:{host_port}/story-template/candidates?limit=10");
    let response = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(url)
        .send()?;
    if !response.status().is_success() {
        anyhow::bail!("Host returned HTTP {}", response.status());
    }
    let payload: Value = response.json()?;
    let candidates = payload
        .get("result")
        .and_then(|result| result.get("candidates"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if candidates.is_empty() {
        return Ok("No queued story template candidates.".to_string());
    }

    let mut vault = read_storylock_vault_payload(package_dir);
    merge_builtin_story_draft_templates(&mut vault);
    let mut templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    let mut items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut imported = 0usize;
    for candidate in candidates {
        let draft = story_draft_from_candidate(&candidate);
        let template_id = draft
            .get("templateId")
            .and_then(Value::as_str)
            .unwrap_or("host-candidate")
            .to_string();
        if items
            .iter()
            .any(|item| item.get("templateId").and_then(Value::as_str) == Some(template_id.as_str()))
        {
            continue;
        }
        items.push(draft);
        imported += 1;
    }
    templates["schemaVersion"] = json!("storylock-story-draft-templates-v1");
    templates["items"] = Value::Array(items);
    vault["storyDraftTemplates"] = templates;
    save_storylock_vault_payload(package_dir, vault)?;
    Ok(format!("Pulled {imported} new candidate template(s)."))
}

fn story_draft_from_candidate(candidate: &Value) -> Value {
    let framework = candidate.get("framework").unwrap_or(candidate);
    let candidate_id = candidate
        .get("candidateId")
        .and_then(Value::as_str)
        .unwrap_or("host-candidate");
    let title = framework
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Host candidate story");
    let summary = framework
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("A Host-generated candidate framework waiting for manual StoryLock editing.");
    let plot = framework
        .get("storyPlot")
        .and_then(Value::as_str)
        .unwrap_or("This candidate was queued by Host. StoryLock should manually edit it into a private 24-question story before export.");
    let anchors = framework
        .get("memoryAnchors")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .take(8)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec!["host candidate", "private clue", "manual edit"]);
    let mut draft = story_template_author_draft_json(
        candidate_id,
        title,
        summary,
        plot,
        &anchors,
    );
    draft["source"] = json!({
        "kind": "host-story-template-candidate",
        "candidateId": candidate_id,
        "hostInvokesStoryLock": false
    });
    draft
}

fn format_story_draft_template_summary(package_dir: &Path) -> String {
    let vault = read_storylock_vault_payload(package_dir);
    let templates = vault
        .get("storyDraftTemplates")
        .cloned()
        .unwrap_or_else(default_story_draft_templates_json);
    let items = templates
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        return "No story draft template is stored.".to_string();
    }
    items
        .iter()
        .enumerate()
        .map(|(index, draft)| {
            let node_count = draft
                .get("nodes")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            format!(
                "{}. templateId={}\nstoryTitle={}\nsummary={}\nnodes={}\nformat=authorDraft\n",
                index + 1,
                draft
                    .get("templateId")
                    .and_then(Value::as_str)
                    .unwrap_or("current-author-draft"),
                draft
                    .get("storyTitle")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                draft
                    .get("summary")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                node_count
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    let pending_state = if read_storylock_vault(package_dir)
        .get("pendingAuthorDraft")
        .is_some_and(|value| !value.is_null())
    {
        "pending temporary draft exists; export will promote it inside vault.stlk"
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
        "identity-package/\n  vault.stlk\n  package-manifest.json\n  resource-catalog.json\n  learning-policy.json\n\nLocal path: {}\ntemporaryDraft={pending_state}\nresources={resources}\npermissionObjects={permission_objects}\npreflight={status}\nerrors:\n{errors}\n\nStoryLock UI internal export preview only; Yian Host reads learning-policy.json for retention scheduling, but does not read drafts, vault files, raw story, answers, passwords, private keys, or signingKeyBytes.",
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
        "Pre-export test passed. StoryLock questions and related package data are ready for encrypted export; verified {total_correct} correct answer markers."
    ))
}

fn default_storylock_export_dir(package_dir: &Path) -> std::path::PathBuf {
    package_dir
        .parent()
        .map(|parent| parent.join("storylock-managed-key-package"))
        .unwrap_or_else(|| std::path::PathBuf::from("storylock-managed-key-package"))
}

fn storylock_export_dir_from_window(
    core: &StoryLockCoreApp,
    package_dir: &Path,
) -> std::path::PathBuf {
    let configured = core.get_export_package_dir().trim().to_string();
    if configured.is_empty() {
        default_storylock_export_dir(package_dir)
    } else {
        std::path::PathBuf::from(configured)
    }
}

#[cfg(test)]
fn export_storylock_package(package_dir: &Path) -> Result<std::path::PathBuf> {
    let export_dir = default_storylock_export_dir(package_dir);
    export_storylock_package_to(package_dir, &export_dir)
}

fn export_storylock_package_to(
    package_dir: &Path,
    export_dir: &Path,
) -> Result<std::path::PathBuf> {
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
    if export_dir.exists() {
        fs::remove_dir_all(export_dir)?;
    }
    copy_dir_recursive(package_dir, export_dir)?;
    fs::write(
        export_dir.join("EXPORT_STATUS.txt"),
        format!(
            "Exported from StoryLock Core after learning test.\nSource: {}\nExportedAt: {}\nTemporaryDraftCleared: true\n",
            package_dir.display(),
            ui_now_timestamp()
        ),
    )?;
    remove_pending_author_draft(package_dir)?;
    Ok(export_dir.to_path_buf())
}

fn promote_pending_author_draft(package_dir: &Path) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    if let Some(pending) = vault.get("pendingAuthorDraft").cloned() {
        if !pending.is_null() {
            vault["authorDraft"] = pending;
            vault["pendingAuthorDraft"] = Value::Null;
            save_storylock_vault_payload(package_dir, vault)?;
        }
    }
    Ok(())
}

fn remove_pending_author_draft(package_dir: &Path) -> Result<()> {
    let mut vault = read_storylock_vault_payload(package_dir);
    if vault
        .get("pendingAuthorDraft")
        .is_some_and(|pending| !pending.is_null())
    {
        vault["pendingAuthorDraft"] = Value::Null;
        save_storylock_vault_payload(package_dir, vault)?;
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
    for required_file in required_storylock_package_files() {
        if !package_dir.join(required_file).exists() {
            errors.push(PreflightIssue {
                code: "SL_PKG_REQUIRED_FILE_MISSING",
                path: "$.files".to_string(),
                message: format!("missing required file: {required_file}"),
            });
        }
    }

    let manifest = read_json_or_default(&storylock_core_manifest_path(package_dir), Value::Null);
    if let Some(files) = manifest.get("files").and_then(Value::as_array) {
        for required_file in required_storylock_package_files() {
            if !files
                .iter()
                .any(|item| item.as_str() == Some(required_file))
            {
                errors.push(PreflightIssue {
                    code: "SL_PKG_REQUIRED_FILE_MISSING",
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

    let policy = read_learning_policy(package_dir);
    validate_learning_policy(&policy, &mut errors);

    let vault = read_storylock_vault_payload(package_dir);
    let draft = storylock_author_draft_from_vault(&vault);
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

    validate_story_draft_templates(&vault, &mut errors);

    let catalog = read_json_or_default(
        &storylock_core_catalog_path(package_dir),
        default_resource_catalog_json(),
    );
    let role_index = build_catalog_role_index(&catalog, &mut errors);
    for (file_name, bundle) in storylock_templates_from_vault(&vault)
        .as_object()
        .cloned()
        .unwrap_or_default()
    {
        validate_template_references(&file_name, &bundle, &role_index, &mut errors);
    }

    PreflightResult { errors }
}

fn validate_story_draft_templates(vault: &Value, errors: &mut Vec<PreflightIssue>) {
    let Some(items) = vault
        .get("storyDraftTemplates")
        .and_then(|templates| templates.get("items"))
        .and_then(Value::as_array)
    else {
        errors.push(PreflightIssue {
            code: "SL_STORY_TEMPLATE_MISSING",
            path: "$.storyDraftTemplates.items".to_string(),
            message: "story draft templates must be stored as authorDraft-compatible items".to_string(),
        });
        return;
    };

    for (index, item) in items.iter().enumerate() {
        for field in ["storyTitle", "summary", "storyPlot"] {
            if item.get(field).and_then(Value::as_str).unwrap_or("").is_empty() {
                errors.push(PreflightIssue {
                    code: "SL_STORY_TEMPLATE_FIELD_MISSING",
                    path: format!("$.storyDraftTemplates.items[{index}].{field}"),
                    message: format!("story draft template must include {field}"),
                });
            }
        }
        match item.get("nodes").and_then(Value::as_array) {
            Some(nodes) if nodes.len() == 24 => {}
            Some(nodes) => errors.push(PreflightIssue {
                code: "SL_STORY_TEMPLATE_NODE_COUNT",
                path: format!("$.storyDraftTemplates.items[{index}].nodes"),
                message: format!("story draft template must contain exactly 24 nodes, got {}", nodes.len()),
            }),
            None => errors.push(PreflightIssue {
                code: "SL_STORY_TEMPLATE_NODE_COUNT",
                path: format!("$.storyDraftTemplates.items[{index}].nodes"),
                message: "story draft template nodes must be an array".to_string(),
            }),
        }
    }
}

fn validate_learning_policy(policy: &Value, errors: &mut Vec<PreflightIssue>) {
    if policy.get("schemaVersion").and_then(Value::as_str) != Some("1") {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.schemaVersion".to_string(),
            message: "learning-policy.json schemaVersion must be 1".to_string(),
        });
    }
    if policy
        .get("hostReadable")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        != true
    {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.hostReadable".to_string(),
            message: "learning-policy.json must be host-readable for retention execution".to_string(),
        });
    }
    validate_fixed_policy_number(
        policy,
        &["preLearning", "questionCount"],
        24,
        "$.preLearning.questionCount",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["preLearning", "promptsPerQuestion"],
        2,
        "$.preLearning.promptsPerQuestion",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["preLearning", "totalPrompts"],
        48,
        "$.preLearning.totalPrompts",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["preLearning", "minRepeatGap"],
        12,
        "$.preLearning.minRepeatGap",
        errors,
    );
    validate_range_policy_number(
        policy,
        &["preLearning", "errorTolerance"],
        "$.preLearning.errorTolerance",
        errors,
    );
    validate_range_policy_number(
        policy,
        &["preLearning", "weakItemLimit"],
        "$.preLearning.weakItemLimit",
        errors,
    );
    validate_fixed_policy_number(
        policy,
        &["retentionLearning", "questionCount"],
        22,
        "$.retentionLearning.questionCount",
        errors,
    );
    let phases = policy
        .get("retentionLearning")
        .and_then(|value| value.get("phases"))
        .and_then(Value::as_array);
    let Some(phases) = phases else {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: "$.retentionLearning.phases".to_string(),
            message: "learning-policy.json retention phases must be an array".to_string(),
        });
        return;
    };
    for required_phase in ["initial", "consolidation", "adaptation", "stable", "long_term"] {
        let Some(phase) = phases
            .iter()
            .find(|item| item.get("phase").and_then(Value::as_str) == Some(required_phase))
        else {
            errors.push(PreflightIssue {
                code: "SL_LEARNING_POLICY_INVALID",
                path: "$.retentionLearning.phases".to_string(),
                message: format!("learning-policy.json missing phase: {required_phase}"),
            });
            continue;
        };
        validate_phase_policy_number(phase, required_phase, "duration", errors);
        validate_phase_policy_number(phase, required_phase, "frequency", errors);
    }
}

fn policy_value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn validate_fixed_policy_number(
    policy: &Value,
    path: &[&str],
    expected: i64,
    display_path: &str,
    errors: &mut Vec<PreflightIssue>,
) {
    if policy_value_at(policy, path).and_then(Value::as_i64) != Some(expected) {
        errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: display_path.to_string(),
            message: format!("learning-policy.json {display_path} must be {expected}"),
        });
    }
}

fn validate_range_policy_number(
    policy: &Value,
    path: &[&str],
    display_path: &str,
    errors: &mut Vec<PreflightIssue>,
) {
    match policy_value_at(policy, path).and_then(Value::as_i64) {
        Some(value) if (1..=9).contains(&value) => {}
        _ => errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: display_path.to_string(),
            message: format!("learning-policy.json {display_path} must be a number from 1 to 9"),
        }),
    }
}

fn validate_phase_policy_number(
    phase: &Value,
    phase_name: &str,
    section: &str,
    errors: &mut Vec<PreflightIssue>,
) {
    match phase
        .get(section)
        .and_then(|value| value.get("value"))
        .and_then(Value::as_i64)
    {
        Some(value) if (1..=9).contains(&value) => {}
        _ => errors.push(PreflightIssue {
            code: "SL_LEARNING_POLICY_INVALID",
            path: format!("$.retentionLearning.phases.{phase_name}.{section}.value"),
            message: format!("{phase_name} {section} value must be a number from 1 to 9"),
        }),
    }
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

fn default_author_draft_json() -> Value {
    dongguo_wolf_author_draft_json()
}

fn story_template_author_draft_json(
    template_id: &str,
    title: &str,
    summary: &str,
    plot: &str,
    anchors: &[&str],
) -> Value {
    const ELEMENTS: [&str; 8] = [
        "time", "place", "person", "object", "event", "reaction", "choice", "result",
    ];
    let nodes = (1..=24)
        .map(|index| {
            let element_id = ELEMENTS[(index - 1) % ELEMENTS.len()];
            let correct = [
                format!("node {index:02} anchor one"),
                format!("node {index:02} anchor two"),
                format!("node {index:02} anchor three"),
            ];
            let wrong = [
                format!("node {index:02} distractor four"),
                format!("node {index:02} distractor five"),
                format!("node {index:02} distractor six"),
                format!("node {index:02} distractor seven"),
                format!("node {index:02} distractor eight"),
                format!("node {index:02} distractor nine"),
            ];
            let answer_options = correct
                .iter()
                .map(|text| json!({ "text": text, "isCorrect": true }))
                .chain(wrong.iter().map(|text| json!({ "text": text, "isCorrect": false })))
                .collect::<Vec<_>>();
            json!({
                "nodeId": format!("node-{index:02}"),
                "title": format!("Question {index:02}"),
                "elementId": element_id,
                "question": format!("Which three anchors belong to memory node {index:02}?"),
                "recommendedSelectionMode": "multi_select",
                "recommendedCorrectCount": 3,
                "candidatePoolSize": 9,
                "recallPriority": "high",
                "verifyPolicy": "caseInsensitive + trim",
                "editorNotes": "StoryLock UI local draft only.",
                "canonicalAnswerLocalOnly": correct[0],
                "acceptedAnswersLocalOnly": correct,
                "answerOptionsLocalOnly": answer_options
            })
        })
        .collect::<Vec<_>>();
    json!({
        "version": "1",
        "templateId": template_id,
        "storyTitle": title,
        "summary": summary,
        "storyPlot": plot,
        "memoryAnchors": anchors,
        "elementGroups": ["time", "place", "person", "object", "event", "reaction", "choice", "result"],
        "nodes": nodes
    })
}

fn dongguo_wolf_author_draft_json() -> Value {
    story_template_author_draft_json(
        "dongguo-wolf",
        "东郭先生和狼",
        "东郭先生救了被追捕的狼，狼脱险后反要吃掉恩人；最后借助旁人判断，揭示善良必须有边界。",
        "东郭先生在路上遇见一只被猎人追赶的狼。狼恳求他把自己藏进书袋，东郭先生一时心软救了它。猎人离开后，狼却露出凶相，说自己饥饿难忍，要吃掉东郭先生。双方争执不下，便请路边老人评理。老人让狼重新钻进袋子，确认事情经过后扎紧袋口，提醒东郭先生：善良需要判断对象，也需要边界。这个模板适合扩展恩情、伪装、风险判断、边界选择等 24 个记忆问题。",
        &["路上", "猎人", "书袋", "狼", "老人", "边界"],
    )
}

fn zhizi_yilin_author_draft_json() -> Value {
    story_template_author_draft_json(
        "zhizi-yilin",
        "智子疑邻",
        "同样的提醒，因为说话人身份不同而被截然不同地理解；故事适合扩展偏见、信任和证据判断。",
        "宋国有一家人的墙被雨水冲坏了。儿子提醒父亲，墙坏了如果不修，夜里可能会有盗贼进来；邻居也说了同样的话。当天夜里果然丢了东西，主人却夸儿子聪明，怀疑邻居偷窃。这个故事把同一句话放在亲疏不同的位置上，展示人会被身份偏见影响判断。它适合扩展墙、雨夜、邻居、儿子、失窃、怀疑、证据等 24 个问题。",
        &["雨夜", "破墙", "儿子", "邻居", "失窃", "偏见"],
    )
}

fn shouzhudaitu_author_draft_json() -> Value {
    story_template_author_draft_json(
        "shouzhudaitu",
        "守株待兔",
        "农夫偶然捡到撞树而死的兔子，随后放弃耕作，等待偶然再次发生，最终田地荒芜。",
        "宋国有个农夫正在田里劳作，一只兔子突然撞到树桩死了。农夫捡到兔子后，以为只要守着树桩就能再次得到兔子，于是放下农具，不再耕作。日子一天天过去，兔子没有再来，田地却荒芜了。这个模板可以扩展偶然、经验误判、等待、代价、结果等记忆元素。",
        &["田地", "树桩", "兔子", "农夫", "等待", "荒芜"],
    )
}
fn default_resource_catalog_json() -> Value {
    json!({
        "version": "1",
        "resources": [{
            "resourceId": "github-main",
            "resourceKind": "website_account",
            "providerId": "github",
            "displayName": "GitHub main account",
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
            "displayName": "GitHub main login",
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn format_all_template_bundles(package_dir: &Path) -> String {
    let templates = storylock_templates_from_vault(&read_storylock_vault(package_dir));
    [
        ("login-sites.json", default_login_templates_json()),
        ("signing-actions.json", default_signing_templates_json()),
        ("agent-tasks.json", default_agent_templates_json()),
    ]
    .iter()
    .map(|(file_name, fallback)| {
        let key = match *file_name {
            "login-sites.json" => "loginSites",
            "signing-actions.json" => "signingActions",
            "agent-tasks.json" => "agentTasks",
            _ => "",
        };
        let bundle = templates
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.clone());
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
        assert!(storylock_core_vault_path(&dir).exists());
        assert!(storylock_core_learning_policy_path(&dir).exists());
        let manifest = read_json_or_default(&storylock_core_manifest_path(&dir), Value::Null);
        assert!(manifest
            .get("files")
            .and_then(Value::as_array)
            .is_some_and(|files| files
                .iter()
                .any(|item| item.as_str() == Some("learning-policy.json"))));
    }

    #[test]
    fn default_story_templates_include_useful_fables() {
        let templates = default_story_draft_templates_json();
        let items = templates
            .get("items")
            .and_then(Value::as_array)
            .expect("default story draft templates");
        assert!(items.len() >= 3);
        for expected in ["dongguo-wolf", "zhizi-yilin", "shouzhudaitu"] {
            assert!(items.iter().any(|item| {
                item.get("templateId").and_then(Value::as_str) == Some(expected)
                    && item
                        .get("nodes")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        == Some(24)
            }));
        }
    }

    #[test]
    fn host_story_candidate_converts_to_author_draft_template() {
        let candidate = json!({
            "candidateId": "story-template-test",
            "framework": {
                "title": "Host Candidate",
                "summary": "Candidate summary",
                "storyPlot": "Candidate plot",
                "memoryAnchors": ["anchor-one", "anchor-two"]
            }
        });
        let draft = story_draft_from_candidate(&candidate);
        assert_eq!(
            draft.get("templateId").and_then(Value::as_str),
            Some("story-template-test")
        );
        assert_eq!(
            draft.get("storyTitle").and_then(Value::as_str),
            Some("Host Candidate")
        );
        assert_eq!(
            draft.get("nodes").and_then(Value::as_array).map(Vec::len),
            Some(24)
        );
    }

    #[test]
    fn export_preview_is_redacted() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let preview = build_export_preview(&dir);
        assert!(preview.contains("permissionObjects=2"));
        assert!(preview.contains("preflight=OK"));
        assert!(preview.contains("learning-policy.json"));
        assert!(preview.contains("StoryLock UI internal export preview only"));
        assert!(preview.contains("Yian Host reads learning-policy.json"));
        assert!(!preview.contains("signingKeyBytes="));
        assert!(!preview.contains("privateKey="));
        assert!(!preview.contains("password="));
    }

    #[test]
    fn effective_author_draft_prefers_pending_temp_file() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let mut pending = read_effective_author_draft(&dir);
        pending["storyTitle"] = json!("pending temp title");
        write_pending_author_draft(&dir, &pending).expect("write pending draft");
        let effective = read_effective_author_draft(&dir);
        assert_eq!(
            effective.get("storyTitle").and_then(Value::as_str),
            Some("pending temp title")
        );
    }

    #[test]
    fn story_draft_template_uses_author_draft_schema_and_restores_ui() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let mut draft = read_effective_author_draft(&dir);
        draft["templateId"] = json!("template-unified-title");
        draft["storyTitle"] = json!("Template Unified Title");
        draft["summary"] = json!("Template unified summary");
        draft["storyPlot"] = json!("Template unified plot detail");
        draft["nodes"][0]["question"] = json!("Unified question one?");
        normalize_author_draft_schema(&mut draft);
        let mut vault = read_storylock_vault_payload(&dir);
        vault["storyDraftTemplates"] = story_draft_templates_from_draft(&draft);
        save_storylock_vault_payload(&dir, vault).expect("save story draft template");

        let vault = read_storylock_vault(&dir);
        let template = vault
            .get("storyDraftTemplates")
            .and_then(|templates| templates.get("items"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .expect("story draft template");
        assert_eq!(
            template.get("storyTitle").and_then(Value::as_str),
            Some("Template Unified Title")
        );
        assert_eq!(
            template.get("summary").and_then(Value::as_str),
            Some("Template unified summary")
        );
        assert_eq!(
            template.get("storyPlot").and_then(Value::as_str),
            Some("Template unified plot detail")
        );
        assert_eq!(
            template.get("nodes").and_then(Value::as_array).map(Vec::len),
            Some(24)
        );

        let mut pending = read_effective_author_draft(&dir);
        pending["storyTitle"] = json!("Different pending title");
        write_pending_author_draft(&dir, &pending).expect("write different pending");
        let mut vault = read_storylock_vault_payload(&dir);
        let restored = vault
            .get("storyDraftTemplates")
            .and_then(|templates| templates.get("items"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .cloned()
            .expect("template draft");
        vault["pendingAuthorDraft"] = restored;
        save_storylock_vault_payload(&dir, vault).expect("restore template as pending");
        let effective = read_effective_author_draft(&dir);
        assert_eq!(
            effective.get("storyTitle").and_then(Value::as_str),
            Some("Template Unified Title")
        );
        assert_eq!(
            effective
                .get("nodes")
                .and_then(Value::as_array)
                .and_then(|nodes| nodes.first())
                .and_then(|node| node.get("question"))
                .and_then(Value::as_str),
            Some("Unified question one?")
        );
    }

    #[test]
    fn export_promotes_and_clears_pending_temp_draft() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let mut pending = read_effective_author_draft(&dir);
        pending["storyTitle"] = json!("promoted title");
        write_pending_author_draft(&dir, &pending).expect("write pending draft");

        let export_dir = export_storylock_package(&dir).expect("export package");
        let vault = read_storylock_vault(&dir);
        assert_eq!(
            vault
                .get("authorDraft")
                .and_then(|draft| draft.get("storyTitle"))
                .and_then(Value::as_str),
            Some("promoted title")
        );
        assert!(vault
            .get("pendingAuthorDraft")
            .map(|value| value.is_null())
            .unwrap_or(true));
        assert!(export_dir.join("vault.stlk").exists());
        assert!(export_dir.join("learning-policy.json").exists());
    }

    #[test]
    fn learning_policy_is_host_readable_and_blocks_invalid_values() {
        let dir = temp_core_dir();
        ensure_storylock_core_package(&dir).expect("init package");
        let policy = read_learning_policy(&dir);
        assert_eq!(
            policy
                .get("retentionLearning")
                .and_then(|value| value.get("questionCount"))
                .and_then(Value::as_i64),
            Some(22)
        );
        assert!(host_learning_plan_status(&dir).contains("Learning plan:"));

        let mut broken = policy;
        broken["preLearning"]["totalPrompts"] = json!(47);
        write_learning_policy(&dir, &broken).expect("write broken policy");
        let result = preflight_storylock_core_package(&dir);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.code == "SL_LEARNING_POLICY_INVALID"));
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
        let mut vault = read_storylock_vault(&dir);
        vault["templates"]["agentTasks"] = json!({
            "version": "1",
            "templateType": "agent-tasks",
            "items": [{
                "templateId": "broken-agent",
                "resourceId": "github-main",
                "bindings": [
                    { "fieldName": "missing", "role": "missing_role" }
                ]
            }]
        });
        write_storylock_vault(&dir, &vault).expect("write broken template");
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
