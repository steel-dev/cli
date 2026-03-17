//! Unified output formatting for human (text) and machine (JSON) modes.

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

static JSON_MODE: AtomicBool = AtomicBool::new(false);

/// Enable JSON output mode. Called once at startup.
pub fn set_json_mode(enabled: bool) {
    JSON_MODE.store(enabled, Ordering::Relaxed);
}

/// Check if JSON output mode is active.
pub fn is_json() -> bool {
    JSON_MODE.load(Ordering::Relaxed)
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
    let msg = format!("{err:#}");
    let output = format_error(is_json(), &msg);
    if is_json() {
        println!("{output}");
    } else {
        eprintln!("{output}");
    }
    std::process::exit(1);
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
}
