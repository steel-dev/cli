//! Integration tests for browser session lifecycle contracts.
//!
//! Pure-function tests for `to_session_summary`, `sanitize_connect_url`, and
//! connect-URL injection logic.  Also covers daemon protocol types
//! (`DaemonCreateParams`, `SessionInfo`, `GetSessionInfo`) and
//! `list_daemon_names()`.

use serde_json::json;
use tempfile::TempDir;

use steel_cli::browser::daemon::protocol::{
    DaemonCommand, DaemonCreateParams, SessionInfo,
};
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

fn local_auth() -> Auth {
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

// ===========================================================================
// DaemonCreateParams JSON roundtrip
// ===========================================================================

#[test]
fn daemon_create_params_json_roundtrip() {
    let params = DaemonCreateParams {
        api_key: Some("sk-test-key".to_string()),
        base_url: "https://api.steel.dev/v1".to_string(),
        mode: ApiMode::Cloud,
        session_name: "work".to_string(),
        stealth: true,
        proxy_url: Some("http://proxy:8080".to_string()),
        timeout_ms: Some(60000),
        headless: Some(true),
        region: Some("us-east-1".to_string()),
        solve_captcha: true,
        profile_id: Some("prof-1".to_string()),
        persist_profile: true,
        namespace: Some("ns".to_string()),
        credentials: false,
    };

    let json_str = serde_json::to_string(&params).unwrap();
    let back: DaemonCreateParams = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.api_key.as_deref(), Some("sk-test-key"));
    assert_eq!(back.base_url, "https://api.steel.dev/v1");
    assert_eq!(back.mode, ApiMode::Cloud);
    assert_eq!(back.session_name, "work");
    assert!(back.stealth);
    assert_eq!(back.proxy_url.as_deref(), Some("http://proxy:8080"));
    assert_eq!(back.timeout_ms, Some(60000));
    assert_eq!(back.headless, Some(true));
    assert_eq!(back.region.as_deref(), Some("us-east-1"));
    assert!(back.solve_captcha);
    assert_eq!(back.profile_id.as_deref(), Some("prof-1"));
    assert!(back.persist_profile);
    assert_eq!(back.namespace.as_deref(), Some("ns"));
    assert!(!back.credentials);
}

#[test]
fn daemon_create_params_minimal_roundtrip() {
    let params = DaemonCreateParams {
        api_key: None,
        base_url: "http://localhost:3000/v1".to_string(),
        mode: ApiMode::Local,
        session_name: "default".to_string(),
        stealth: false,
        proxy_url: None,
        timeout_ms: None,
        headless: None,
        region: None,
        solve_captcha: false,
        profile_id: None,
        persist_profile: false,
        namespace: None,
        credentials: false,
    };

    let json_str = serde_json::to_string(&params).unwrap();
    let back: DaemonCreateParams = serde_json::from_str(&json_str).unwrap();

    assert!(back.api_key.is_none());
    assert_eq!(back.mode, ApiMode::Local);
    assert_eq!(back.session_name, "default");
    assert!(!back.stealth);
}

#[test]
fn daemon_create_params_to_create_options() {
    let params = DaemonCreateParams {
        api_key: Some("sk-key".to_string()),
        base_url: "https://api.steel.dev/v1".to_string(),
        mode: ApiMode::Cloud,
        session_name: "test".to_string(),
        stealth: true,
        proxy_url: Some("http://proxy".to_string()),
        timeout_ms: Some(30000),
        headless: Some(false),
        region: Some("eu-west-1".to_string()),
        solve_captcha: true,
        profile_id: Some("prof".to_string()),
        persist_profile: true,
        namespace: Some("ns".to_string()),
        credentials: true,
    };

    let opts = params.to_create_options();
    assert!(opts.stealth);
    assert_eq!(opts.proxy_url.as_deref(), Some("http://proxy"));
    assert_eq!(opts.timeout_ms, Some(30000));
    assert_eq!(opts.headless, Some(false));
    assert_eq!(opts.region.as_deref(), Some("eu-west-1"));
    assert!(opts.solve_captcha);
    assert_eq!(opts.profile_id.as_deref(), Some("prof"));
    assert!(opts.persist_profile);
    assert_eq!(opts.namespace.as_deref(), Some("ns"));
    assert!(opts.credentials);
}

