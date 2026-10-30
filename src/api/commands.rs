use std::time::Duration;

use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(40);
const FILE_TIMEOUT: Duration = Duration::from_secs(300);
const DELIVERY_RETRIES: u32 = 2;
const BUSY_RETRIES: u32 = 3;
const MAX_RETRY_AFTER_SECS: u64 = 10;
const STATUS_POLLS: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub status: u16,
    pub retry_after: Option<u64>,
    pub body: Value,
}

impl Reply {
    pub fn error_code(&self) -> Option<&str> {
        self.body
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str)
    }

    pub fn error_message(&self) -> Option<&str> {
        self.body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
    }

    fn is_pending(&self) -> bool {
        self.status == 200
            && matches!(
                self.body["execution"]["state"].as_str(),
                Some("queued" | "running")
            )
    }
}

pub struct Session<'a> {
    pub client: &'a SteelClient,
    pub base_url: &'a str,
    pub mode: ApiMode,
    pub auth: &'a Auth,
    pub id: &'a str,
}

pub fn new_key() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("failed to generate an idempotency key");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

const fn backoff(attempt: u32) -> Duration {
    Duration::from_millis(250 * 4u64.pow(attempt.saturating_sub(1)))
}

impl Session<'_> {
    fn url(&self, path: &str) -> String {
        format!("{}/sessions/{}{path}", self.base_url, self.id)
    }

    async fn post(&self, body: &Value, key: Option<&str>) -> Result<Reply, ApiError> {
        let url = self.url("/commands");
        let mut req = self
            .client
            .authed(reqwest::Method::POST, &url, self.mode, self.auth)?
            .timeout(REQUEST_TIMEOUT)
            .json(body);
        if let Some(key) = key {
            req = req.header("Idempotency-Key", key);
        }
        let unreachable = |source| ApiError::Unreachable {
            url: url.clone(),
            source,
        };
        let resp = req.send().await.map_err(unreachable)?;
        let status = resp.status().as_u16();
        let retry_after = resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok());
        let text = resp.text().await.map_err(unreachable)?;
        let body = if text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };
        Ok(Reply {
            status,
            retry_after,
            body,
        })
    }

    pub async fn run(&self, mut body: Value) -> Result<Reply, ApiError> {
        let mut key = new_key();
        let mut maybe_ran = false;
        let mut delivery = 0;
        let mut busy = 0;
        let mut renewed = false;
        loop {
            let reply = match self.post(&body, Some(&key)).await {
                Ok(reply) => reply,
                Err(err @ ApiError::Unreachable { .. }) => {
                    maybe_ran = true;
                    if delivery >= DELIVERY_RETRIES {
                        return Err(err);
                    }
                    delivery += 1;
                    tokio::time::sleep(backoff(delivery)).await;
                    continue;
                }
                Err(err) => return Err(err),
            };
            match (reply.status, reply.error_code()) {
                (503, _) if delivery < DELIVERY_RETRIES => {
                    maybe_ran = true;
                    delivery += 1;
                    tokio::time::sleep(backoff(delivery)).await;
                }
                (429, _) if busy < BUSY_RETRIES => {
                    busy += 1;
                    let wait = reply.retry_after.unwrap_or(1).min(MAX_RETRY_AFTER_SECS);
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                }
                (422, Some("invalid_operation_token")) if !maybe_ran && !renewed => {
                    let Some(next) = reply.body.get("next").cloned() else {
                        return Ok(reply);
                    };
                    body["after"] = next;
                    key = new_key();
                    renewed = true;
                }
                (200, _) => return self.settle(reply, body.get("eventsAfter")).await,
                _ => return Ok(reply),
            }
        }
    }

    pub async fn control(&self, command: Value, events: Option<&Value>) -> Result<Reply, ApiError> {
        let mut body = json!({ "commands": [command] });
        if let Some(cursor) = events {
            body["eventsAfter"] = cursor.clone();
        }
        let mut delivery = 0;
        loop {
            let result = self.post(&body, None).await;
            let retry = match &result {
                Err(ApiError::Unreachable { .. }) => true,
                Ok(reply) => reply.status == 503,
                Err(_) => false,
            };
            if !retry || delivery >= DELIVERY_RETRIES {
                return result;
            }
            delivery += 1;
            tokio::time::sleep(backoff(delivery)).await;
        }
    }

    async fn settle(&self, mut reply: Reply, events: Option<&Value>) -> Result<Reply, ApiError> {
        for _ in 0..STATUS_POLLS {
            if !reply.is_pending() {
                return Ok(reply);
            }
            let Some(execution) = reply.body["execution"]["id"].as_str() else {
                return Ok(reply);
            };
            let next = reply.body.get("next").cloned();
            let status = json!({ "command": "status", "execution": execution });
            let mut polled = self.control(status, events).await?;
            if polled.status != 200 {
                return Ok(polled);
            }
            if polled.body.get("next").is_none()
                && let Some(next) = next
            {
                polled.body["next"] = next;
            }
            reply = polled;
        }
        Ok(reply)
    }

    pub async fn upload_file(&self, bytes: Vec<u8>, name: &str) -> Result<String, ApiError> {
        let url = self.url("/files");
        let part = reqwest::multipart::Part::bytes(bytes).file_name(name.to_string());
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("path", name.to_string());
        let resp = self
            .client
            .authed(reqwest::Method::POST, &url, self.mode, self.auth)?
            .timeout(FILE_TIMEOUT)
            .multipart(form)
            .send()
            .await
            .map_err(|source| ApiError::Unreachable {
                url: url.clone(),
                source,
            })?;
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(failed(status, &body));
        }
        Ok(body
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or(name)
            .to_string())
    }

    pub async fn download_file(&self, name: &str) -> Result<Vec<u8>, ApiError> {
        let encoded: Vec<String> = name
            .split('/')
            .map(|segment| urlencoding::encode(segment).into_owned())
            .collect();
        let url = self.url(&format!("/files/{}", encoded.join("/")));
        let resp = self
            .client
            .authed(reqwest::Method::GET, &url, self.mode, self.auth)?
            .timeout(FILE_TIMEOUT)
            .send()
            .await
            .map_err(|source| ApiError::Unreachable {
                url: url.clone(),
                source,
            })?;
        let status = resp.status();
        if !status.is_success() {
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            return Err(failed(status, &body));
        }
        Ok(resp.bytes().await.map_err(ApiError::Other)?.to_vec())
    }
}

