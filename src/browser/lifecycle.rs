//! Browser session lifecycle orchestration.
//! Combines API client + session state to implement high-level browser operations.
//! Ported from: cli/source/utils/browser/lifecycle.ts + session-policy.ts

use serde_json::Value;

use crate::api::client::{ApiError, SteelClient};
use crate::api::session::CreateSessionOptions;
use crate::config::auth::Auth;
use crate::config::session_state::{SessionState, SessionStatePaths, read_state, with_lock};
use crate::config::settings::ApiMode;

/// Summary of a browser session, matching TS `BrowserSessionSummary`.
#[derive(Debug)]
pub struct SessionSummary {
    pub id: String,
    pub mode: ApiMode,
    pub name: Option<String>,
    pub live: bool,
    pub status: Option<String>,
    pub connect_url: Option<String>,
    pub viewer_url: Option<String>,
    pub profile_id: Option<String>,
}

/// Result of stopping browser sessions.
pub struct StopResult {
    pub mode: ApiMode,
    pub all: bool,
    pub stopped_session_ids: Vec<String>,
}

/// Result of solving a CAPTCHA.
pub struct CaptchaSolveResult {
    pub mode: ApiMode,
    pub session_id: String,
    pub success: bool,
    pub message: Option<String>,
    pub raw: Value,
}

/// Result of checking CAPTCHA status.
pub struct CaptchaStatusResult {
    pub mode: ApiMode,
    pub session_id: String,
    pub status: String,
    pub types: Vec<String>,
    pub raw: Value,
}

const CLOSED_SESSION_STATUSES: &[&str] = &[
    "closed",
    "completed",
    "ended",
    "failed",
    "released",
    "stopped",
    "terminated",
];

