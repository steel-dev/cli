//! Integration tests for browser session lifecycle contracts.
//!
//! These tests mirror the contracts from the Node.js `browser-lifecycle.test.ts`,
//! using wiremock to mock the Steel API and tempfile for isolated session state.

use serde_json::json;
use tempfile::TempDir;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use steel_cli::api::client::SteelClient;
use steel_cli::api::session::CreateSessionOptions;
use steel_cli::browser::lifecycle::*;
use steel_cli::config::auth::{Auth, AuthSource};
use steel_cli::config::session_state::{
    read_state, with_lock, SessionStatePaths,
};
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

fn tmp_paths() -> (TempDir, SessionStatePaths) {
    let dir = TempDir::new().unwrap();
    let paths = SessionStatePaths::new(dir.path());
    (dir, paths)
}

fn new_client() -> SteelClient {
    SteelClient::new().unwrap()
}

/// Mount a mock that returns a single live session for POST /sessions.
async fn mock_create_session(server: &MockServer, id: &str) {
    Mock::given(method("POST"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "status": "live",
            "isLive": true,
        })))
        .mount(server)
        .await;
}

/// Mount a mock that returns a single session for GET /sessions/:id.
async fn mock_get_session(server: &MockServer, id: &str, live: bool) {
    let status = if live { "live" } else { "released" };
    Mock::given(method("GET"))
        .and(path(format!("/sessions/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "status": status,
            "isLive": live,
        })))
        .mount(server)
        .await;
}

/// Mount a mock that returns 404 for GET /sessions/:id.
async fn mock_get_session_not_found(server: &MockServer, id: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/sessions/{id}")))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(json!({"message": "Not found"})),
        )
        .mount(server)
        .await;
}

/// Mount a mock for POST /sessions/:id/release.
async fn mock_release_session(server: &MockServer, id: &str) {
    Mock::given(method("POST"))
        .and(path(format!("/sessions/{id}/release")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(server)
        .await;
}

// ===========================================================================
// 1. Session Creation: creates cloud session and persists active state
// ===========================================================================

#[tokio::test]
async fn creates_cloud_session_and_persists_active_state() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    mock_create_session(&server, "sess-cloud-1").await;

    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        &CreateSessionOptions::default(),
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "sess-cloud-1");
    assert_eq!(summary.mode, ApiMode::Cloud);
    assert!(summary.live);

    // Verify state was persisted
    let state = read_state(&paths.state_path);
    assert_eq!(state.active_session_id.as_deref(), Some("sess-cloud-1"));
    assert_eq!(state.active_api_mode, Some(ApiMode::Cloud));
}

// ===========================================================================
// 2. Session Creation: maps config fields into create payload
// ===========================================================================

#[tokio::test]
async fn maps_session_config_fields_into_create_payload() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Expect the exact mapped payload
    Mock::given(method("POST"))
        .and(path("/sessions"))
        .and(body_json(json!({
            "timeout": 60000,
            "headless": true,
            "region": "us-east-1",
            "solveCaptcha": true,
            "stealthConfig": {
                "humanizeInteractions": true,
                "autoCaptchaSolving": true
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sess-mapped",
            "status": "live",
            "isLive": true,
        })))
        .mount(&server)
        .await;

    let options = CreateSessionOptions {
        timeout_ms: Some(60000),
        headless: Some(true),
        region: Some("us-east-1".to_string()),
        solve_captcha: true,
        stealth: true,
        ..Default::default()
    };

    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        &options,
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "sess-mapped");
}

// ===========================================================================
// 3. Session Creation: creates local session when explicit api-url is provided
// ===========================================================================

#[tokio::test]
async fn creates_local_session_with_explicit_api_url() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    mock_create_session(&server, "sess-local-1").await;

    // When using Local mode (as resolved from an explicit non-steel api-url),
    // the session should be created in Local mode.
    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Local,
        &local_auth(),
        &paths,
        None,
        &CreateSessionOptions::default(),
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "sess-local-1");
    assert_eq!(summary.mode, ApiMode::Local);
    assert!(summary.live);

    let state = read_state(&paths.state_path);
    assert_eq!(state.active_session_id.as_deref(), Some("sess-local-1"));
    assert_eq!(state.active_api_mode, Some(ApiMode::Local));
}

// ===========================================================================
// 4. Session Creation: reattaches a named live session instead of creating new
// ===========================================================================

