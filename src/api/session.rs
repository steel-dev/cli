use anyhow::Context;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

const CLOUD_CONNECT_URL: &str = "wss://connect.steel.dev";
const CLOUD_VIEWER_URL: &str = "https://app.steel.dev/sessions";

#[derive(Debug, Default)]
pub struct CreateSessionOptions {
    pub stealth: bool,
    pub proxy_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub inactivity_timeout_ms: Option<u64>,
    pub headless: Option<bool>,
    pub region: Option<String>,
    pub solve_captcha: bool,
    pub profile_id: Option<String>,
    pub persist_profile: bool,
    pub namespace: Option<String>,
    pub credentials: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub status: Option<String>,
    pub websocket_url: Option<String>,
    pub session_viewer_url: Option<String>,
    pub timeout: Option<u64>,
    pub inactivity_timeout: Option<u64>,
    pub created_at: Option<String>,
    pub profile_id: Option<String>,
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.trim().is_empty())
}

impl Session {
    pub fn parse(value: Value) -> anyhow::Result<Self> {
        serde_json::from_value(value).context("The Steel API returned an unexpected session.")
    }

    pub fn is_live(&self) -> bool {
        !matches!(self.status.as_deref(), Some("released" | "failed"))
    }

    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout.filter(|&ms| ms > 0)
    }

    pub fn inactivity_timeout_ms(&self) -> Option<u64> {
        self.inactivity_timeout.filter(|&ms| ms > 0)
    }

    pub fn created_at_ms(&self) -> Option<u64> {
        let created = self
            .created_at
            .as_deref()?
            .parse::<jiff::Timestamp>()
            .ok()?;
        u64::try_from(created.as_millisecond()).ok()
    }

    pub fn viewer_url(&self, mode: ApiMode) -> Option<String> {
        non_empty(self.session_viewer_url.as_deref())
            .map(str::to_string)
            .or_else(|| (mode == ApiMode::Cloud).then(|| format!("{CLOUD_VIEWER_URL}/{}", self.id)))
    }

    pub fn connect_url(&self, mode: ApiMode, auth: &Auth) -> Option<String> {
        let given = non_empty(self.websocket_url.as_deref()).map(str::to_string);
        let (ApiMode::Cloud, Some(key)) = (mode, auth.api_key.as_deref()) else {
            return given;
        };
        let Some(url) = given else {
            return Some(format!(
                "{CLOUD_CONNECT_URL}?apiKey={key}&sessionId={}",
                self.id
            ));
        };
        if url.contains("apiKey=") {
            return Some(url);
        }
        let Ok(mut parsed) = url::Url::parse(&url) else {
            return Some(url);
        };
        parsed.query_pairs_mut().append_pair("apiKey", key);
        Some(parsed.to_string())
    }
}

/// Build the JSON body for session creation.
fn build_create_body(options: &CreateSessionOptions) -> Value {
    let mut payload = json!({});
    let obj = payload.as_object_mut().unwrap();

    if let Some(ref proxy_url) = options.proxy_url {
        let trimmed = proxy_url.trim();
        if !trimmed.is_empty() {
            obj.insert("proxyUrl".into(), json!(trimmed));
        }
    }

    if let Some(timeout) = options.timeout_ms {
        obj.insert("timeout".into(), json!(timeout));
    }

    if let Some(inactivity_timeout) = options.inactivity_timeout_ms {
        obj.insert("inactivityTimeout".into(), json!(inactivity_timeout));
    }

    if let Some(headless) = options.headless {
        obj.insert("headless".into(), json!(headless));
    }

    if let Some(ref region) = options.region {
        let trimmed = region.trim();
        if !trimmed.is_empty() {
            obj.insert("region".into(), json!(trimmed));
        }
    }

    if options.stealth {
        obj.insert(
            "stealthConfig".into(),
            json!({"humanizeInteractions": true, "autoCaptchaSolving": true}),
        );
        obj.insert("solveCaptcha".into(), json!(true));
    }

    if options.solve_captcha {
        obj.insert("solveCaptcha".into(), json!(true));
    }

    if let Some(ref profile_id) = options.profile_id {
        obj.insert("profileId".into(), json!(profile_id));
    }

    if options.persist_profile {
        obj.insert("persistProfile".into(), json!(true));
    }

    if let Some(ref namespace) = options.namespace {
        let trimmed = namespace.trim();
        if !trimmed.is_empty() {
            obj.insert("namespace".into(), json!(trimmed));
        }
    }

    if options.credentials {
        obj.insert("credentials".into(), json!({}));
    }

    payload
}