/// Extract session ID from API response. Matches TS `getSessionId()`.
pub fn get_session_id(session: &Value) -> Option<String> {
    session
        .get("id")
        .or_else(|| session.get("sessionId"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

/// Check if session is live. Matches TS `isSessionLive()`.
pub fn is_session_live(session: &Value) -> bool {
    let live_keys = ["isLive", "live", "active"];
    for key in &live_keys {
        if let Some(v) = session.get(key) {
            if let Some(b) = v.as_bool() {
                return b;
            }
        }
    }

    if let Some(ended) = session.get("endedAt") {
        if let Some(s) = ended.as_str() {
            if !s.trim().is_empty() {
                return false;
            }
        }
    }

    if let Some(status) = get_session_status(session) {
        return !CLOSED_SESSION_STATUSES.contains(&status.to_lowercase().as_str());
    }

    true
}

fn get_session_status(session: &Value) -> Option<String> {
    session
        .get("status")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

fn get_connect_url(session: &Value) -> Option<String> {
    let keys = [
        "websocketUrl",
        "wsUrl",
        "connectUrl",
        "cdpUrl",
        "browserWSEndpoint",
        "wsEndpoint",
    ];
    for key in &keys {
        if let Some(v) = session.get(key) {
            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

fn get_viewer_url(session: &Value, mode: ApiMode, session_id: &str) -> Option<String> {
    let keys = ["sessionViewerUrl", "viewerUrl", "liveViewUrl"];
    for key in &keys {
        if let Some(v) = session.get(key) {
            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    if mode == ApiMode::Cloud {
        return Some(format!("https://app.steel.dev/sessions/{session_id}"));
    }

    None
}

/// Build session summary from API response. Matches TS `toSessionSummary()`.
pub fn to_session_summary(
    session: &Value,
    mode: ApiMode,
    name: Option<&str>,
    auth: &Auth,
) -> anyhow::Result<SessionSummary> {
    let session_id = get_session_id(session)
        .ok_or_else(|| anyhow::anyhow!("Session response did not contain an id."))?;

    let mut connect_url = get_connect_url(session);

    // Fallback: build connect URL for cloud mode
    if connect_url.is_none() && mode == ApiMode::Cloud {
        if let Some(ref api_key) = auth.api_key {
            connect_url = Some(format!(
                "wss://connect.steel.dev?apiKey={api_key}&sessionId={session_id}"
            ));
        }
    }

    // Inject apiKey into connect URL for cloud mode
    if let Some(ref url) = connect_url {
        if mode == ApiMode::Cloud {
            if let Some(ref api_key) = auth.api_key {
                if !url.contains("apiKey=") {
                    if let Ok(mut parsed) = url::Url::parse(url) {
                        parsed.query_pairs_mut().append_pair("apiKey", api_key);
                        connect_url = Some(parsed.to_string());
                    }
                }
            }
        }
    }

    Ok(SessionSummary {
        id: session_id.clone(),
        mode,
        name: name.map(|s| s.to_string()),
        live: is_session_live(session),
        status: get_session_status(session),
        connect_url,
        viewer_url: get_viewer_url(session, mode, &session_id),
        profile_id: session
            .get("profileId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

/// Try to get a live session, returning None if not found or not live.
async fn try_get_live_session(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    session_id: &str,
    auth: &Auth,
) -> Result<Option<Value>, ApiError> {
    match client.get_session(base_url, mode, session_id, auth).await {
        Ok(session) => {
            if is_session_live(&session) {
                Ok(Some(session))
            } else {
                Ok(None)
            }
        }
        Err(e) if e.is_not_found() => Ok(None),
        Err(e) => Err(e),
    }
}

/// Resolve target session from state (for stop/live/captcha commands).
fn resolve_target_session(
    state: &SessionState,
    mode: ApiMode,
    session_name: Option<&str>,
) -> (Option<String>, Option<String>) {
    if let Some(name) = session_name {
        let name = name.trim();
        if !name.is_empty() {
            let session_id = state.named_sessions.get(mode).get(name).cloned();
            return (session_id, Some(name.to_string()));
        }
    }

    if state.active_api_mode == Some(mode) {
        if let Some(ref id) = state.active_session_id {
            return (Some(id.clone()), state.active_session_name.clone());
        }
    }

    (None, None)
}

/// Resolve session ID for captcha commands (supports explicit --session-id).
fn resolve_captcha_session_id(
    state: &SessionState,
    mode: ApiMode,
    session_id: Option<&str>,
    session_name: Option<&str>,
) -> Option<String> {
    if let Some(id) = session_id {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let (id, _) = resolve_target_session(state, mode, session_name);
    id
}

/// Start a browser session. Matches TS `startBrowserSession()`.
pub async fn start_session(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    paths: &SessionStatePaths,
    session_name: Option<&str>,
    options: &CreateSessionOptions,
) -> anyhow::Result<SessionSummary> {
    let session_name = session_name.map(|s| s.trim()).filter(|s| !s.is_empty());

    loop {
        // Check for existing candidate
        let candidate_id = with_lock(paths, false, |state| {
            state
                .resolve_candidate(mode, session_name)
                .map(|s| s.to_string())
        })?;

        if let Some(ref candidate_id) = candidate_id {
            // Try to attach to existing session
            let existing = try_get_live_session(client, base_url, mode, candidate_id, auth)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            if let Some(ref session) = existing {
                // Claim the session under lock
                let claimed = with_lock(paths, true, |state| {
                    let latest = state
                        .resolve_candidate(mode, session_name)
                        .map(|s| s.to_string());
                    if latest.as_deref() != Some(candidate_id) {
                        return false;
                    }
                    state.set_active(mode, candidate_id, session_name);
                    true
                })?;

                if claimed {
                    return to_session_summary(session, mode, session_name, auth);
                }
                continue;
            }

            // Dead session — clear and retry
            with_lock(paths, true, |state| {
                let latest = state
                    .resolve_candidate(mode, session_name)
                    .map(|s| s.to_string());
                if latest.as_deref() == Some(candidate_id) {
                    state.clear_active(mode, candidate_id);
                }
            })?;
            continue;
        }

        // No candidate — create new session
        let created = client
            .create_session(base_url, mode, options, auth)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let created_id = get_session_id(&created)
            .ok_or_else(|| anyhow::anyhow!("API did not return a session id."))?;

        // Claim the created session under lock
        let claimed = with_lock(paths, true, |state| {
            let latest = state
                .resolve_candidate(mode, session_name)
                .map(|s| s.to_string());
            if latest.is_some() {
                return false;
            }

            if let Some(name) = session_name {
                state
                    .named_sessions
                    .get_mut(mode)
                    .insert(name.to_string(), created_id.clone());
            }
            state.set_active(mode, &created_id, session_name);
            true
        })?;

        if claimed {
            return to_session_summary(&created, mode, session_name, auth);
        }

        // Race: another session appeared — release ours and retry
        let _ = client
            .release_session(base_url, mode, &created_id, auth)
            .await;
    }
}

/// Stop browser sessions. Matches TS `stopBrowserSession()`.
pub async fn stop_session(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    paths: &SessionStatePaths,
    session_name: Option<&str>,
    all: bool,
) -> anyhow::Result<StopResult> {
    if all {
        let sessions = client
            .list_sessions(base_url, mode, auth)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let live_ids: Vec<String> = sessions
            .iter()
            .filter(|s| is_session_live(s))
            .filter_map(|s| get_session_id(s))
            .collect();

        for id in &live_ids {
            let _ = client.release_session(base_url, mode, id, auth).await;
        }

        with_lock(paths, true, |state| {
            for id in &live_ids {
                state.clear_active(mode, id);
            }
        })?;

        return Ok(StopResult {
            mode,
            all: true,
            stopped_session_ids: live_ids,
        });
    }

    let target_id = with_lock(paths, false, |state| {
        let (id, _) = resolve_target_session(state, mode, session_name);
        id
    })?;

    let Some(target_id) = target_id else {
        return Ok(StopResult {
            mode,
            all: false,
            stopped_session_ids: vec![],
        });
    };

    client
        .release_session(base_url, mode, &target_id, auth)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    with_lock(paths, true, |state| {
        state.clear_active(mode, &target_id);
    })?;

    Ok(StopResult {
        mode,
        all: false,
        stopped_session_ids: vec![target_id],
    })
}

/// List browser sessions with names resolved from state.
pub async fn list_sessions(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    paths: &SessionStatePaths,
) -> anyhow::Result<Vec<SessionSummary>> {
    let sessions = client
        .list_sessions(base_url, mode, auth)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let state = read_state(&paths.state_path);

    let mut summaries = Vec::new();
    for session in &sessions {
        let id = get_session_id(session);
        let name = id
            .as_deref()
            .and_then(|id| state.resolve_name(mode, id))
            .map(|s| s.to_string());
        let summary = to_session_summary(session, mode, name.as_deref(), auth)?;
        summaries.push(summary);
    }

    Ok(summaries)
}

/// Get the live URL for the active session.
pub async fn get_live_url(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    paths: &SessionStatePaths,
    session_name: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let target_id = with_lock(paths, false, |state| {
        let (id, _) = resolve_target_session(state, mode, session_name);
        id
    })?;

    let Some(target_id) = target_id else {
        return Ok(None);
    };

    let session = try_get_live_session(client, base_url, mode, &target_id, auth)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let Some(session) = session else {
        return Ok(None);
    };

    let summary = to_session_summary(&session, mode, None, auth)?;
    Ok(summary.viewer_url)
}

/// Solve a CAPTCHA for a browser session.
pub async fn solve_captcha(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    paths: &SessionStatePaths,
    explicit_session_id: Option<&str>,
    session_name: Option<&str>,
    page_id: Option<&str>,
    url: Option<&str>,
    task_id: Option<&str>,
) -> anyhow::Result<CaptchaSolveResult> {
    let session_id = with_lock(paths, false, |state| {
        resolve_captcha_session_id(state, mode, explicit_session_id, session_name)
    })?;

    let session_id = session_id.ok_or_else(|| {
        anyhow::anyhow!(
            "No target browser session found for CAPTCHA solving. \
             Pass `--session-id`, pass `--session <name>`, or start a session first."
        )
    })?;

    let raw = client
        .solve_captcha(base_url, mode, &session_id, page_id, url, task_id, auth)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let success = raw
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let message = raw
        .get("message")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(CaptchaSolveResult {
        mode,
        session_id,
        success,
        message,
        raw,
    })
}

/// Get CAPTCHA status for a browser session.
pub async fn captcha_status(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    paths: &SessionStatePaths,
    explicit_session_id: Option<&str>,
    session_name: Option<&str>,
    page_id: Option<&str>,
    wait: bool,
    timeout_ms: Option<u64>,
    interval_ms: Option<u64>,
) -> anyhow::Result<CaptchaStatusResult> {
    let timeout = timeout_ms.unwrap_or(60_000);
    let interval = interval_ms.unwrap_or(1_000);

    let session_id = with_lock(paths, false, |state| {
        resolve_captcha_session_id(state, mode, explicit_session_id, session_name)
    })?;

    let session_id = session_id.ok_or_else(|| {
        anyhow::anyhow!(
            "No target browser session found for CAPTCHA status. \
             Pass `--session-id`, pass `--session <name>`, or start a session first."
        )
    })?;

    let start = std::time::Instant::now();

    loop {
        let pages = client
            .captcha_status(base_url, mode, &session_id, page_id, auth)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let (status, types) = normalize_captcha_status(&pages);

        if !wait || is_terminal_captcha_status(&status) {
            return Ok(CaptchaStatusResult {
                mode,
                session_id,
                status,
                types,
                raw: serde_json::json!({"pages": pages}),
            });
        }

        if start.elapsed().as_millis() as u64 >= timeout {
            anyhow::bail!(
                "CAPTCHA status polling timed out after {timeout}ms. Last status: {status}"
            );
        }

        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    }
}

const KNOWN_CAPTCHA_TYPES: &[&str] = &["recaptchaV2", "recaptchaV3", "turnstile", "image_to_text"];

fn normalize_captcha_status(pages: &[Value]) -> (String, Vec<String>) {
    if pages.is_empty() {
        return ("none".to_string(), vec![]);
    }

    let mut all_tasks: Vec<&Value> = Vec::new();
    let mut any_solving = false;

    for page in pages {
        if page.get("isSolvingCaptcha").and_then(|v| v.as_bool()) == Some(true) {
            any_solving = true;
        }

        if let Some(tasks) = page.get("tasks").and_then(|v| v.as_array()) {
            for task in tasks {
                if task.is_object() {
                    all_tasks.push(task);
                }
            }
        }
    }

    if all_tasks.is_empty() {
        let status = if any_solving { "solving" } else { "none" };
        return (status.to_string(), vec![]);
    }

    let mut solving_types: Vec<String> = Vec::new();
    let mut failed_types: Vec<String> = Vec::new();
    let mut has_solving = false;
    let mut has_failed = false;
    let mut has_solved = false;

    for task in &all_tasks {
        let task_status = task.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let task_type = task.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let is_known = KNOWN_CAPTCHA_TYPES.contains(&task_type);

        match task_status {
            "solving" | "detected" | "validating" => {
                has_solving = true;
                if is_known && !solving_types.contains(&task_type.to_string()) {
                    solving_types.push(task_type.to_string());
                }
            }
            "failed_to_solve" | "failed_to_detect" | "validation_failed" => {
                has_failed = true;
                if is_known && !failed_types.contains(&task_type.to_string()) {
                    failed_types.push(task_type.to_string());
                }
            }
            "solved" => {
                has_solved = true;
            }
            _ => {}
        }
    }

    if has_solving || any_solving {
        return ("solving".to_string(), solving_types);
    }

    if has_failed {
        return ("failed".to_string(), failed_types);
    }

    if has_solved {
        return ("solved".to_string(), vec![]);
    }

    ("none".to_string(), vec![])
}

fn is_terminal_captcha_status(status: &str) -> bool {
    matches!(status, "solved" | "failed" | "none")
}

/// Sanitize connect URL for display (mask API key).
pub fn sanitize_connect_url(url: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(url) {
        let pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .map(|(k, v)| {
                if k == "apiKey" && v.len() > 7 {
                    (k.to_string(), format!("{}...", &v[..7]))
                } else {
                    (k.to_string(), v.to_string())
                }
            })
            .collect();

        if !pairs.is_empty() {
            parsed.query_pairs_mut().clear();
            for (k, v) in &pairs {
                parsed.query_pairs_mut().append_pair(k, v);
            }
        }

        parsed.to_string()
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_session_id_from_id() {
        let s = json!({"id": "s1"});
        assert_eq!(get_session_id(&s).as_deref(), Some("s1"));
    }

    #[test]
    fn get_session_id_from_session_id() {
        let s = json!({"sessionId": "s2"});
        assert_eq!(get_session_id(&s).as_deref(), Some("s2"));
    }

    #[test]
    fn get_session_id_empty() {
        let s = json!({"id": " "});
        assert_eq!(get_session_id(&s), None);
    }

    #[test]
    fn session_live_by_flag() {
        assert!(is_session_live(&json!({"isLive": true})));
        assert!(!is_session_live(&json!({"isLive": false})));
    }

    #[test]
    fn session_live_by_status() {
        assert!(is_session_live(&json!({"status": "running"})));
        assert!(!is_session_live(&json!({"status": "released"})));
        assert!(!is_session_live(&json!({"status": "terminated"})));
    }

    #[test]
    fn session_live_ended_at() {
        assert!(!is_session_live(&json!({"endedAt": "2025-01-01"})));
    }

    #[test]
    fn session_live_default() {
        assert!(is_session_live(&json!({"id": "s1"})));
    }

    #[test]
    fn viewer_url_cloud_fallback() {
        let url = get_viewer_url(&json!({}), ApiMode::Cloud, "s1");
        assert_eq!(url.as_deref(), Some("https://app.steel.dev/sessions/s1"));
    }

    #[test]
    fn viewer_url_local_none() {
        let url = get_viewer_url(&json!({}), ApiMode::Local, "s1");
        assert_eq!(url, None);
    }

    #[test]
    fn viewer_url_from_response() {
        let url = get_viewer_url(
            &json!({"sessionViewerUrl": "https://custom.dev/view"}),
            ApiMode::Local,
            "s1",
        );
        assert_eq!(url.as_deref(), Some("https://custom.dev/view"));
    }

    #[test]
    fn connect_url_from_response() {
        let url = get_connect_url(&json!({"websocketUrl": "wss://foo.dev/ws"}));
        assert_eq!(url.as_deref(), Some("wss://foo.dev/ws"));
    }

    #[test]
    fn sanitize_masks_api_key() {
        let url = "wss://connect.steel.dev?apiKey=sk-12345678&sessionId=s1";
        let sanitized = sanitize_connect_url(url);
        assert!(sanitized.contains("sk-1234..."));
        assert!(sanitized.contains("sessionId=s1"));
    }

    #[test]
    fn normalize_captcha_none() {
        let (status, types) = normalize_captcha_status(&[]);
        assert_eq!(status, "none");
        assert!(types.is_empty());
    }

    #[test]
    fn normalize_captcha_solving() {
        let pages = vec![json!({
            "isSolvingCaptcha": true,
            "tasks": [{"status": "solving", "type": "turnstile"}]
        })];
        let (status, types) = normalize_captcha_status(&pages);
        assert_eq!(status, "solving");
        assert_eq!(types, vec!["turnstile"]);
    }

    #[test]
    fn normalize_captcha_solved() {
        let pages = vec![json!({
            "tasks": [{"status": "solved", "type": "recaptchaV2"}]
        })];
        let (status, _types) = normalize_captcha_status(&pages);
        assert_eq!(status, "solved");
    }

    #[test]
    fn normalize_captcha_failed() {
        let pages = vec![json!({
            "tasks": [{"status": "failed_to_solve", "type": "turnstile"}]
        })];
        let (status, types) = normalize_captcha_status(&pages);
        assert_eq!(status, "failed");
        assert_eq!(types, vec!["turnstile"]);
    }
}
