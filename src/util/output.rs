//! Unified output formatting for human (text) and machine (JSON) modes.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

static JSON_MODE: AtomicBool = AtomicBool::new(false);
static IS_TTY: AtomicBool = AtomicBool::new(true);
static NO_COLOR: AtomicBool = AtomicBool::new(false);

/// Initialize output settings. Called once at startup.
///
/// - `explicit_json`: whether `--json` flag was passed
/// - Detects TTY via `stdout().is_terminal()`
/// - `STEEL_FORCE_TTY=1` overrides auto-detection
/// - `NO_COLOR` env var is detected for future color support
/// - Piped output (non-TTY) automatically enables JSON mode
pub fn init(explicit_json: bool) {
    let stdout_is_tty = std::io::stdout().is_terminal();
    let force_tty = std::env::var("STEEL_FORCE_TTY").is_ok_and(|v| v == "1");
    let tty = force_tty || stdout_is_tty;

    IS_TTY.store(tty, Ordering::Relaxed);
    NO_COLOR.store(std::env::var("NO_COLOR").is_ok(), Ordering::Relaxed);

    let json = explicit_json || !tty;
    JSON_MODE.store(json, Ordering::Relaxed);
}

/// Enable JSON output mode directly. For tests only.
#[cfg(test)]
pub fn set_json_mode(enabled: bool) {
    JSON_MODE.store(enabled, Ordering::Relaxed);
}

/// Check if JSON output mode is active.
pub fn is_json() -> bool {
    JSON_MODE.load(Ordering::Relaxed)
}

/// Check if stdout is a TTY (terminal).
pub fn is_tty() -> bool {
    IS_TTY.load(Ordering::Relaxed)
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
    pub const AUTH: i32 = 3;
    pub const NETWORK: i32 = 4;
    pub const API_CLIENT: i32 = 5;
    pub const API_SERVER: i32 = 6;
}

/// Sentinel error: the command already printed its own output and reported
/// failure; the process should exit with `code` without printing anything else.
#[derive(Debug)]
pub struct SilentExit(pub i32);

impl std::fmt::Display for SilentExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit {}", self.0)
    }
}

impl std::error::Error for SilentExit {}

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

fn format_success_field(json_mode: bool, data: &Value, field: &str) -> Option<String> {
    if json_mode {
        Some(format_success_data(json_mode, data))
    } else {
        let v = &data[field];
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if !v.is_null() {
            Some(v.to_string())
        } else {
            None
        }
    }
}

fn format_success_text(json_mode: bool, data: &Value) -> Option<String> {
    if json_mode {
        Some(format_success_data(json_mode, data))
    } else {
        data.as_str().map(|s| s.to_string())
    }
}