#[tokio::test]
async fn reattaches_named_live_session() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed state with a named session
    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "existing-sess".to_string());
    })
    .unwrap();

    // Mock: GET /sessions/existing-sess returns a live session
    mock_get_session(&server, "existing-sess", true).await;

    // No POST /sessions mock -- if start_session tries to create, it will fail
    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        Some("work"),
        &CreateSessionOptions::default(),
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "existing-sess");
    assert_eq!(summary.mode, ApiMode::Cloud);
    assert!(summary.live);

    // Active state should point to the reattached session
    let state = read_state(&paths.state_path);
    assert_eq!(
        state.active_session_id.as_deref(),
        Some("existing-sess")
    );
    assert_eq!(
        state.active_session_name.as_deref(),
        Some("work")
    );
}

// ===========================================================================
// 5. Session Creation: clears dead sessions from state and creates new one
// ===========================================================================

#[tokio::test]
async fn clears_dead_session_and_creates_new() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed state with a named session that is now dead
    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "dead-sess".to_string());
    })
    .unwrap();

    // Mock: GET /sessions/dead-sess returns 404 (dead)
    mock_get_session_not_found(&server, "dead-sess").await;
    // Mock: POST /sessions creates a new session
    mock_create_session(&server, "new-sess").await;

    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        Some("work"),
        &CreateSessionOptions::default(),
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "new-sess");
    assert!(summary.live);

    // Verify dead session was cleared and new one is active
    let state = read_state(&paths.state_path);
    assert_eq!(state.active_session_id.as_deref(), Some("new-sess"));
    assert_eq!(
        state.named_sessions.cloud.get("work").map(|s| s.as_str()),
        Some("new-sess")
    );
}

// ===========================================================================
// 6. Session Stop: stops active session by releasing via API
// ===========================================================================

#[tokio::test]
async fn stops_active_session_by_releasing() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed active session
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "sess-to-stop", None);
    })
    .unwrap();

    mock_release_session(&server, "sess-to-stop").await;

    let result = stop_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        false,
    )
    .await
    .unwrap();

    assert_eq!(result.stopped_session_ids, vec!["sess-to-stop"]);
    assert!(!result.all);
    assert_eq!(result.mode, ApiMode::Cloud);
}

// ===========================================================================
// 7. Session Stop: --all releases all live sessions
// ===========================================================================

#[tokio::test]
async fn stop_all_releases_all_live_sessions() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Mock: GET /sessions returns two live sessions and one dead
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": "live-1", "status": "live", "isLive": true},
            {"id": "live-2", "status": "live", "isLive": true},
            {"id": "dead-1", "status": "released", "isLive": false},
        ])))
        .mount(&server)
        .await;

    mock_release_session(&server, "live-1").await;
    mock_release_session(&server, "live-2").await;

    let result = stop_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        true,
    )
    .await
    .unwrap();

    assert!(result.all);
    assert_eq!(result.stopped_session_ids.len(), 2);
    assert!(result.stopped_session_ids.contains(&"live-1".to_string()));
    assert!(result.stopped_session_ids.contains(&"live-2".to_string()));
}

// ===========================================================================
// 8. Session Stop: clears session from state
// ===========================================================================

#[tokio::test]
async fn stop_clears_session_from_state() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed with an active named session
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "sess-named", Some("work"));
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "sess-named".to_string());
    })
    .unwrap();

    mock_release_session(&server, "sess-named").await;

    stop_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        false,
    )
    .await
    .unwrap();

    // State should be cleared
    let state = read_state(&paths.state_path);
    assert!(state.active_session_id.is_none());
    assert!(state.active_api_mode.is_none());
    assert!(state.active_session_name.is_none());
    assert!(state.named_sessions.cloud.get("work").is_none());
}

// ===========================================================================
// 9. Session List: lists sessions with names resolved from state
// ===========================================================================

#[tokio::test]
async fn list_sessions_resolves_names_from_state() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed state with named sessions
    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "sess-1".to_string());
        state
            .named_sessions
            .cloud
            .insert("dev".to_string(), "sess-2".to_string());
    })
    .unwrap();

    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": "sess-1", "status": "live", "isLive": true},
            {"id": "sess-2", "status": "live", "isLive": true},
            {"id": "sess-3", "status": "live", "isLive": true},
        ])))
        .mount(&server)
        .await;

    let summaries = list_sessions(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
    )
    .await
    .unwrap();

    assert_eq!(summaries.len(), 3);

    let s1 = summaries.iter().find(|s| s.id == "sess-1").unwrap();
    assert_eq!(s1.name.as_deref(), Some("work"));

    let s2 = summaries.iter().find(|s| s.id == "sess-2").unwrap();
    assert_eq!(s2.name.as_deref(), Some("dev"));

    let s3 = summaries.iter().find(|s| s.id == "sess-3").unwrap();
    assert!(s3.name.is_none());
}