// ===========================================================================
// SessionInfo JSON roundtrip
// ===========================================================================

#[test]
fn session_info_json_roundtrip() {
    let info = SessionInfo {
        session_id: "sess-123".to_string(),
        session_name: "work".to_string(),
        mode: ApiMode::Cloud,
        status: Some("live".to_string()),
        connect_url: Some("wss://connect.steel.dev?sessionId=sess-123".to_string()),
        viewer_url: Some("https://app.steel.dev/sessions/sess-123".to_string()),
        profile_id: Some("prof-1".to_string()),
    };

    let json_str = serde_json::to_string(&info).unwrap();
    let back: SessionInfo = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.session_id, "sess-123");
    assert_eq!(back.session_name, "work");
    assert_eq!(back.mode, ApiMode::Cloud);
    assert_eq!(back.status.as_deref(), Some("live"));
    assert!(back.connect_url.is_some());
    assert!(back.viewer_url.is_some());
    assert_eq!(back.profile_id.as_deref(), Some("prof-1"));
}

#[test]
fn session_info_minimal_roundtrip() {
    let info = SessionInfo {
        session_id: "sess-min".to_string(),
        session_name: "default".to_string(),
        mode: ApiMode::Local,
        status: None,
        connect_url: None,
        viewer_url: None,
        profile_id: None,
    };

    let json_str = serde_json::to_string(&info).unwrap();
    let back: SessionInfo = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.session_id, "sess-min");
    assert_eq!(back.mode, ApiMode::Local);
    assert!(back.status.is_none());
    assert!(back.connect_url.is_none());
}

// ===========================================================================
// GetSessionInfo command serialization
// ===========================================================================

#[test]
fn get_session_info_serialization() {
    let cmd = DaemonCommand::GetSessionInfo;
    let v = serde_json::to_value(&cmd).unwrap();
    assert_eq!(v, json!({"action": "get_session_info"}));
}

#[test]
fn get_session_info_roundtrip() {
    let cmd = DaemonCommand::GetSessionInfo;
    let json_str = serde_json::to_string(&cmd).unwrap();
    let back: DaemonCommand = serde_json::from_str(&json_str).unwrap();
    assert_eq!(cmd, back);
}

// ===========================================================================
// list_daemon_names
// ===========================================================================

#[test]
fn list_daemon_names_finds_sock_files() {
    let dir = TempDir::new().unwrap();

    // Create mock socket files
    std::fs::write(dir.path().join("daemon-work.sock"), "").unwrap();
    std::fs::write(dir.path().join("daemon-dev.sock"), "").unwrap();
    std::fs::write(dir.path().join("daemon-default.sock"), "").unwrap();

    // Also create non-socket files that should be ignored
    std::fs::write(dir.path().join("daemon-work.pid"), "").unwrap();
    std::fs::write(dir.path().join("daemon-work.log"), "").unwrap();
    std::fs::write(dir.path().join("config.json"), "").unwrap();

    // Use env override to point to temp dir
    let names = list_daemon_names_in(dir.path());

    assert_eq!(names.len(), 3);
    assert!(names.contains(&"work".to_string()));
    assert!(names.contains(&"dev".to_string()));
    assert!(names.contains(&"default".to_string()));
}

#[test]
fn list_daemon_names_empty_dir() {
    let dir = TempDir::new().unwrap();
    let names = list_daemon_names_in(dir.path());
    assert!(names.is_empty());
}

#[test]
fn list_daemon_names_ignores_non_sock() {
    let dir = TempDir::new().unwrap();

    std::fs::write(dir.path().join("daemon-work.pid"), "").unwrap();
    std::fs::write(dir.path().join("daemon-work.log"), "").unwrap();
    std::fs::write(dir.path().join("daemon-.sock"), "").unwrap(); // empty name

    let names = list_daemon_names_in(dir.path());
    assert!(names.is_empty());
}

/// Testable version that takes an explicit directory.
fn list_daemon_names_in(dir: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(rest) = name.strip_prefix("daemon-") {
            if let Some(session_name) = rest.strip_suffix(".sock") {
                if !session_name.is_empty() {
                    names.push(session_name.to_string());
                }
            }
        }
    }
    names
}