#[cfg(test)]
fn format_error(json_mode: bool, msg: &str) -> String {
    if json_mode {
        json!({"success": false, "error": msg}).to_string()
    } else {
        format!("Error: {msg}")
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

/// Print success: JSON wraps in envelope, text prints `data[field]`.
/// Strings are printed without quotes; numbers/bools are printed as-is.
pub fn success_field(data: Value, field: &str) {
    if let Some(output) = format_success_field(is_json(), &data, field) {
        println!("{output}");
    }
}

/// Print success: JSON wraps in envelope, text prints data as a string.
pub fn success_text(data: Value) {
    if let Some(output) = format_success_text(is_json(), &data) {
        println!("{output}");
    }
}

/// Format a top-level error for output, then exit. Called from main.
pub fn handle_error(err: &anyhow::Error) -> ! {
    if let Some(SilentExit(code)) = err.downcast_ref::<SilentExit>() {
        std::process::exit(*code);
    }

    let (code, error_code, hint) = classify_error(err);
    let msg = format!("{err:#}");

    if is_json() {
        let mut obj = json!({
            "success": false,
            "error": msg,
            "error_code": error_code,
        });
        if let Some(h) = hint {
            obj["hint"] = json!(h);
        }
        println!("{obj}");
    } else {
        eprintln!("Error: {msg}");
        if let Some(h) = hint {
            eprintln!("Hint: {h}");
        }
    }

    std::process::exit(code);
}

fn classify_error(err: &anyhow::Error) -> (i32, &'static str, Option<&'static str>) {
    if let Some(api_err) = err.downcast_ref::<crate::api::client::ApiError>() {
        match api_err {
            crate::api::client::ApiError::MissingAuth => (
                exit_code::AUTH,
                "auth_required",
                Some("Run `steel login` or set STEEL_API_KEY."),
            ),
            crate::api::client::ApiError::Unreachable { .. } => (
                exit_code::NETWORK,
                "unreachable",
                Some("Check your network connection and API URL."),
            ),
            crate::api::client::ApiError::RequestFailed { status, .. } => match *status {
                401 => (
                    exit_code::AUTH,
                    "unauthorized",
                    Some("API key may be invalid. Run `steel login`."),
                ),
                403 => (exit_code::API_CLIENT, "forbidden", None),
                404 => (exit_code::API_CLIENT, "not_found", None),
                429 => (
                    exit_code::API_CLIENT,
                    "rate_limited",
                    Some("Too many requests. Wait and retry."),
                ),
                s if s >= 500 => (
                    exit_code::API_SERVER,
                    "server_error",
                    Some("Steel API server error. Try again later."),
                ),
                _ => (exit_code::API_CLIENT, "request_failed", None),
            },
            _ => (
                exit_code::NETWORK,
                "network_error",
                Some("Check your network connection."),
            ),
        }
    } else {
        (exit_code::GENERAL, "internal_error", None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // --- format_success_field ---

    #[test]
    fn format_success_field_json_mode() {
        let data = json!({"url": "https://example.com"});
        let out = format_success_field(true, &data, "url").unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["url"], "https://example.com");
    }

    #[test]
    fn format_success_field_text_string() {
        let data = json!({"name": "Alice"});
        let out = format_success_field(false, &data, "name").unwrap();
        assert_eq!(out, "Alice");
    }

    #[test]
    fn format_success_field_text_number() {
        let data = json!({"count": 42});
        let out = format_success_field(false, &data, "count").unwrap();
        assert_eq!(out, "42");
    }

    #[test]
    fn format_success_field_text_null() {
        let data = json!({"missing": null});
        let out = format_success_field(false, &data, "missing");
        assert!(out.is_none());
    }

    // --- format_success_text ---

    #[test]
    fn format_success_text_json_mode() {
        let data = json!("hello");
        let out = format_success_text(true, &data).unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"], "hello");
    }

    #[test]
    fn format_success_text_text_string() {
        let data = json!("hello");
        let out = format_success_text(false, &data).unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn format_success_text_text_non_string() {
        let data = json!(123);
        let out = format_success_text(false, &data);
        assert!(out.is_none());
    }

    // --- format_error ---

    #[test]
    fn format_error_json_mode() {
        let out = format_error(true, "something broke");
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"], "something broke");
    }

    #[test]
    fn format_error_text_mode() {
        let out = format_error(false, "something broke");
        assert_eq!(out, "Error: something broke");
    }

    // --- classify_error ---

    #[test]
    fn classify_missing_auth() {
        let err: anyhow::Error = crate::api::client::ApiError::MissingAuth.into();
        let (code, error_code, hint) = classify_error(&err);
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
        let (code, error_code, _) = classify_error(&err);
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
        let (code, error_code, _) = classify_error(&err);
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
        let (code, error_code, hint) = classify_error(&err);
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
        let (code, error_code, _) = classify_error(&err);
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
        let (code, error_code, _) = classify_error(&err);
        assert_eq!(code, exit_code::API_CLIENT);
        assert_eq!(error_code, "request_failed");
    }

    #[test]
    fn classify_generic_error() {
        let err = anyhow::anyhow!("something unexpected");
        let (code, error_code, _) = classify_error(&err);
        assert_eq!(code, exit_code::GENERAL);
        assert_eq!(error_code, "internal_error");
    }
}