impl SteelClient {
    pub async fn create_session(&self, options: &CreateSessionOptions) -> anyhow::Result<Session> {
        let body = build_create_body(options);
        let data = self
            .request(reqwest::Method::POST, "/sessions", Some(body))
            .await?;
        Session::parse(data)
    }

    pub async fn solve_captcha(
        &self,
        session_id: &str,
        page_id: Option<&str>,
        url: Option<&str>,
        task_id: Option<&str>,
    ) -> Result<Value, ApiError> {
        let mut payload = json!({});
        let obj = payload.as_object_mut().unwrap();

        if let Some(pid) = page_id {
            let trimmed = pid.trim();
            if !trimmed.is_empty() {
                obj.insert("pageId".into(), json!(trimmed));
            }
        }
        if let Some(u) = url {
            let trimmed = u.trim();
            if !trimmed.is_empty() {
                obj.insert("url".into(), json!(trimmed));
            }
        }
        if let Some(tid) = task_id {
            let trimmed = tid.trim();
            if !trimmed.is_empty() {
                obj.insert("taskId".into(), json!(trimmed));
            }
        }

        self.request(
            reqwest::Method::POST,
            &format!("/sessions/{session_id}/captchas/solve"),
            Some(payload),
        )
        .await
    }

    pub async fn captcha_status(
        &self,
        session_id: &str,
        page_id: Option<&str>,
    ) -> Result<Vec<Value>, ApiError> {
        let path = match page_id {
            Some(pid) if !pid.trim().is_empty() => {
                let encoded = urlencoding::encode(pid.trim());
                format!("/sessions/{session_id}/captchas/status?pageId={encoded}")
            }
            _ => format!("/sessions/{session_id}/captchas/status"),
        };

        let data = self.request(reqwest::Method::GET, &path, None).await?;

        if let Some(arr) = data.as_array() {
            return Ok(arr.iter().filter(|v| v.is_object()).cloned().collect());
        }
        if data.is_object() {
            return Ok(vec![data]);
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(value: Value) -> Session {
        Session::parse(value).unwrap()
    }

    fn key(api_key: Option<&str>) -> Auth {
        Auth {
            api_key: api_key.map(str::to_string),
            source: crate::config::auth::AuthSource::Env,
        }
    }

    #[test]
    fn parse_needs_an_id() {
        assert!(Session::parse(json!({"status": "live"})).is_err());
        assert_eq!(session(json!({"id": "s1"})).id, "s1");
    }

    #[test]
    fn released_and_failed_sessions_are_not_live() {
        assert!(session(json!({"id": "s1", "status": "live"})).is_live());
        assert!(session(json!({"id": "s1", "status": "idle"})).is_live());
        assert!(session(json!({"id": "s1"})).is_live());
        assert!(!session(json!({"id": "s1", "status": "released"})).is_live());
        assert!(!session(json!({"id": "s1", "status": "failed"})).is_live());
    }

    #[test]
    fn zero_timeouts_mean_no_limit() {
        let s = session(json!({"id": "s1", "timeout": 300000, "inactivityTimeout": 120000}));
        assert_eq!(s.timeout_ms(), Some(300000));
        assert_eq!(s.inactivity_timeout_ms(), Some(120000));
        let s = session(json!({"id": "s1", "timeout": 0, "inactivityTimeout": 0}));
        assert_eq!(s.timeout_ms(), None);
        assert_eq!(s.inactivity_timeout_ms(), None);
        assert_eq!(session(json!({"id": "s1"})).timeout_ms(), None);
    }

    #[test]
    fn created_at_is_read_as_epoch_ms() {
        let s = session(json!({"id": "s1", "createdAt": "2025-01-15T10:30:00Z"}));
        assert_eq!(s.created_at_ms(), Some(1_736_937_000_000));
        let s = session(json!({"id": "s1", "createdAt": "2025-01-15T10:30:00.123Z"}));
        assert_eq!(s.created_at_ms(), Some(1_736_937_000_123));
        assert_eq!(session(json!({"id": "s1"})).created_at_ms(), None);
    }

    #[test]
    fn viewer_url_falls_back_to_the_cloud_dashboard() {
        let given = session(json!({"id": "s1", "sessionViewerUrl": "https://custom.dev/view"}));
        assert_eq!(
            given.viewer_url(ApiMode::Local).as_deref(),
            Some("https://custom.dev/view")
        );
        let bare = session(json!({"id": "s1"}));
        assert_eq!(
            bare.viewer_url(ApiMode::Cloud).as_deref(),
            Some("https://app.steel.dev/sessions/s1")
        );
        assert_eq!(bare.viewer_url(ApiMode::Local), None);
    }

    #[test]
    fn cloud_connect_urls_carry_the_api_key_once() {
        let auth = key(Some("sk-test"));
        let fresh =
            session(json!({"id": "s1", "websocketUrl": "wss://connect.steel.dev?sessionId=s1"}));
        let url = fresh.connect_url(ApiMode::Cloud, &auth).unwrap();
        assert!(url.contains("apiKey=sk-test"));
        assert!(url.contains("sessionId=s1"));

        let keyed = session(
            json!({"id": "s1", "websocketUrl": "wss://connect.steel.dev?apiKey=old&sessionId=s1"}),
        );
        let url = keyed.connect_url(ApiMode::Cloud, &auth).unwrap();
        assert_eq!(url.matches("apiKey").count(), 1);
        assert!(url.contains("apiKey=old"));

        let local =
            session(json!({"id": "s1", "websocketUrl": "ws://localhost:3000/ws?sessionId=s1"}));
        assert!(
            !local
                .connect_url(ApiMode::Local, &auth)
                .unwrap()
                .contains("apiKey")
        );
    }

    #[test]
    fn cloud_connect_url_is_built_when_missing() {
        let bare = session(json!({"id": "s1"}));
        let url = bare
            .connect_url(ApiMode::Cloud, &key(Some("sk-test")))
            .unwrap();
        assert_eq!(url, "wss://connect.steel.dev?apiKey=sk-test&sessionId=s1");
        assert_eq!(bare.connect_url(ApiMode::Cloud, &key(None)), None);
        assert_eq!(
            bare.connect_url(ApiMode::Local, &key(Some("sk-test"))),
            None
        );
    }

    // --- build_create_body ---

    #[test]
    fn create_body_empty() {
        let body = build_create_body(&CreateSessionOptions::default());
        assert_eq!(body, json!({}));
    }

    #[test]
    fn create_body_stealth() {
        let body = build_create_body(&CreateSessionOptions {
            stealth: true,
            ..Default::default()
        });
        assert_eq!(body["solveCaptcha"], true);
        assert!(body["stealthConfig"].is_object());
    }

    #[test]
    fn create_body_all_fields() {
        let body = build_create_body(&CreateSessionOptions {
            proxy_url: Some("http://proxy:8080".into()),
            timeout_ms: Some(30000),
            inactivity_timeout_ms: Some(120000),
            headless: Some(true),
            region: Some("us-east-1".into()),
            solve_captcha: true,
            namespace: Some("ns".into()),
            credentials: true,
            ..Default::default()
        });
        assert_eq!(body["proxyUrl"], "http://proxy:8080");
        assert_eq!(body["timeout"], 30000);
        assert_eq!(body["inactivityTimeout"], 120000);
        assert_eq!(body["headless"], true);
        assert_eq!(body["region"], "us-east-1");
        assert_eq!(body["solveCaptcha"], true);
        assert_eq!(body["namespace"], "ns");
        assert!(body["credentials"].is_object());
    }

    #[test]
    fn create_body_inactivity_timeout() {
        let body = build_create_body(&CreateSessionOptions {
            inactivity_timeout_ms: Some(120000),
            ..Default::default()
        });
        assert_eq!(body["inactivityTimeout"], 120000);
    }

    #[test]
    fn create_body_trims_whitespace() {
        let body = build_create_body(&CreateSessionOptions {
            proxy_url: Some("  ".into()),
            region: Some("  ".into()),
            namespace: Some("  ".into()),
            ..Default::default()
        });
        // Empty strings should not appear
        assert!(body.get("proxyUrl").is_none());
        assert!(body.get("region").is_none());
        assert!(body.get("namespace").is_none());
    }

    // --- Integration tests with wiremock ---

    use crate::config::auth::AuthSource;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> SteelClient {
        let auth = Auth {
            api_key: Some("test-key".into()),
            source: AuthSource::Env,
        };
        SteelClient::with(server.uri(), ApiMode::Local, auth).unwrap()
    }

    #[tokio::test]
    async fn create_session_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/sessions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"id": "new-sess", "status": "live"})),
            )
            .mount(&server)
            .await;

        let session = client(&server)
            .create_session(&CreateSessionOptions::default())
            .await
            .unwrap();

        assert_eq!(session.id, "new-sess");
    }

    #[tokio::test]
    async fn api_error_extracts_message() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/sessions/s1"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(json!({"message": "Invalid API key"})),
            )
            .mount(&server)
            .await;

        let err = client(&server).cli_get_session("s1").await.unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("403"), "expected 403 in: {msg}");
        assert!(
            msg.contains("Invalid API key"),
            "expected error message in: {msg}"
        );
    }

    #[tokio::test]
    async fn get_session_404_is_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/sessions/gone"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "Not found"})))
            .mount(&server)
            .await;

        let err = client(&server).cli_get_session("gone").await.unwrap_err();

        assert!(err.is_not_found());
    }
}
