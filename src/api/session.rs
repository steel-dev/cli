//! Session API: create, list, get, release, captcha.
//! Ported from: cli/source/utils/browser/lifecycle/api-client.ts

use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

/// Options for creating a session. Matches TS `StartSessionRequestOptions`.
#[derive(Debug, Default)]
pub struct CreateSessionOptions {
    pub stealth: bool,
    pub proxy_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub headless: Option<bool>,
    pub region: Option<String>,
    pub solve_captcha: bool,
    pub profile_id: Option<String>,
    pub persist_profile: bool,
    pub namespace: Option<String>,
    pub credentials: bool,
}

/// Extract session list from response. Matches TS `extractSessionList()`.
fn extract_session_list(data: &Value) -> Vec<Value> {
    // Direct array
    if let Some(arr) = data.as_array() {
        return arr.iter().filter(|v| v.is_object()).cloned().collect();
    }

    // Nested { sessions: [...] }
    if let Some(obj) = data.as_object() {
        if let Some(Value::Array(arr)) = obj.get("sessions") {
            return arr.iter().filter(|v| v.is_object()).cloned().collect();
        }

        // Single session object with id
        if obj.contains_key("id") {
            return vec![data.clone()];
        }
    }

    vec![]
}

/// Extract a single session from response. Matches TS `extractSingleSession()`.
fn extract_single_session(data: &Value) -> Result<Value, ApiError> {
    if let Some(obj) = data.as_object() {
        // Nested { session: {...} }
        if let Some(session) = obj.get("session")
            && session.is_object()
        {
            return Ok(session.clone());
        }
        return Ok(data.clone());
    }

    Err(ApiError::RequestFailed {
        status: 0,
        message: "Unexpected empty response from Steel session API.".into(),
        body: None,
    })
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
    pub async fn list_sessions(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
    ) -> Result<Vec<Value>, ApiError> {
        let data = self
            .request(
                base_url,
                mode,
                reqwest::Method::GET,
                "/sessions",
                None,
                auth,
            )
            .await?;
        Ok(extract_session_list(&data))
    }

    pub async fn get_session(
        &self,
        base_url: &str,
        mode: ApiMode,
        session_id: &str,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        let data = self
            .request(
                base_url,
                mode,
                reqwest::Method::GET,
                &format!("/sessions/{session_id}"),
                None,
                auth,
            )
            .await?;
        extract_single_session(&data)
    }

    pub async fn create_session(
        &self,
        base_url: &str,
        mode: ApiMode,
        options: &CreateSessionOptions,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        let body = build_create_body(options);
        let data = self
            .request(
                base_url,
                mode,
                reqwest::Method::POST,
                "/sessions",
                Some(body),
                auth,
            )
            .await?;
        extract_single_session(&data)
    }

    pub async fn release_session(
        &self,
        base_url: &str,
        mode: ApiMode,
        session_id: &str,
        auth: &Auth,
    ) -> Result<(), ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("/sessions/{session_id}/release"),
            None,
            auth,
        )
        .await?;
        Ok(())
    }

    pub async fn solve_captcha(
        &self,
        base_url: &str,
        mode: ApiMode,
        session_id: &str,
        page_id: Option<&str>,
        url: Option<&str>,
        task_id: Option<&str>,
        auth: &Auth,
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
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("/sessions/{session_id}/captchas/solve"),
            Some(payload),
            auth,
        )
        .await
    }

    pub async fn captcha_status(
        &self,
        base_url: &str,
        mode: ApiMode,
        session_id: &str,
        page_id: Option<&str>,
        auth: &Auth,
    ) -> Result<Vec<Value>, ApiError> {
        let path = match page_id {
            Some(pid) if !pid.trim().is_empty() => {
                let encoded = urlencoding::encode(pid.trim());
                format!("/sessions/{session_id}/captchas/status?pageId={encoded}")
            }
            _ => format!("/sessions/{session_id}/captchas/status"),
        };

        let data = self
            .request(base_url, mode, reqwest::Method::GET, &path, None, auth)
            .await?;

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

    // --- extract_session_list ---

    #[test]
    fn extract_list_from_array() {
        let data = json!([{"id": "s1"}, {"id": "s2"}]);
        let list = extract_session_list(&data);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn extract_list_from_nested() {
        let data = json!({"sessions": [{"id": "s1"}]});
        let list = extract_session_list(&data);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn extract_list_single_object() {
        let data = json!({"id": "s1", "status": "live"});
        let list = extract_session_list(&data);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn extract_list_null() {
        let list = extract_session_list(&Value::Null);
        assert!(list.is_empty());
    }

    // --- extract_single_session ---

    #[test]
    fn extract_single_direct() {
        let data = json!({"id": "s1"});
        let session = extract_single_session(&data).unwrap();
        assert_eq!(session["id"], "s1");
    }

    #[test]
    fn extract_single_nested() {
        let data = json!({"session": {"id": "s1"}});
        let session = extract_single_session(&data).unwrap();
        assert_eq!(session["id"], "s1");
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
            headless: Some(true),
            region: Some("us-east-1".into()),
            solve_captcha: true,
            namespace: Some("ns".into()),
            credentials: true,
            ..Default::default()
        });
        assert_eq!(body["proxyUrl"], "http://proxy:8080");
        assert_eq!(body["timeout"], 30000);
        assert_eq!(body["headless"], true);
        assert_eq!(body["region"], "us-east-1");
        assert_eq!(body["solveCaptcha"], true);
        assert_eq!(body["namespace"], "ns");
        assert!(body["credentials"].is_object());
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
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_auth() -> Auth {
        Auth {
            api_key: Some("test-key".into()),
            source: AuthSource::Env,
        }
    }

    #[tokio::test]
    async fn list_sessions_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/sessions"))
            .and(header("Steel-Api-Key", "test-key"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!([{"id": "s1", "status": "live"}])),
            )
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let sessions = client
            .list_sessions(&server.uri(), ApiMode::Local, &test_auth())
            .await
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["id"], "s1");
    }

    #[tokio::test]
    async fn get_session_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/sessions/s1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"id": "s1", "status": "live"})),
            )
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let session = client
            .get_session(&server.uri(), ApiMode::Local, "s1", &test_auth())
            .await
            .unwrap();

        assert_eq!(session["id"], "s1");
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

        let client = SteelClient::new().unwrap();
        let session = client
            .create_session(
                &server.uri(),
                ApiMode::Local,
                &CreateSessionOptions::default(),
                &test_auth(),
            )
            .await
            .unwrap();

        assert_eq!(session["id"], "new-sess");
    }

    #[tokio::test]
    async fn release_session_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/sessions/s1/release"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        client
            .release_session(&server.uri(), ApiMode::Local, "s1", &test_auth())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn api_error_extracts_message() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/sessions"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(json!({"message": "Invalid API key"})),
            )
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let err = client
            .list_sessions(&server.uri(), ApiMode::Local, &test_auth())
            .await
            .unwrap_err();

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

        let client = SteelClient::new().unwrap();
        let err = client
            .get_session(&server.uri(), ApiMode::Local, "gone", &test_auth())
            .await
            .unwrap_err();

        assert!(err.is_not_found());
    }
}
