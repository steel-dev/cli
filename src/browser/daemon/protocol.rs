use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::session::CreateSessionOptions;
use crate::config::settings::ApiMode;

/// Parameters passed to the daemon subprocess via env var so it can create
/// the cloud session itself.  Serialized as JSON into `STEEL_DAEMON_PARAMS`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonCreateParams {
    pub api_key: Option<String>,
    pub base_url: String,
    pub mode: ApiMode,
    pub session_name: String,
    // Flattened CreateSessionOptions fields:
    pub stealth: bool,
    pub proxy_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub headless: Option<bool>,
    pub region: Option<String>,
    pub solve_captcha: bool,
    pub profile_id: Option<String>,
    pub persist_profile: bool,
    pub namespace: Option<String>,
    pub credentials: bool,
}

impl DaemonCreateParams {
    pub fn to_create_options(&self) -> CreateSessionOptions {
        CreateSessionOptions {
            stealth: self.stealth,
            proxy_url: self.proxy_url.clone(),
            timeout_ms: self.timeout_ms,
            headless: self.headless,
            region: self.region.clone(),
            solve_captcha: self.solve_captcha,
            profile_id: self.profile_id.clone(),
            persist_profile: self.persist_profile,
            namespace: self.namespace.clone(),
            credentials: self.credentials,
        }
    }
}

/// Information about the daemon-managed session, returned by `GetSessionInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub session_name: String,
    pub mode: ApiMode,
    pub status: Option<String>,
    pub connect_url: Option<String>,
    pub viewer_url: Option<String>,
    pub profile_id: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub id: u64,
    pub command: DaemonCommand,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
    GetSessionInfo,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DaemonResponse {
    pub id: u64,
    pub result: DaemonResult,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DaemonResult {
    Ok { data: serde_json::Value },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- Serialization shape ---

    #[test]
    fn navigate_serialization_shape() {
        let cmd = DaemonCommand::Navigate {
            url: "https://example.com".into(),
            wait_until: None,
            headers: None,
        };
        let v = serde_json::to_value(&cmd).unwrap();
        assert_eq!(v["action"], "navigate");
        assert_eq!(v["url"], "https://example.com");
    }

    #[test]
    fn close_serialization_shape() {
        let cmd = DaemonCommand::Close;
        let v = serde_json::to_value(&cmd).unwrap();
        assert_eq!(v, json!({"action": "close"}));
    }

    #[test]
    fn snapshot_optional_fields_serialize_as_null() {
        let cmd = DaemonCommand::Snapshot {
            interactive_only: true,
            selector: None,
            compact: false,
            max_depth: None,
            cursor: false,
        };
        let v = serde_json::to_value(&cmd).unwrap();
        assert!(v["selector"].is_null());
        assert!(v["max_depth"].is_null());
    }

    #[test]
    fn click_snake_case_fields() {
        let cmd = DaemonCommand::Click {
            selector: "#btn".into(),
            button: None,
            click_count: Some(2),
            new_tab: true,
        };
        let v = serde_json::to_value(&cmd).unwrap();
        assert_eq!(v["new_tab"], true);
        assert_eq!(v["click_count"], 2);
        assert!(v.get("newTab").is_none());
        assert!(v.get("clickCount").is_none());
    }

    // --- Roundtrip ---

    #[test]
    fn roundtrip_navigate() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".into(), "val".into());
        let cmd = DaemonCommand::Navigate {
            url: "https://example.com".into(),
            wait_until: Some("load".into()),
            headers: Some(headers),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    #[test]
    fn roundtrip_close() {
        let cmd = DaemonCommand::Close;
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    #[test]
    fn roundtrip_snapshot_mixed_options() {
        let cmd = DaemonCommand::Snapshot {
            interactive_only: false,
            selector: Some("div".into()),
            compact: true,
            max_depth: None,
            cursor: true,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    #[test]
    fn roundtrip_scroll_f64() {
        let cmd = DaemonCommand::Scroll {
            selector: None,
            delta_x: 0.0,
            delta_y: -100.5,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    #[test]
    fn roundtrip_screenshot_many_optionals() {
        let cmd = DaemonCommand::Screenshot {
            full_page: true,
            selector: Some("body".into()),
            format: None,
            quality: Some(80),
            annotate: false,
            path: None,
            screenshot_dir: Some("/tmp".into()),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    #[test]
    fn roundtrip_wait_all_optional() {
        let cmd = DaemonCommand::Wait {
            timeout: Some(5000),
            text: None,
            selector: None,
            state: None,
            url: None,
            function: None,
            load_state: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    #[test]
    fn roundtrip_tab_switch() {
        let cmd = DaemonCommand::TabSwitch { index: 3 };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: DaemonCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, back);
    }

    // --- Raw JSON deserialization ---

    #[test]
    fn deserialize_navigate_raw_json() {
        let raw =
            r#"{"action":"navigate","url":"https://example.com","wait_until":null,"headers":null}"#;
        let cmd: DaemonCommand = serde_json::from_str(raw).unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::Navigate {
                url: "https://example.com".into(),
                wait_until: None,
                headers: None,
            }
        );
    }

    #[test]
    fn deserialize_click_raw_json() {
        let raw = r##"{"action":"click","selector":"#btn","button":null,"click_count":null,"new_tab":false}"##;
        let cmd: DaemonCommand = serde_json::from_str(raw).unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::Click {
                selector: "#btn".into(),
                button: None,
                click_count: None,
                new_tab: false,
            }
        );
    }

    // --- Request/Response wrappers ---

    #[test]
    fn daemon_request_roundtrip() {
        let req = DaemonRequest {
            id: 42,
            command: DaemonCommand::Ping,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: DaemonRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn daemon_response_ok_roundtrip() {
        let resp = DaemonResponse {
            id: 1,
            result: DaemonResult::Ok {
                data: json!({"url": "https://example.com"}),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: DaemonResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn daemon_response_error_roundtrip() {
        let resp = DaemonResponse {
            id: 2,
            result: DaemonResult::Error {
                message: "not found".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: DaemonResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn daemon_result_ok_status_tag() {
        let r = DaemonResult::Ok { data: json!(null) };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["status"], "ok");
    }

    #[test]
    fn daemon_result_error_status_tag() {
        let r = DaemonResult::Error {
            message: "fail".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["status"], "error");
    }
}
