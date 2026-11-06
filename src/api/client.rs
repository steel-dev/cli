use std::borrow::Cow;
use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApiError {
    #[error("Missing Steel API key. Run `steel login` or set `STEEL_API_KEY`.")]
    MissingAuth,

    #[error(transparent)]
    DeviceAuth(anyhow::Error),

    #[error("Failed to reach Steel API at {url}.")]
    Unreachable {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error(
        "Steel API at {url} answered with a redirect ({status}) to {location}. Credentials are never sent to a redirect target; set STEEL_API_URL or --api-url to the final API address."
    )]
    Redirected {
        url: String,
        status: u16,
        location: String,
    },

    #[error("Steel API request failed ({status}): {message}")]
    RequestFailed {
        status: u16,
        message: Cow<'static, str>,
        body: Option<Value>,
    },

    #[error(transparent)]
    Other(#[from] reqwest::Error),
}

impl ApiError {
    /// Check if this is a 404/410 (not found). Matches TS `isNotFoundApiError()`.
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::RequestFailed { status, .. } if *status == 404 || *status == 410)
    }

    /// Classify a failed send, keeping refused redirects distinct from network failures
    /// so callers do not retry them.
    pub fn transport(url: String, source: reqwest::Error) -> Self {
        let refused = std::error::Error::source(&source)
            .and_then(|e| e.downcast_ref::<RedirectRefused>())
            .map(|r| (r.status, r.location.clone()));
        match refused {
            Some((status, location)) => Self::Redirected {
                url,
                status,
                location,
            },
            None => Self::Unreachable { url, source },
        }
    }

    /// A 3xx the client returned unfollowed: reqwest skips the redirect policy when the
    /// body cannot be replayed (multipart streams) or the location is missing.
    pub fn redirect_response(url: &str, resp: &reqwest::Response) -> Option<Self> {
        if !resp.status().is_redirection() {
            return None;
        }
        let location = resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| resp.url().join(v).ok())
            .map_or_else(|| "an unspecified location".to_string(), display_location);
        Some(Self::Redirected {
            url: url.to_string(),
            status: resp.status().as_u16(),
            location,
        })
    }
}

/// Redirect targets are shown without credentials, query or fragment.
fn display_location(mut location: url::Url) -> String {
    let _ = location.set_username("");
    let _ = location.set_password(None);
    location.set_query(None);
    location.set_fragment(None);
    location.to_string()
}

/// Raised by the product client's redirect policy before any request reaches the target.
#[derive(Debug)]
struct RedirectRefused {
    status: u16,
    location: String,
}

impl std::fmt::Display for RedirectRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "refused {} redirect to {}", self.status, self.location)
    }
}

impl std::error::Error for RedirectRefused {}

/// Authenticated requests carry `Authorization` or `Steel-Api-Key`; reqwest strips only
/// the former, and only across origins, so redirects are refused outright.
fn refuse_redirects() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        let refused = RedirectRefused {
            status: attempt.status().as_u16(),
            location: display_location(attempt.url().clone()),
        };
        attempt.error(refused)
    })
}

/// Extract a human-readable error message from a JSON response body.
/// Matches TS `extractApiErrorMessage()`.
fn extract_error_message(body: &Value, status_text: &str) -> Cow<'static, str> {
    // Try body.message
    if let Some(msg) = body.get("message").and_then(|v| v.as_str())
        && !msg.trim().is_empty()
    {
        return Cow::Owned(msg.to_string());
    }

    // Try body.error.message
    if let Some(msg) = body
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|v| v.as_str())
        && !msg.trim().is_empty()
    {
        return Cow::Owned(msg.to_string());
    }

    if let Some(msg) = body.get("error").and_then(|v| v.as_str())
        && !msg.trim().is_empty()
    {
        return Cow::Owned(msg.to_string());
    }

    if !status_text.is_empty() {
        return Cow::Owned(status_text.to_string());
    }

    Cow::Borrowed("Unknown API error")
}

pub struct SteelClient {
    http: Client,
    base_url: String,
    mode: ApiMode,
    auth: Auth,
}

