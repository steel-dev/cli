//! Device authorization (RFC 8628) + account-token-backed project management.
//!
//! The device + token endpoints are unauthenticated and use an OAuth-style error
//! envelope, so they are called with `reqwest` directly to inspect the `error` code.
//! Project / API-key / logout calls are authenticated with the account token and go
//! through [`SteelClient`].

use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::config::auth;
use crate::config::settings::ApiMode;

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceAuthorization {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrgPayload {
    pub id: String,
    #[serde(default)]
    pub slug: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceTokenSuccess {
    pub access_token: String,
    #[allow(dead_code)]
    pub token_name: String,
    pub org: OrgPayload,
    #[serde(default)]
    pub tos_required: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPayload {
    pub id: String,
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub is_default: bool,
}

/// Start a device authorization request. Returns the user code and verification URLs.
pub async fn request_device_authorization(base_url: &str) -> anyhow::Result<DeviceAuthorization> {
    let client = reqwest::Client::builder().user_agent("steel-cli").build()?;
    let response = client
        .post(format!("{base_url}/auth/device"))
        .header("Content-Type", "application/json")
        .json(&json!({ "client_id": "cli" }))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        anyhow::bail!(
            "Could not start device login ({}). Please try again shortly.",
            status.as_u16()
        );
    }

    Ok(response.json().await?)
}

/// Poll the token endpoint until the device is approved, denied, or expires.
///
/// Honors the RFC 8628 `interval` and `slow_down` backoff and stops after `expires_in`.
pub async fn poll_for_token(
    base_url: &str,
    device_code: &str,
    device_name: &str,
    interval: u64,
    expires_in: u64,
) -> anyhow::Result<DeviceTokenSuccess> {
    let client = reqwest::Client::builder().user_agent("steel-cli").build()?;
    let mut wait = interval.max(1);
    let deadline = Instant::now() + Duration::from_secs(expires_in.max(1));

    loop {
        if Instant::now() >= deadline {
            anyhow::bail!("Device login expired before it was approved. Run `steel login` again.");
        }

        tokio::time::sleep(Duration::from_secs(wait)).await;

        let response = client
            .post(format!("{base_url}/auth/device/token"))
            .header("Content-Type", "application/json")
            .json(&json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": device_code,
                "client_id": "cli",
                "device_name": device_name,
            }))
            .send()
            .await?;

        let status = response.status();
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);

        if status.is_success() {
            return Ok(serde_json::from_value(body)?);
        }

        match classify_token_error(&body) {
            TokenPoll::Pending => continue,
            TokenPoll::SlowDown => {
                wait += 5;
                continue;
            }
            TokenPoll::Denied => {
                anyhow::bail!("Device login was denied in the browser. Run `steel login` again.")
            }
            TokenPoll::Expired => {
                anyhow::bail!("The device code expired. Run `steel login` again.")
            }
            TokenPoll::Failed(message) => anyhow::bail!("Device login failed: {message}"),
        }
    }
}

enum TokenPoll {
    Pending,
    SlowDown,
    Denied,
    Expired,
    Failed(String),
}

fn classify_token_error(body: &serde_json::Value) -> TokenPoll {
    let error = body.get("error").and_then(|v| v.as_str()).unwrap_or("");
    match error {
        "authorization_pending" => TokenPoll::Pending,
        "slow_down" => TokenPoll::SlowDown,
        "access_denied" => TokenPoll::Denied,
        "expired_token" => TokenPoll::Expired,
        other => {
            let description = body
                .get("error_description")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(if other.is_empty() {
                    "unknown error"
                } else {
                    other
                });
            TokenPoll::Failed(description.to_string())
        }
    }
}

/// List projects for the organization tied to the account token.
pub async fn list_projects(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
) -> anyhow::Result<Vec<ProjectPayload>> {
    let client = SteelClient::new()?;
    let data = client
        .request(
            base_url,
            mode,
            reqwest::Method::GET,
            "/projects",
            None,
            &auth::account_token_auth(account_token),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let projects = data
        .get("projects")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(projects
        .into_iter()
        .filter_map(|p| serde_json::from_value(p).ok())
        .collect())
}

/// Create a new project. Requires an org admin account token (API returns 403 otherwise).
pub async fn create_project(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
    name: &str,
) -> anyhow::Result<ProjectPayload> {
    let client = SteelClient::new()?;
    let data = client
        .request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/projects",
            Some(json!({ "name": name })),
            &auth::account_token_auth(account_token),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    serde_json::from_value(data).map_err(|e| anyhow::anyhow!("Unexpected project response: {e}"))
}

/// Mint a project-scoped API key (used for browser/session work).
pub async fn create_project_api_key(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
    project_id: &str,
    name: &str,
) -> anyhow::Result<String> {
    let client = SteelClient::new()?;
    let data = client
        .request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/api-keys",
            Some(json!({ "name": name, "projectId": project_id })),
            &auth::account_token_auth(account_token),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    data.get("key")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("API key response did not include a key"))
}

/// Revoke the account token used for this request (server-side logout).
pub async fn revoke_account_token(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
) -> anyhow::Result<()> {
    let client = SteelClient::new()?;
    client
        .request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/auth/cli-tokens/logout",
            None,
            &auth::account_token_auth(account_token),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_error_is_classified() {
        let body = json!({ "error": "authorization_pending" });
        assert!(matches!(classify_token_error(&body), TokenPoll::Pending));
    }

    #[test]
    fn slow_down_error_is_classified() {
        let body = json!({ "error": "slow_down" });
        assert!(matches!(classify_token_error(&body), TokenPoll::SlowDown));
    }

    #[test]
    fn denied_error_is_classified() {
        let body = json!({ "error": "access_denied" });
        assert!(matches!(classify_token_error(&body), TokenPoll::Denied));
    }

    #[test]
    fn expired_error_is_classified() {
        let body = json!({ "error": "expired_token" });
        assert!(matches!(classify_token_error(&body), TokenPoll::Expired));
    }

    #[test]
    fn unknown_error_uses_description() {
        let body = json!({ "error": "server_error", "error_description": "boom" });
        match classify_token_error(&body) {
            TokenPoll::Failed(msg) => assert_eq!(msg, "boom"),
            _ => panic!("expected Failed"),
        }
    }

    #[test]
    fn device_token_success_deserializes() {
        let body = json!({
            "access_token": "ste-cli-abc",
            "token_name": "My Mac",
            "org": { "id": "org-1", "slug": "acme", "name": "Acme" }
        });
        let parsed: DeviceTokenSuccess = serde_json::from_value(body).unwrap();
        assert_eq!(parsed.access_token, "ste-cli-abc");
        assert_eq!(parsed.org.id, "org-1");
        assert_eq!(parsed.org.name, "Acme");
    }

    #[test]
    fn project_payload_deserializes_camel_case() {
        let body = json!({
            "id": "proj-1",
            "slug": "dev",
            "name": "Dev",
            "isDefault": true,
            "isProduction": false
        });
        let parsed: ProjectPayload = serde_json::from_value(body).unwrap();
        assert_eq!(parsed.id, "proj-1");
        assert!(parsed.is_default);
    }
}