// ===========================================================================
// 14. Connect URL Contract: injects apiKey into connect URL for cloud mode
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
    assert_eq!(count, 1, "Expected exactly one apiKey, got {count} in: {url}");
}

// ===========================================================================
// 15. Connect URL Contract: fallback builds wss://connect.steel.dev URL
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
// 16. Connect URL Contract: sanitize_connect_url masks apiKey
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
// 17. Session State: named sessions are stored and retrieved correctly
// ===========================================================================

#[test]
fn named_sessions_stored_and_retrieved() {
    let (_dir, paths) = tmp_paths();

    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "sess-1".to_string());
        state
            .named_sessions
            .cloud
            .insert("dev".to_string(), "sess-2".to_string());
        state
            .named_sessions
            .local
            .insert("local-work".to_string(), "sess-3".to_string());
    })
    .unwrap();

    let state = read_state(&paths.state_path);

    assert_eq!(
        state.named_sessions.cloud.get("work").map(|s| s.as_str()),
        Some("sess-1")
    );
    assert_eq!(
        state.named_sessions.cloud.get("dev").map(|s| s.as_str()),
        Some("sess-2")
    );
    assert_eq!(
        state
            .named_sessions
            .local
            .get("local-work")
            .map(|s| s.as_str()),
        Some("sess-3")
    );
}

#[test]
fn resolve_candidate_returns_named_session() {
    let (_dir, paths) = tmp_paths();

    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "sess-1".to_string());
    })
    .unwrap();

    let id = with_lock(&paths, false, |state| {
        state
            .resolve_candidate(ApiMode::Cloud, Some("work"))
            .map(|s| s.to_string())
    })
    .unwrap();

    assert_eq!(id.as_deref(), Some("sess-1"));
}

#[test]
fn resolve_name_from_state() {
    let (_dir, paths) = tmp_paths();

    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "sess-1".to_string());
    })
    .unwrap();

    let state = read_state(&paths.state_path);
    let name = state.resolve_name(ApiMode::Cloud, "sess-1");
    assert_eq!(name, Some("work"));
}

// ===========================================================================
// 18. Session State: state persists across calls (file-backed)
// ===========================================================================

#[test]
fn state_persists_across_calls() {
    let (_dir, paths) = tmp_paths();

    // First call: write state
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "sess-persist", Some("project-a"));
        state
            .named_sessions
            .cloud
            .insert("project-a".to_string(), "sess-persist".to_string());
    })
    .unwrap();

    // Second call: read state back
    let state = read_state(&paths.state_path);
    assert_eq!(
        state.active_session_id.as_deref(),
        Some("sess-persist")
    );
    assert_eq!(
        state.active_session_name.as_deref(),
        Some("project-a")
    );
    assert_eq!(state.active_api_mode, Some(ApiMode::Cloud));
    assert_eq!(
        state
            .named_sessions
            .cloud
            .get("project-a")
            .map(|s| s.as_str()),
        Some("sess-persist")
    );
}

#[test]
fn state_survives_multiple_mutations() {
    let (_dir, paths) = tmp_paths();

    // Write first named session
    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("alpha".to_string(), "sess-a".to_string());
    })
    .unwrap();

    // Write second named session in separate call
    with_lock(&paths, true, |state| {
        state
            .named_sessions
            .cloud
            .insert("beta".to_string(), "sess-b".to_string());
    })
    .unwrap();

    // Both should be present
    let state = read_state(&paths.state_path);
    assert_eq!(
        state.named_sessions.cloud.get("alpha").map(|s| s.as_str()),
        Some("sess-a")
    );
    assert_eq!(
        state.named_sessions.cloud.get("beta").map(|s| s.as_str()),
        Some("sess-b")
    );
}

// ===========================================================================
// 19. Session State: cross-process state (write in one call, read in another)
// ===========================================================================

#[test]
fn cross_process_state_write_then_read() {
    let (_dir, paths) = tmp_paths();

    // Simulate "process 1": write state
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Local, "cross-sess", Some("shared"));
        state
            .named_sessions
            .local
            .insert("shared".to_string(), "cross-sess".to_string());
    })
    .unwrap();

    // Simulate "process 2": read state from disk (fresh read, no in-memory carry-over)
    let fresh_state = read_state(&paths.state_path);
    assert_eq!(
        fresh_state.active_session_id.as_deref(),
        Some("cross-sess")
    );
    assert_eq!(
        fresh_state.active_api_mode,
        Some(ApiMode::Local)
    );
    assert_eq!(
        fresh_state
            .named_sessions
            .local
            .get("shared")
            .map(|s| s.as_str()),
        Some("cross-sess")
    );

    // "Process 2" can also resolve the candidate
    let candidate = fresh_state
        .resolve_candidate(ApiMode::Local, Some("shared"))
        .map(|s| s.to_string());
    assert_eq!(candidate.as_deref(), Some("cross-sess"));
}