fn failed(status: reqwest::StatusCode, body: &Value) -> ApiError {
    let message = body
        .get("message")
        .or_else(|| body.get("error").and_then(|e| e.get("message")))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            status
                .canonical_reason()
                .unwrap_or("request failed")
                .to_string()
        });
    ApiError::RequestFailed {
        status: status.as_u16(),
        message: message.into(),
        body: Some(body.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::auth::AuthSource;
    use wiremock::matchers::{body_partial_json, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    const ID: &str = "11111111-2222-3333-4444-555555555555";

    fn auth() -> Auth {
        Auth {
            api_key: Some("sk-test".into()),
            source: AuthSource::Env,
        }
    }

    fn completed(next: &str) -> Value {
        json!({
            "next": next,
            "epoch": 1,
            "page": {"tab": "t1", "url": "https://example.com/", "title": "Example"},
            "execution": {"id": "x-1", "state": "completed", "effect": "applied", "replayed": false, "ran": 1},
            "results": [{"ok": true, "effect": "applied"}]
        })
    }

    async fn run(server: &MockServer, body: Value) -> Result<Reply, ApiError> {
        let client = SteelClient::new().unwrap();
        let auth = auth();
        let base = server.uri();
        let session = Session {
            client: &client,
            base_url: &base,
            mode: ApiMode::Cloud,
            auth: &auth,
            id: ID,
        };
        session.run(body).await
    }

    fn commands_path() -> String {
        format!("/sessions/{ID}/commands")
    }

    fn keys(requests: &[Request]) -> Vec<String> {
        requests
            .iter()
            .map(|r| {
                r.headers
                    .get("idempotency-key")
                    .map(|v| v.to_str().unwrap().to_string())
                    .unwrap_or_default()
            })
            .collect()
    }

    #[tokio::test]
    async fn sends_api_key_idempotency_key_and_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .and(header("steel-api-key", "sk-test"))
            .and(header_exists("idempotency-key"))
            .and(body_partial_json(
                json!({"after": "op_1", "commands": [{"command": "click", "target": "e1"}]}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(completed("op_2")))
            .expect(1)
            .mount(&server)
            .await;
        let reply = run(
            &server,
            json!({"after": "op_1", "commands": [{"command": "click", "target": "e1"}]}),
        )
        .await
        .unwrap();
        assert_eq!(reply.status, 200);
        assert_eq!(reply.body["next"], "op_2");
    }

    #[tokio::test]
    async fn retries_delivery_failures_with_the_same_key() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(503).set_body_json(
                json!({"error": {"code": "delivery_failed", "message": "unreachable"}}),
            ))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(completed("op_2")))
            .mount(&server)
            .await;
        let reply = run(&server, json!({"commands": [{"command": "reload"}]}))
            .await
            .unwrap();
        assert_eq!(reply.status, 200);
        let requests = server.received_requests().await.unwrap();
        let keys = keys(&requests);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], keys[1]);
        assert!(!keys[0].is_empty());
    }

    #[tokio::test]
    async fn gives_up_after_repeated_delivery_failures() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(503).set_body_json(
                json!({"error": {"code": "delivery_failed", "message": "unreachable"}}),
            ))
            .mount(&server)
            .await;
        let reply = run(&server, json!({"commands": [{"command": "reload"}]}))
            .await
            .unwrap();
        assert_eq!(reply.status, 503);
        assert_eq!(reply.error_code(), Some("delivery_failed"));
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn waits_and_retries_when_busy() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("retry-after", "0")
                    .set_body_json(json!({"error": {"code": "queue_full", "message": "busy"}})),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(completed("op_2")))
            .mount(&server)
            .await;
        let reply = run(&server, json!({"commands": [{"command": "reload"}]}))
            .await
            .unwrap();
        assert_eq!(reply.status, 200);
        let keys = keys(&server.received_requests().await.unwrap());
        assert_eq!(keys[0], keys[1]);
    }

    #[tokio::test]
    async fn renews_an_invalid_after_on_the_first_try() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .and(body_partial_json(json!({"after": "op_old"})))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "next": "op_new",
                "error": {"code": "invalid_operation_token", "message": "expired"}
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .and(body_partial_json(json!({"after": "op_new"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(completed("op_3")))
            .mount(&server)
            .await;
        let reply = run(
            &server,
            json!({"after": "op_old", "commands": [{"command": "reload"}]}),
        )
        .await
        .unwrap();
        assert_eq!(reply.status, 200);
        assert_eq!(reply.body["next"], "op_3");
    }

    #[tokio::test]
    async fn does_not_renew_after_an_uncertain_attempt() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(503).set_body_json(
                json!({"error": {"code": "delivery_failed", "message": "unreachable"}}),
            ))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "next": "op_new",
                "error": {"code": "invalid_operation_token", "message": "expired"}
            })))
            .mount(&server)
            .await;
        let reply = run(
            &server,
            json!({"after": "op_old", "commands": [{"command": "reload"}]}),
        )
        .await
        .unwrap();
        assert_eq!(reply.status, 422);
        assert_eq!(reply.error_code(), Some("invalid_operation_token"));
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn waits_for_a_running_execution_with_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .and(body_partial_json(json!({"commands": [{"command": "status", "execution": "x-7"}]})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "epoch": 2,
                "page": null,
                "execution": {"id": "x-7", "state": "completed", "effect": "none", "replayed": false, "ran": 1},
                "results": [{"ok": true, "effect": "none", "waited": "text"}],
                "events": {"cursor": "c-9", "items": []}
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "next": "op_9",
                "epoch": 2,
                "page": null,
                "execution": {"id": "x-7", "state": "running", "effect": "none", "replayed": false, "ran": 1},
                "results": []
            })))
            .mount(&server)
            .await;
        let reply = run(
            &server,
            json!({"commands": [{"command": "wait", "text": "Done"}], "eventsAfter": "c-3"}),
        )
        .await
        .unwrap();
        assert_eq!(reply.body["execution"]["state"], "completed");
        assert_eq!(reply.body["next"], "op_9");
        let requests = server.received_requests().await.unwrap();
        let status: Value = serde_json::from_slice(&requests[1].body).unwrap();
        assert_eq!(status["eventsAfter"], "c-3");
        assert!(requests[1].headers.get("idempotency-key").is_none());
    }

    #[tokio::test]
    async fn passes_session_errors_through() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(commands_path()))
            .respond_with(
                ResponseTemplate::new(410)
                    .set_body_json(json!({"error": {"code": "session_ended", "message": "ended"}})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let reply = run(&server, json!({"commands": [{"command": "reload"}]}))
            .await
            .unwrap();
        assert_eq!(reply.status, 410);
        assert_eq!(reply.error_code(), Some("session_ended"));
    }

    #[tokio::test]
    async fn missing_api_key_fails_before_sending() {
        let client = SteelClient::new().unwrap();
        let auth = Auth {
            api_key: None,
            source: AuthSource::None,
        };
        let session = Session {
            client: &client,
            base_url: "http://127.0.0.1:1",
            mode: ApiMode::Cloud,
            auth: &auth,
            id: ID,
        };
        let result = session.run(json!({"commands": []})).await;
        assert!(matches!(result, Err(ApiError::MissingAuth)));
    }

    #[test]
    fn keys_are_random_hex() {
        let a = new_key();
        let b = new_key();
        assert_eq!(a.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
}
