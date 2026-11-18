//! Unified output formatting for human (text) and machine (JSON) modes.

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

static JSON_MODE: AtomicBool = AtomicBool::new(false);
static NO_COLOR: AtomicBool = AtomicBool::new(false);

/// Initialize output settings. Called once at startup.
pub fn init(json: bool) {
    NO_COLOR.store(std::env::var("NO_COLOR").is_ok(), Ordering::Relaxed);
    JSON_MODE.store(json, Ordering::Relaxed);
}

/// Check if JSON output mode is active.
pub fn is_json() -> bool {
    JSON_MODE.load(Ordering::Relaxed)
}

/// Check if `NO_COLOR` environment variable is set.
pub fn no_color() -> bool {
    NO_COLOR.load(Ordering::Relaxed)
}

/// Human-only status messages, printed to stderr. Suppressed in JSON mode.
#[macro_export]
macro_rules! status {
    ($($arg:tt)*) => {
        if !$crate::util::output::is_json() {
            eprintln!($($arg)*);
        }
    };
}

// --- Exit codes ---

pub mod exit_code {
    pub const GENERAL: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const AUTH: i32 = 3;
    pub const NETWORK: i32 = 4;
    pub const API_CLIENT: i32 = 5;
    pub const API_SERVER: i32 = 6;
    pub const INTERRUPTED: i32 = 130;
}

/// Sentinel error: the command already printed its own output and reported
/// failure; the process should exit with `code` without printing anything else.
#[derive(Debug)]
pub struct SilentExit {
    pub code: i32,
    pub error_code: String,
}

impl SilentExit {
    pub fn new(error_code: &str) -> Self {
        Self {
            code: exit_code::GENERAL,
            error_code: error_code.to_string(),
        }
    }
}

impl std::fmt::Display for SilentExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit {}", self.code)
    }
}

impl std::error::Error for SilentExit {}

pub fn fail(code: &str, message: &str, data: Option<&Value>) -> anyhow::Error {
    if is_json() {
        let mut obj = json!({"success": false, "error": message, "error_code": code});
        if let Some(data) = data {
            obj["data"] = data.clone();
        }
        println!("{obj}");
    } else {
        eprintln!("Error: {message}");
    }
    SilentExit::new(code).into()
}

// --- Pure formatting functions (no I/O) ---

fn format_success(json_mode: bool, data: &Value, text: &str) -> Option<String> {
    if json_mode {
        Some(json!({"success": true, "data": data}).to_string())
    } else if !text.is_empty() {
        Some(text.to_string())
    } else {
        None
    }
}

fn format_success_data(json_mode: bool, data: &Value) -> String {
    if json_mode {
        json!({"success": true, "data": data}).to_string()
    } else {
        serde_json::to_string_pretty(data).unwrap_or_default()
    }
}

fn format_success_silent(json_mode: bool) -> Option<String> {
    if json_mode {
        Some(json!({"success": true}).to_string())
    } else {
        None
    }
}

// --- Public I/O functions (delegate to format_*) ---

/// Print a success response. In JSON mode, wraps data in `{"success": true, "data": ...}`.
/// In text mode, prints the provided human-readable text (if any).
pub fn success(data: Value, text: &str) {
    if let Some(output) = format_success(is_json(), &data, text) {
        if is_json() {
            println!("{output}");
        } else {
            print!("{output}");
        }
    }
}

/// Print a success response with just data (no text fallback).
pub fn success_data(data: Value) {
    println!("{}", format_success_data(is_json(), &data));
}

/// Print a silent success (for commands that produce no output on success).
pub fn success_silent() {
    if let Some(output) = format_success_silent(is_json()) {
        println!("{output}");
    }
}