#[test]
fn cross_process_clear_visible_to_other_reader() {
    let (_dir, paths) = tmp_paths();

    // Process 1: create session
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "to-clear", Some("temp"));
        state
            .named_sessions
            .cloud
            .insert("temp".to_string(), "to-clear".to_string());
    })
    .unwrap();

    // Process 2: clear the session
    with_lock(&paths, true, |state| {
        state.clear_active(ApiMode::Cloud, "to-clear");
    })
    .unwrap();

    // Process 3: reads and sees it cleared
    let state = read_state(&paths.state_path);
    assert!(state.active_session_id.is_none());
    assert!(state.named_sessions.cloud.get("temp").is_none());
}

// ===========================================================================
// Additional edge-case and contract tests
// ===========================================================================

#[tokio::test]
async fn stop_with_no_active_session_returns_empty() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // No mocks needed -- nothing should be called
    let result = stop_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        false,
    )
    .await
    .unwrap();

    assert!(result.stopped_session_ids.is_empty());
}

#[tokio::test]
async fn stop_named_session_releases_correct_session() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed two named sessions and an active one
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "active-sess", None);
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "named-sess".to_string());
    })
    .unwrap();

    mock_release_session(&server, "named-sess").await;

    let result = stop_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        Some("work"),
        false,
    )
    .await
    .unwrap();

    assert_eq!(result.stopped_session_ids, vec!["named-sess"]);

    // The active session (different from the named one) should remain
    let state = read_state(&paths.state_path);
    assert_eq!(
        state.active_session_id.as_deref(),
        Some("active-sess"),
        "Active session should remain since we stopped a different named session"
    );
}

#[tokio::test]
async fn start_session_with_dead_unnamed_active_creates_new() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed with a dead unnamed active session
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "dead-active", None);
    })
    .unwrap();

    // GET /sessions/dead-active -> not live
    Mock::given(method("GET"))
        .and(path("/sessions/dead-active"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "dead-active",
            "status": "released",
            "isLive": false,
        })))
        .mount(&server)
        .await;

    mock_create_session(&server, "fresh-sess").await;

    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        &CreateSessionOptions::default(),
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "fresh-sess");

    let state = read_state(&paths.state_path);
    assert_eq!(state.active_session_id.as_deref(), Some("fresh-sess"));
}

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
    let summary =
        to_session_summary(&session, ApiMode::Cloud, Some("my-session"), &auth).unwrap();
    assert_eq!(summary.name.as_deref(), Some("my-session"));
}

#[tokio::test]
async fn list_sessions_empty() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let summaries = list_sessions(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
    )
    .await
    .unwrap();

    assert!(summaries.is_empty());
}

#[tokio::test]
async fn create_session_sends_api_key_header() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    Mock::given(method("POST"))
        .and(path("/sessions"))
        .and(header("Steel-Api-Key", "sk-test-key-12345678"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "auth-sess",
            "status": "live",
            "isLive": true,
        })))
        .mount(&server)
        .await;

    let summary = start_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        &CreateSessionOptions::default(),
    )
    .await
    .unwrap();

    assert_eq!(summary.id, "auth-sess");
}

#[tokio::test]
async fn stop_all_clears_all_from_state() {
    let server = MockServer::start().await;
    let client = new_client();
    let (_dir, paths) = tmp_paths();

    // Pre-seed with an active session
    with_lock(&paths, true, |state| {
        state.set_active(ApiMode::Cloud, "all-1", Some("work"));
        state
            .named_sessions
            .cloud
            .insert("work".to_string(), "all-1".to_string());
    })
    .unwrap();

    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": "all-1", "status": "live", "isLive": true},
        ])))
        .mount(&server)
        .await;

    mock_release_session(&server, "all-1").await;

    stop_session(
        &client,
        &server.uri(),
        ApiMode::Cloud,
        &cloud_auth(),
        &paths,
        None,
        true,
    )
    .await
    .unwrap();

    let state = read_state(&paths.state_path);
    assert!(state.active_session_id.is_none());
    assert!(state.named_sessions.cloud.get("work").is_none());
}