impl SteelClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let (mode, base_url, auth) = crate::util::api::resolve_with_auth();
        Self::with(base_url, mode, auth)
    }

    pub fn with(base_url: String, mode: ApiMode, auth: Auth) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .user_agent("steel-cli")
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .redirect(refuse_redirects())
            .build()?;
        Ok(Self {
            http,
            base_url,
            mode,
            auth,
        })
    }

    pub fn at(&self, base_url: &str, mode: ApiMode) -> Self {
        Self {
            http: self.http.clone(),
            base_url: base_url.to_string(),
            mode,
            auth: self.auth.clone(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub const fn mode(&self) -> ApiMode {
        self.mode
    }

    pub const fn auth(&self) -> &Auth {
        &self.auth
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    pub async fn authed(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<reqwest::RequestBuilder, ApiError> {
        if self.mode == ApiMode::Cloud && !self.auth.has_credentials() {
            return Err(ApiError::MissingAuth);
        }
        let req = self.http.request(method, self.url(path));
        if self.auth.access_token.is_some() {
            // Refreshes when due and refuses a transport the token was not issued for.
            let token = crate::config::device::access_token_for(&self.base_url)
                .await
                .map_err(ApiError::DeviceAuth)?;
            return Ok(req.bearer_auth(token));
        }
        Ok(match &self.auth.api_key {
            Some(key) => req.header("Steel-Api-Key", key),
            None => req,
        })
    }

    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ApiError> {
        self.request_within(method, path, body, None).await
    }

    pub async fn request_within(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
        timeout: Option<Duration>,
    ) -> Result<Value, ApiError> {
        let mut req = self
            .authed(method, path)
            .await?
            .header("Content-Type", "application/json");
        if let Some(body) = body {
            req = req.json(&body);
        }
        if let Some(timeout) = timeout {
            req = req.timeout(timeout);
        }
        self.send(req, path).await
    }

    pub async fn send(&self, req: reqwest::RequestBuilder, path: &str) -> Result<Value, ApiError> {
        let resp = req
            .send()
            .await
            .map_err(|source| ApiError::transport(self.url(path), source))?;
        if let Some(err) = ApiError::redirect_response(&self.url(path), &resp) {
            return Err(err);
        }

        let status = resp.status();
        let status_text = status.canonical_reason().unwrap_or("").to_string();
        let response_text = resp.text().await.map_err(ApiError::Other)?;
        let response_data: Value = if response_text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&response_text).unwrap_or(Value::String(response_text))
        };

        if !status.is_success() {
            let message = extract_error_message(&response_data, &status_text);
            return Err(ApiError::RequestFailed {
                status: status.as_u16(),
                message,
                body: Some(response_data),
            });
        }

        Ok(response_data)
    }
}

/// Two listeners on the same host but different ports: `origin` answers every request
/// with a redirect to `target`, which must never be contacted.
#[cfg(test)]
pub mod redirect_fixture {
    use wiremock::{Mock, MockServer, ResponseTemplate};

    pub struct Pair {
        pub origin: MockServer,
        pub target: MockServer,
    }

    pub async fn pair(status: u16) -> Pair {
        let origin = MockServer::start().await;
        let target = MockServer::start().await;
        Mock::given(wiremock::matchers::any())
            .respond_with(ResponseTemplate::new(status).insert_header(
                "location",
                format!("{}/captured?token=leak", target.uri()).as_str(),
            ))
            .mount(&origin)
            .await;
        Pair { origin, target }
    }

    impl Pair {
        pub async fn assert_target_untouched(&self) {
            assert_eq!(self.origin.received_requests().await.unwrap().len(), 1);
            assert!(self.target.received_requests().await.unwrap().is_empty());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn redirects_are_refused_before_the_target_sees_credentials() {
        for status in [301, 302, 303, 307, 308] {
            let pair = redirect_fixture::pair(status).await;
            let auth = Auth {
                api_key: Some("sk-test".into()),
                access_token: None,
                source: crate::config::auth::AuthSource::Env,
            };
            let client = SteelClient::with(pair.origin.uri(), ApiMode::Local, auth).unwrap();

            let err = client
                .request(reqwest::Method::POST, "/sessions", None)
                .await
                .unwrap_err();

            pair.assert_target_untouched().await;
            let ApiError::Redirected {
                status: got,
                ref location,
                ..
            } = err
            else {
                panic!("expected a refused redirect, got {err:?}");
            };
            assert_eq!(got, status);
            assert!(!location.contains("leak"), "{location}");
            assert!(err.to_string().contains("STEEL_API_URL"), "{err}");
        }
    }

    #[tokio::test]
    async fn redirects_without_a_location_are_still_reported_as_redirects() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(302))
            .mount(&server)
            .await;
        let auth = Auth {
            api_key: Some("sk-test".into()),
            access_token: None,
            source: crate::config::auth::AuthSource::Env,
        };
        let client = SteelClient::with(server.uri(), ApiMode::Local, auth).unwrap();

        let err = client
            .request(reqwest::Method::GET, "/sessions", None)
            .await
            .unwrap_err();

        assert!(
            matches!(err, ApiError::Redirected { status: 302, ref location, .. } if location == "an unspecified location"),
            "{err:?}"
        );
    }

    #[test]
    fn extract_message_from_body() {
        let body = serde_json::json!({"message": "Rate limit exceeded"});
        assert_eq!(extract_error_message(&body, ""), "Rate limit exceeded");
    }

    #[test]
    fn extract_message_from_nested_error() {
        let body = serde_json::json!({"error": {"message": "Invalid key"}});
        assert_eq!(extract_error_message(&body, ""), "Invalid key");
    }

    #[test]
    fn extract_message_falls_back_to_status_text() {
        let body = serde_json::json!({});
        assert_eq!(extract_error_message(&body, "Not Found"), "Not Found");
    }

    #[test]
    fn extract_message_unknown_fallback() {
        let body = serde_json::json!({});
        assert_eq!(extract_error_message(&body, ""), "Unknown API error");
    }

    #[test]
    fn is_not_found_404() {
        let err = ApiError::RequestFailed {
            status: 404,
            message: "Not found".into(),
            body: None,
        };
        assert!(err.is_not_found());
    }

    #[test]
    fn is_not_found_410() {
        let err = ApiError::RequestFailed {
            status: 410,
            message: "Gone".into(),
            body: None,
        };
        assert!(err.is_not_found());
    }

    #[test]
    fn is_not_found_500_false() {
        let err = ApiError::RequestFailed {
            status: 500,
            message: "Server error".into(),
            body: None,
        };
        assert!(!err.is_not_found());
    }

    #[tokio::test]
    async fn missing_auth_in_cloud_mode() {
        let auth = Auth {
            api_key: None,
            access_token: None,
            source: crate::config::auth::AuthSource::None,
        };
        let client = SteelClient::with("http://localhost".into(), ApiMode::Cloud, auth).unwrap();

        let result = client.request(reqwest::Method::GET, "/test", None).await;

        assert!(matches!(result, Err(ApiError::MissingAuth)));
    }

    #[tokio::test]
    async fn requests_time_out_instead_of_hanging() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(30)).await;
        });
        let auth = Auth {
            api_key: None,
            access_token: None,
            source: crate::config::auth::AuthSource::None,
        };
        let client = SteelClient::with(format!("http://{addr}"), ApiMode::Local, auth).unwrap();

        let result = client
            .request_within(
                reqwest::Method::GET,
                "/test",
                None,
                Some(Duration::from_millis(200)),
            )
            .await;

        assert!(
            matches!(result, Err(ApiError::Unreachable { ref source, .. }) if source.is_timeout())
        );
    }

    #[tokio::test]
    async fn local_mode_allows_no_auth() {
        // This will fail to connect, but should NOT fail with MissingAuth
        let auth = Auth {
            api_key: None,
            access_token: None,
            source: crate::config::auth::AuthSource::None,
        };
        let client = SteelClient::with("http://127.0.0.1:1".into(), ApiMode::Local, auth).unwrap();

        let result = client.request(reqwest::Method::GET, "/test", None).await;

        assert!(matches!(result, Err(ApiError::Unreachable { .. })));
    }
}
