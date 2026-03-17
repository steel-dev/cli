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

/// Print a success response. In JSON mode, wraps data in `{"success": true, "data": ...}`.
/// In text mode, prints the provided human-readable text (if any).
pub fn success(data: Value, text: &str) {
    if is_json() {
        println!("{}", json!({"success": true, "data": data}));
    } else if !text.is_empty() {
        print!("{text}");
    }
}

/// Print a success response with just data (no text fallback).
pub fn success_data(data: Value) {
    if is_json() {
        println!("{}", json!({"success": true, "data": data}));
    } else {
        // Pretty-print JSON data as text fallback
        println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
    }
}

/// Print a silent success (for commands that produce no output on success).
pub fn success_silent() {
    if is_json() {
        println!("{}", json!({"success": true}));
    }
}

/// Print an error response. In JSON mode, outputs structured error.
/// In text mode, prints to stderr. Returns the error for propagation.
pub fn error(msg: &str) -> anyhow::Error {
    if is_json() {
        println!("{}", json!({"success": false, "error": msg}));
        std::process::exit(1);
    }
    anyhow::anyhow!("{msg}")
}
