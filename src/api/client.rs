use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Missing Steel API key. Run `steel login` or set `STEEL_API_KEY`.")]
    MissingAuth,

    #[error("Failed to reach Steel API at {url}.")]
    Unreachable {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Steel API request failed ({status}): {message}")]
    RequestFailed {
        status: u16,
        message: String,
        body: Option<Value>,
    },

    #[error(transparent)]
    Other(#[from] reqwest::Error),
}

impl ApiError {
    /// Check if this is a 404/410 (not found). Matches TS `isNotFoundApiError()`.
    pub fn is_not_found(&self) -> bool {
        matches!(self, ApiError::RequestFailed { status, .. } if *status == 404 || *status == 410)
    }
}

/// Extract a human-readable error message from a JSON response body.
/// Matches TS `extractApiErrorMessage()`.
fn extract_error_message(body: &Value, status_text: &str) -> String {
    // Try body.message
    if let Some(msg) = body.get("message").and_then(|v| v.as_str()) {
        if !msg.trim().is_empty() {
            return msg.to_string();
        }
    }

    // Try body.error.message
    if let Some(msg) = body
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|v| v.as_str())
    {
        if !msg.trim().is_empty() {
            return msg.to_string();
        }
    }

    if !status_text.is_empty() {
        return status_text.to_string();
    }

    "Unknown API error".to_string()
}

pub struct SteelClient {
    http: Client,
}

impl SteelClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let http = Client::builder().user_agent("steel-cli").build()?;
        Ok(Self { http })
    }

    /// Make an authenticated API request.
    /// Matches the shared request pattern in TS `requestApi()` and `requestTopLevelApi()`.
    pub async fn request(
        &self,
        base_url: &str,
        mode: ApiMode,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        // Cloud mode requires auth
        if mode == ApiMode::Cloud && auth.api_key.is_none() {
            return Err(ApiError::MissingAuth);
        }

        let url = format!("{base_url}{path}");

        let mut req = self.http.request(method, &url);
        req = req.header("Content-Type", "application/json");

        if let Some(key) = &auth.api_key {
            req = req.header("Steel-Api-Key", key);
        }

        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req.send().await.map_err(|e| ApiError::Unreachable {
            url: url.clone(),
            source: e,
        })?;

        let status = resp.status();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("").to_string();

        // Read body as text, then try to parse as JSON
        let response_text = resp.text().await.map_err(ApiError::Other)?;

        let response_data: Value = if response_text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&response_text).unwrap_or(Value::String(response_text))
        };

        if !status.is_success() {
            let message = extract_error_message(&response_data, &status_text);
            return Err(ApiError::RequestFailed {
                status: status_code,
                message,
                body: Some(response_data),
            });
        }

        Ok(response_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let client = SteelClient::new().unwrap();
        let auth = Auth {
            api_key: None,
            source: crate::config::auth::AuthSource::None,
        };

        let result = client
            .request("http://localhost", ApiMode::Cloud, reqwest::Method::GET, "/test", None, &auth)
            .await;

        assert!(matches!(result, Err(ApiError::MissingAuth)));
    }

    #[tokio::test]
    async fn local_mode_allows_no_auth() {
        // This will fail to connect, but should NOT fail with MissingAuth
        let client = SteelClient::new().unwrap();
        let auth = Auth {
            api_key: None,
            source: crate::config::auth::AuthSource::None,
        };

        let result = client
            .request(
                "http://127.0.0.1:1",
                ApiMode::Local,
                reqwest::Method::GET,
                "/test",
                None,
                &auth,
            )
            .await;

        assert!(matches!(result, Err(ApiError::Unreachable { .. })));
    }
}
