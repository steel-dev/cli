use std::time::Duration;

use reqwest::multipart::{Form, Part};
use serde_json::Value;

use crate::api::client::{ApiError, SteelClient};

const UPLOAD_TIMEOUT: Duration = Duration::from_secs(600);

fn profile_form(zip: Vec<u8>) -> Result<Form, ApiError> {
    let part = Part::bytes(zip)
        .file_name("userDataDir.zip")
        .mime_str("application/zip")?;
    Ok(Form::new().part("userDataDir", part))
}

impl SteelClient {
    pub async fn create_profile(&self, zip: Vec<u8>) -> anyhow::Result<String> {
        let req = self
            .authed(reqwest::Method::POST, "/profiles")?
            .timeout(UPLOAD_TIMEOUT)
            .multipart(profile_form(zip)?);
        let body = self.send(req, "/profiles").await?;
        body.get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow::anyhow!("Profile upload response missing id"))
    }

    pub async fn update_profile(&self, profile_id: &str, zip: Vec<u8>) -> Result<(), ApiError> {
        let path = format!("/profiles/{}", urlencoding::encode(profile_id));
        let req = self
            .authed(reqwest::Method::PATCH, &path)?
            .timeout(UPLOAD_TIMEOUT)
            .multipart(profile_form(zip)?);
        self.send(req, &path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::api::client::{ApiError, SteelClient};
    use crate::config::auth::{Auth, AuthSource};
    use crate::config::settings::ApiMode;

    fn client(server: &MockServer) -> SteelClient {
        let auth = Auth {
            api_key: Some("sk-test".into()),
            source: AuthSource::Env,
        };
        SteelClient::with(server.uri(), ApiMode::Local, auth).unwrap()
    }

    #[tokio::test]
    async fn create_profile_returns_the_new_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/profiles"))
            .and(header("steel-api-key", "sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "prof_1"})))
            .mount(&server)
            .await;

        let id = client(&server)
            .create_profile(b"zip".to_vec())
            .await
            .unwrap();

        assert_eq!(id, "prof_1");
    }

    #[tokio::test]
    async fn upload_errors_keep_the_status_even_without_json() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/profiles/prof_1"))
            .respond_with(ResponseTemplate::new(413).set_body_string("<html>too large</html>"))
            .mount(&server)
            .await;

        let err = client(&server)
            .update_profile("prof_1", b"zip".to_vec())
            .await
            .unwrap_err();

        assert!(matches!(err, ApiError::RequestFailed { status: 413, .. }));
    }

    #[tokio::test]
    async fn cloud_uploads_need_an_api_key() {
        let auth = Auth {
            api_key: None,
            source: AuthSource::None,
        };
        let client = SteelClient::with("http://127.0.0.1:1".into(), ApiMode::Cloud, auth).unwrap();

        let err = client.create_profile(b"zip".to_vec()).await.unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ApiError>(),
            Some(ApiError::MissingAuth)
        ));
    }
}
