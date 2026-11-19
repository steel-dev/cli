use serde_json::Value;

use crate::api::client::{ApiError, SteelClient};

impl SteelClient {
    pub async fn send_feedback(&self, body: &Value) -> Result<Value, ApiError> {
        self.request(reqwest::Method::POST, "/feedback/agent", Some(body.clone()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::config::auth::{Auth, AuthSource};
    use crate::config::settings::ApiMode;

    fn client(server: &MockServer, api_key: Option<&str>) -> SteelClient {
        let auth = Auth {
            api_key: api_key.map(Into::into),
            access_token: None,
            source: AuthSource::Env,
        };
        SteelClient::with(server.uri(), ApiMode::Cloud, auth).unwrap()
    }

    #[tokio::test]
    async fn sends_feedback_with_the_api_key() {
        let server = MockServer::start().await;
        let body = json!({"category": "bug", "message": "browser start fails"});

        Mock::given(method("POST"))
            .and(path("/feedback/agent"))
            .and(header("Steel-Api-Key", "test-key"))
            .and(body_json(&body))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "f-1"})))
            .expect(1)
            .mount(&server)
            .await;

        let data = client(&server, Some("test-key"))
            .send_feedback(&body)
            .await
            .unwrap();

        assert_eq!(data["id"], "f-1");
    }

    #[tokio::test]
    async fn refuses_to_send_without_signing_in() {
        let server = MockServer::start().await;

        let err = client(&server, None)
            .send_feedback(&json!({"category": "bug", "message": "x"}))
            .await
            .unwrap_err();

        assert!(matches!(err, ApiError::MissingAuth));
        assert!(server.received_requests().await.unwrap().is_empty());
    }
}