pub fn print_table(header: &[&str], rows: &[Vec<String>]) {
    let widths: Vec<usize> = header
        .iter()
        .enumerate()
        .map(|(column, name)| {
            rows.iter()
                .map(|row| row.get(column).map_or(0, String::len))
                .chain(std::iter::once(name.len()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |cells: Vec<&str>| {
        cells
            .iter()
            .enumerate()
            .map(|(column, cell)| format!("{cell:<width$}", width = widths[column]))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_string()
    };
    println!("{}", line(header.to_vec()));
    for row in rows {
        println!("{}", line(row.iter().map(String::as_str).collect()));
    }
}

/// Format a top-level error for output, then exit. Called from main.
pub fn handle_error(err: &anyhow::Error) -> ! {
    if let Some(silent) = err.downcast_ref::<SilentExit>() {
        std::process::exit(silent.code);
    }

    let class = classify(err);
    let msg = format!("{err:#}");
    let data = err
        .downcast_ref::<crate::util::error::LoginRequired>()
        .map(crate::util::error::LoginRequired::to_json);

    if is_json() {
        let mut obj = json!({
            "success": false,
            "error": msg,
            "error_code": class.code,
        });
        if let Some(h) = class.hint {
            obj["hint"] = json!(h);
        }
        if let Some(data) = data {
            obj["data"] = data;
        }
        println!("{obj}");
    } else {
        eprintln!("Error: {msg}");
        if let Some(h) = class.hint {
            eprintln!("Hint: {h}");
        }
    }

    std::process::exit(class.exit_code);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorClass {
    pub exit_code: i32,
    pub code: &'static str,
    pub telemetry: &'static str,
    pub hint: Option<&'static str>,
}

const fn class(
    exit_code: i32,
    code: &'static str,
    telemetry: &'static str,
    hint: Option<&'static str>,
) -> ErrorClass {
    ErrorClass {
        exit_code,
        code,
        telemetry,
        hint,
    }
}

const NETWORK_HINT: Option<&str> = Some("Check your network connection.");
const LOCAL_STATE_HINT: Option<&str> = Some("Delete the file and retry.");
const LOGIN_HINT: Option<&str> = Some(
    "If an agent runs this command, show the URL and code to the user and retry after they approve. STEEL_API_KEY also works.",
);

const fn classify_api(err: &crate::api::client::ApiError) -> ErrorClass {
    use crate::api::client::ApiError;
    match err {
        ApiError::DeviceAuth(_) => class(
            exit_code::AUTH,
            "device_auth_failed",
            "device_auth_failed",
            Some("Resume pending login or reauthorize as directed."),
        ),
        ApiError::MissingAuth => class(
            exit_code::AUTH,
            "auth_required",
            "auth_required",
            Some("Run `steel login` or set STEEL_API_KEY."),
        ),
        ApiError::Unreachable { .. } => class(
            exit_code::NETWORK,
            "unreachable",
            "network_unreachable",
            Some("Check your network connection and API URL."),
        ),
        ApiError::Redirected { .. } => class(
            exit_code::NETWORK,
            "network_error",
            "api_redirect_refused",
            NETWORK_HINT,
        ),
        ApiError::RequestFailed { status, .. } => match *status {
            401 => class(
                exit_code::AUTH,
                "unauthorized",
                "api_unauthorized",
                Some("API key may be invalid. Run `steel login`."),
            ),
            403 => class(exit_code::API_CLIENT, "forbidden", "api_forbidden", None),
            404 | 410 => class(exit_code::API_CLIENT, "not_found", "api_not_found", None),
            429 => class(
                exit_code::API_CLIENT,
                "rate_limited",
                "api_rate_limited",
                Some("Too many requests. Wait and retry."),
            ),
            s if s >= 500 => class(
                exit_code::API_SERVER,
                "server_error",
                "api_server_error",
                Some("Steel API server error. Try again later."),
            ),
            _ => class(
                exit_code::API_CLIENT,
                "request_failed",
                "api_request_failed",
                None,
            ),
        },
        ApiError::Other(_) => class(
            exit_code::NETWORK,
            "network_error",
            "network_error",
            NETWORK_HINT,
        ),
    }
}

pub fn classify(err: &anyhow::Error) -> ErrorClass {
    use crate::util::error::{ApiBadResponse, InvalidInput, LocalStateInvalid, LoginRequired};
    if let Some(api_err) = err.downcast_ref::<crate::api::client::ApiError>() {
        return classify_api(api_err);
    }
    if err.downcast_ref::<LoginRequired>().is_some() {
        return class(
            exit_code::AUTH,
            "auth_required",
            "auth_required",
            LOGIN_HINT,
        );
    }
    if err.downcast_ref::<InvalidInput>().is_some() {
        return class(exit_code::USAGE, "invalid_input", "invalid_input", None);
    }
    if err.downcast_ref::<ApiBadResponse>().is_some() {
        return class(
            exit_code::API_SERVER,
            "api_bad_response",
            "api_bad_response",
            None,
        );
    }
    if err.downcast_ref::<LocalStateInvalid>().is_some() {
        return class(
            exit_code::GENERAL,
            "local_state_invalid",
            "local_state_invalid",
            LOCAL_STATE_HINT,
        );
    }
    if let Some(http) = err
        .chain()
        .find_map(|source| source.downcast_ref::<reqwest::Error>())
    {
        if http.is_decode() {
            return class(
                exit_code::API_SERVER,
                "api_bad_response",
                "api_bad_response",
                None,
            );
        }
        if !http.is_builder() {
            return class(
                exit_code::NETWORK,
                "network_error",
                "network_error",
                NETWORK_HINT,
            );
        }
    }
    if err.chain().any(|source| source.is::<std::io::Error>()) {
        return class(exit_code::GENERAL, "io_error", "io_error", None);
    }
    class(exit_code::GENERAL, "internal_error", "internal_error", None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use serde_json::json;

    // --- format_success ---

    #[test]
    fn format_success_json_mode() {
        let data = json!({"key": "value"});
        let out = format_success(true, &data, "ignored").unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["key"], "value");
    }

    #[test]
    fn format_success_text_mode() {
        let data = json!(null);
        let out = format_success(false, &data, "hello world").unwrap();
        assert_eq!(out, "hello world");
    }

    #[test]
    fn format_success_text_mode_empty() {
        let data = json!(null);
        let out = format_success(false, &data, "");
        assert!(out.is_none());
    }

    // --- format_success_data ---

    #[test]
    fn format_success_data_json_mode() {
        let data = json!(42);
        let out = format_success_data(true, &data);
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"], 42);
    }

    #[test]
    fn format_success_data_text_mode() {
        let data = json!({"a": 1});
        let out = format_success_data(false, &data);
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["a"], 1);
    }

    // --- format_success_silent ---

    #[test]
    fn format_success_silent_json_mode() {
        let out = format_success_silent(true).unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none());
    }

    #[test]
    fn format_success_silent_text_mode() {
        assert!(format_success_silent(false).is_none());
    }

    // --- classify ---

    #[test]
    fn classify_missing_auth() {
        let err: anyhow::Error = crate::api::client::ApiError::MissingAuth.into();
        let ErrorClass {
            exit_code: code,
            code: error_code,
            hint,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::AUTH);
        assert_eq!(error_code, "auth_required");
        assert!(hint.is_some());
    }

    #[test]
    fn classify_request_failed_401() {
        let err: anyhow::Error = crate::api::client::ApiError::RequestFailed {
            status: 401,
            message: std::borrow::Cow::Borrowed("Unauthorized"),
            body: None,
        }
        .into();
        let ErrorClass {
            exit_code: code,
            code: error_code,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::AUTH);
        assert_eq!(error_code, "unauthorized");
    }

    #[test]
    fn classify_request_failed_404() {
        let err: anyhow::Error = crate::api::client::ApiError::RequestFailed {
            status: 404,
            message: std::borrow::Cow::Borrowed("Not Found"),
            body: None,
        }
        .into();
        let ErrorClass {
            exit_code: code,
            code: error_code,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::API_CLIENT);
        assert_eq!(error_code, "not_found");
    }

    #[test]
    fn classify_request_failed_429() {
        let err: anyhow::Error = crate::api::client::ApiError::RequestFailed {
            status: 429,
            message: std::borrow::Cow::Borrowed("Too Many Requests"),
            body: None,
        }
        .into();
        let ErrorClass {
            exit_code: code,
            code: error_code,
            hint,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::API_CLIENT);
        assert_eq!(error_code, "rate_limited");
        assert!(hint.is_some());
    }

    #[test]
    fn classify_request_failed_500() {
        let err: anyhow::Error = crate::api::client::ApiError::RequestFailed {
            status: 500,
            message: std::borrow::Cow::Borrowed("Internal Server Error"),
            body: None,
        }
        .into();
        let ErrorClass {
            exit_code: code,
            code: error_code,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::API_SERVER);
        assert_eq!(error_code, "server_error");
    }

    #[test]
    fn classify_request_failed_generic_4xx() {
        let err: anyhow::Error = crate::api::client::ApiError::RequestFailed {
            status: 422,
            message: std::borrow::Cow::Borrowed("Unprocessable Entity"),
            body: None,
        }
        .into();
        let ErrorClass {
            exit_code: code,
            code: error_code,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::API_CLIENT);
        assert_eq!(error_code, "request_failed");
    }

    #[test]
    fn classify_generic_error() {
        let err = anyhow::anyhow!("something unexpected");
        let ErrorClass {
            exit_code: code,
            code: error_code,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::GENERAL);
        assert_eq!(error_code, "internal_error");
    }

    #[test]
    fn classify_invalid_input() {
        let err = crate::util::error::invalid_input("Session names cannot look like session IDs.");
        let ErrorClass {
            exit_code: code,
            code: error_code,
            ..
        } = classify(&err);
        assert_eq!(code, exit_code::USAGE);
        assert_eq!(error_code, "invalid_input");
    }

    #[test]
    fn classify_login_required_as_auth_required() {
        let err: anyhow::Error = crate::util::error::LoginRequired {
            verification_uri: "https://auth.steel.dev/device".into(),
            verification_uri_complete: "https://auth.steel.dev/device?user_code=ABCD2345".into(),
            user_code: "ABCD2345".into(),
            expires_at: u64::MAX,
            started: true,
        }
        .into();
        let class = classify(&err);
        assert_eq!(class.exit_code, exit_code::AUTH);
        assert_eq!(class.code, "auth_required");
        assert_eq!(class.telemetry, "auth_required");
        assert!(class.hint.is_some());
    }

    #[test]
    fn classify_bad_session_response() {
        let err = serde_json::from_value::<Vec<u8>>(json!({}))
            .map_err(anyhow::Error::from)
            .context(crate::util::error::ApiBadResponse(
                "The Steel API returned an unexpected session.",
            ))
            .unwrap_err();
        let class = classify(&err);
        assert_eq!(class.exit_code, exit_code::API_SERVER);
        assert_eq!(class.code, "api_bad_response");
    }

    #[test]
    fn classify_corrupt_local_state() {
        let err = serde_json::from_str::<Value>("{")
            .map_err(anyhow::Error::from)
            .context(crate::util::error::LocalStateInvalid("default.json".into()))
            .unwrap_err();
        let class = classify(&err);
        assert_eq!(class.code, "local_state_invalid");
        assert_eq!(class.hint, Some("Delete the file and retry."));
        assert!(format!("{err:#}").starts_with("Could not read default.json: "));
    }

    #[tokio::test]
    async fn classify_reqwest_errors_by_kind() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_string("<html>"))
            .mount(&server)
            .await;
        let decode = reqwest::get(server.uri())
            .await
            .unwrap()
            .json::<Value>()
            .await
            .map_err(anyhow::Error::from)
            .context("Auth service returned non-JSON data")
            .unwrap_err();
        let class = classify(&decode);
        assert_eq!(class.code, "api_bad_response");
        assert_eq!(class.exit_code, exit_code::API_SERVER);

        let unreachable = reqwest::get("http://127.0.0.1:1")
            .await
            .map_err(anyhow::Error::from)
            .unwrap_err();
        assert_eq!(classify(&unreachable).code, "network_error");
    }

    #[test]
    fn classify_io_and_client_errors() {
        let io: anyhow::Error = std::io::Error::other("disk full").into();
        assert_eq!(classify(&io).code, "io_error");
        let device: anyhow::Error =
            crate::api::client::ApiError::DeviceAuth(anyhow::anyhow!("expired")).into();
        assert_eq!(classify(&device).code, "device_auth_failed");
    }

    #[test]
    fn classify_gone_as_not_found() {
        let err: anyhow::Error = crate::api::client::ApiError::RequestFailed {
            status: 410,
            message: std::borrow::Cow::Borrowed("Gone"),
            body: None,
        }
        .into();
        let class = classify(&err);
        assert_eq!(class.code, "not_found");
        assert_eq!(class.telemetry, "api_not_found");
    }
}
