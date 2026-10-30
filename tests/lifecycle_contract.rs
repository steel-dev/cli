//! Integration tests for browser session lifecycle contracts.
//!
//! Pure-function tests for `to_session_summary`, `sanitize_connect_url`, and
//! connect-URL injection logic.

use serde_json::json;

use steel_cli::browser::lifecycle::*;
use steel_cli::config::auth::{Auth, AuthSource};
use steel_cli::config::settings::ApiMode;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cloud_auth() -> Auth {
    Auth {
        api_key: Some("sk-test-key-12345678".to_string()),
        source: AuthSource::Env,
    }
}

const fn local_auth() -> Auth {
    Auth {
        api_key: None,
        source: AuthSource::None,
    }
}

// ===========================================================================
// Connect URL Contract: injects apiKey into connect URL for cloud mode
// ===========================================================================

#[test]
fn injects_api_key_into_connect_url_for_cloud() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
        "websocketUrl": "wss://connect.steel.dev?sessionId=sess-1",
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Cloud, None, &auth).unwrap();

    let url = summary.connect_url.unwrap();
    assert!(
        url.contains("apiKey=sk-test-key-12345678"),
        "Expected apiKey in URL, got: {url}"
    );
    assert!(
        url.contains("sessionId=sess-1"),
        "Expected sessionId preserved in URL, got: {url}"
    );
}

#[test]
fn does_not_inject_api_key_for_local_mode() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
        "websocketUrl": "ws://localhost:3000/ws?sessionId=sess-1",
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Local, None, &auth).unwrap();

    let url = summary.connect_url.unwrap();
    assert!(
        !url.contains("apiKey"),
        "Local mode should not inject apiKey, got: {url}"
    );
}

#[test]
fn does_not_duplicate_api_key_if_already_present() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
        "websocketUrl": "wss://connect.steel.dev?apiKey=existing-key&sessionId=sess-1",
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Cloud, None, &auth).unwrap();

    let url = summary.connect_url.unwrap();
    // Should keep existing key, not inject a second one
    assert!(url.contains("apiKey=existing-key"));
    let count = url.matches("apiKey").count();
    assert_eq!(
        count, 1,
        "Expected exactly one apiKey, got {count} in: {url}"
    );
}

// ===========================================================================
// Connect URL Contract: fallback builds wss://connect.steel.dev URL
// ===========================================================================

#[test]
fn builds_fallback_connect_url_when_not_provided() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Cloud, None, &auth).unwrap();

    let url = summary.connect_url.unwrap();
    assert!(
        url.starts_with("wss://connect.steel.dev"),
        "Expected fallback URL, got: {url}"
    );
    assert!(
        url.contains("apiKey=sk-test-key-12345678"),
        "Expected apiKey in fallback URL, got: {url}"
    );
    assert!(
        url.contains("sessionId=sess-1"),
        "Expected sessionId in fallback URL, got: {url}"
    );
}

#[test]
fn no_fallback_connect_url_for_local_mode() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
    });
    let auth = local_auth();
    let summary = to_session_summary(&session, ApiMode::Local, None, &auth).unwrap();

    assert!(
        summary.connect_url.is_none(),
        "Local mode without connect URL should not build a fallback"
    );
}

// ===========================================================================
// Connect URL Contract: sanitize_connect_url masks apiKey
// ===========================================================================

#[test]
fn sanitize_masks_api_key_in_url() {
    let url = "wss://connect.steel.dev?apiKey=sk-12345678901234&sessionId=sess-1";
    let sanitized = sanitize_connect_url(url);

    assert!(
        sanitized.contains("sk-1234..."),
        "Expected masked key, got: {sanitized}"
    );
    assert!(
        sanitized.contains("sessionId=sess-1"),
        "Expected sessionId preserved, got: {sanitized}"
    );
    assert!(
        !sanitized.contains("sk-12345678901234"),
        "Full API key should be masked, got: {sanitized}"
    );
}

#[test]
fn sanitize_preserves_short_api_key() {
    // Keys with 7 or fewer chars are not masked (threshold check)
    let url = "wss://connect.steel.dev?apiKey=short&sessionId=s1";
    let sanitized = sanitize_connect_url(url);
    assert!(
        sanitized.contains("apiKey=short"),
        "Short key should not be masked, got: {sanitized}"
    );
}

#[test]
fn sanitize_handles_url_without_api_key() {
    let url = "ws://localhost:3000/ws?sessionId=sess-1";
    let sanitized = sanitize_connect_url(url);
    assert_eq!(sanitized, url);
}

#[test]
fn sanitize_handles_invalid_url() {
    let url = "not a valid url";
    let sanitized = sanitize_connect_url(url);
    assert_eq!(sanitized, url);
}

// ===========================================================================
// Viewer URL tests
// ===========================================================================

#[test]
fn viewer_url_defaults_to_cloud_url() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Cloud, None, &auth).unwrap();

    assert_eq!(
        summary.viewer_url.as_deref(),
        Some("https://app.steel.dev/sessions/sess-1")
    );
}

#[test]
fn viewer_url_none_for_local_without_explicit() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
    });
    let auth = local_auth();
    let summary = to_session_summary(&session, ApiMode::Local, None, &auth).unwrap();

    assert!(summary.viewer_url.is_none());
}

#[test]
fn viewer_url_uses_api_response_when_present() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
        "sessionViewerUrl": "https://custom-viewer.example.com/view/sess-1",
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Cloud, None, &auth).unwrap();

    assert_eq!(
        summary.viewer_url.as_deref(),
        Some("https://custom-viewer.example.com/view/sess-1")
    );
}

// ===========================================================================
// to_session_summary edge cases
// ===========================================================================

#[test]
fn to_session_summary_requires_id() {
    let session = json!({"status": "live"});
    let auth = cloud_auth();
    let result = to_session_summary(&session, ApiMode::Cloud, None, &auth);
    assert!(result.is_err());
}

#[test]
fn to_session_summary_passes_name_through() {
    let session = json!({
        "id": "sess-1",
        "status": "live",
        "isLive": true,
    });
    let auth = cloud_auth();
    let summary = to_session_summary(&session, ApiMode::Cloud, Some("my-session"), &auth).unwrap();
    assert_eq!(summary.name.as_deref(), Some("my-session"));
}
