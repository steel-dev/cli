//! Browser session lifecycle utilities.
//! Provides helpers for parsing API responses, CAPTCHA operations, and URL handling.

use serde_json::Value;

use crate::api::client::SteelClient;
use crate::config::auth::Auth;
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
        if let Some(v) = session.get(key)
            && let Some(b) = v.as_bool()
        {
            return b;
        }
    }

    if let Some(ended) = session.get("endedAt")
        && let Some(s) = ended.as_str()
        && !s.trim().is_empty()
    {
        return false;
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

/// Extract session timeout (milliseconds) from API response.
/// The API may return it as `timeout`, `sessionTimeout`, or `timeoutMs`.
pub fn get_session_timeout(session: &Value) -> Option<u64> {
    let keys = ["timeout", "sessionTimeout", "timeoutMs"];
    for key in &keys {
        if let Some(v) = session.get(key) {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
            // Handle string numbers
            if let Some(s) = v.as_str()
                && let Ok(n) = s.trim().parse::<u64>()
            {
                return Some(n);
            }
        }
    }
    None
}

/// Extract session creation timestamp as epoch milliseconds from API response.
/// Looks for `createdAt` or `created_at` as ISO 8601 / RFC 3339 string.
pub fn get_session_created_at_ms(session: &Value) -> Option<u64> {
    let keys = ["createdAt", "created_at"];
    for key in &keys {
        if let Some(v) = session.get(key)
            && let Some(s) = v.as_str()
        {
            let trimmed = s.trim();
            if !trimmed.is_empty()
                && let Ok(ts) = jiff::Timestamp::strptime("%Y-%m-%dT%H:%M:%S%.fZ", trimmed)
                    .or_else(|_| trimmed.parse::<jiff::Timestamp>())
            {
                return Some(ts.as_millisecond() as u64);
            }
        }
    }
    None
}

pub fn get_connect_url(session: &Value) -> Option<String> {
    let keys = [
        "websocketUrl",
        "wsUrl",
        "connectUrl",
        "cdpUrl",
        "browserWSEndpoint",
        "wsEndpoint",
    ];
    for key in &keys {
        if let Some(v) = session.get(key)
            && let Some(s) = v.as_str()
        {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

pub fn get_viewer_url(session: &Value, mode: ApiMode, session_id: &str) -> Option<String> {
    let keys = ["sessionViewerUrl", "viewerUrl", "liveViewUrl"];
    for key in &keys {
        if let Some(v) = session.get(key)
            && let Some(s) = v.as_str()
        {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
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
    if connect_url.is_none()
        && mode == ApiMode::Cloud
        && let Some(ref api_key) = auth.api_key
    {
        connect_url = Some(format!(
            "wss://connect.steel.dev?apiKey={api_key}&sessionId={session_id}"
        ));
    }

    // Inject apiKey into connect URL for cloud mode
    if let Some(ref url) = connect_url
        && mode == ApiMode::Cloud
        && let Some(ref api_key) = auth.api_key
        && !url.contains("apiKey=")
        && let Ok(mut parsed) = url::Url::parse(url)
    {
        parsed.query_pairs_mut().append_pair("apiKey", api_key);
        connect_url = Some(parsed.to_string());
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

/// Solve a CAPTCHA for a browser session.
pub async fn solve_captcha(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    session_id: &str,
    page_id: Option<&str>,
    url: Option<&str>,
    task_id: Option<&str>,
) -> anyhow::Result<CaptchaSolveResult> {
    let raw = client
        .solve_captcha(base_url, mode, session_id, page_id, url, task_id, auth)
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
        session_id: session_id.to_string(),
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
    session_id: &str,
    page_id: Option<&str>,
    wait: bool,
    timeout_ms: Option<u64>,
    interval_ms: Option<u64>,
) -> anyhow::Result<CaptchaStatusResult> {
    let timeout = timeout_ms.unwrap_or(60_000);
    let interval = interval_ms.unwrap_or(1_000);

    let start = std::time::Instant::now();

    loop {
        let pages = client
            .captcha_status(base_url, mode, session_id, page_id, auth)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let (status, types) = normalize_captcha_status(&pages);

        if !wait || is_terminal_captcha_status(&status) {
            return Ok(CaptchaStatusResult {
                mode,
                session_id: session_id.to_string(),
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
    fn session_timeout_from_response() {
        assert_eq!(
            get_session_timeout(&json!({"timeout": 300000})),
            Some(300000)
        );
        assert_eq!(
            get_session_timeout(&json!({"sessionTimeout": 60000})),
            Some(60000)
        );
        assert_eq!(
            get_session_timeout(&json!({"timeoutMs": 120000})),
            Some(120000)
        );
    }

    #[test]
    fn session_timeout_string_number() {
        assert_eq!(
            get_session_timeout(&json!({"timeout": "300000"})),
            Some(300000)
        );
    }

    #[test]
    fn session_timeout_missing() {
        assert_eq!(get_session_timeout(&json!({"id": "s1"})), None);
    }

    #[test]
    fn session_created_at_iso() {
        let s = json!({"createdAt": "2025-01-15T10:30:00Z"});
        let ms = get_session_created_at_ms(&s).unwrap();
        // 2025-01-15T10:30:00Z in epoch ms
        assert!(ms > 1_700_000_000_000);
        assert!(ms < 1_800_000_000_000);
    }

    #[test]
    fn session_created_at_with_fractional() {
        let s = json!({"createdAt": "2025-01-15T10:30:00.123Z"});
        let ms = get_session_created_at_ms(&s).unwrap();
        assert!(ms > 1_700_000_000_000);
    }

    #[test]
    fn session_created_at_missing() {
        assert_eq!(get_session_created_at_ms(&json!({"id": "s1"})), None);
    }

    #[test]
    fn session_created_at_snake_case() {
        let s = json!({"created_at": "2025-01-15T10:30:00Z"});
        assert!(get_session_created_at_ms(&s).is_some());
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
