use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DaemonRequest {
    pub id: u64,
    pub command: DaemonCommand,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum DaemonCommand {
    Navigate {
        url: String,
        wait_until: Option<String>,
        headers: Option<HashMap<String, String>>,
    },
    Click {
        selector: String,
        button: Option<String>,
        click_count: Option<i32>,
        new_tab: bool,
    },
    Fill {
        selector: String,
        value: String,
    },
    TypeText {
        selector: String,
        text: String,
        clear: bool,
        delay_ms: Option<u64>,
    },
    Snapshot {
        interactive_only: bool,
        selector: Option<String>,
        compact: bool,
        max_depth: Option<usize>,
        cursor: bool,
    },
    Screenshot {
        full_page: bool,
        selector: Option<String>,
        format: Option<String>,
        quality: Option<i32>,
        annotate: bool,
        path: Option<String>,
        screenshot_dir: Option<String>,
    },
    Press {
        key: String,
    },
    Hover {
        selector: String,
    },
    Scroll {
        selector: Option<String>,
        delta_x: f64,
        delta_y: f64,
    },
    Select {
        selector: String,
        values: Vec<String>,
    },
    Check {
        selector: String,
    },
    Uncheck {
        selector: String,
    },
    Eval {
        script: String,
    },
    GetText {
        selector: String,
    },
    GetAttribute {
        selector: String,
        attribute: String,
    },
    IsVisible {
        selector: String,
    },
    IsEnabled {
        selector: String,
    },
    IsChecked {
        selector: String,
    },
    Wait {
        timeout: Option<u64>,
        text: Option<String>,
        selector: Option<String>,
        state: Option<String>,
        url: Option<String>,
        function: Option<String>,
        load_state: Option<String>,
    },
    Url,
    Title,
    Back,
    Forward,
    Reload,
    Focus {
        selector: String,
    },
    Clear {
        selector: String,
    },
    SelectAll {
        selector: String,
    },
    ScrollIntoView {
        selector: String,
    },
    DblClick {
        selector: String,
    },
    InnerText {
        selector: String,
    },
    InnerHtml {
        selector: String,
    },
    InputValue {
        selector: String,
    },
    SetValue {
        selector: String,
        value: String,
    },
    Count {
        selector: String,
    },
    BoundingBox {
        selector: String,
    },
    Styles {
        selector: String,
        properties: Option<Vec<String>>,
    },
    Content,
    Find {
        selector: String,
    },
    BringToFront,
    TabList,
    TabNew {
        url: Option<String>,
    },
    TabSwitch {
        index: usize,
    },
    TabClose {
        index: Option<usize>,
    },
    Close,
    Ping,
    Shutdown,
}

#[derive(Serialize, Deserialize)]
pub struct DaemonResponse {
    pub id: u64,
    pub result: DaemonResult,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DaemonResult {
    Ok { data: serde_json::Value },
    Error { message: String },
}
