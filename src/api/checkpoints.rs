use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::api::computers::computer_path;
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

pub fn checkpoint_path(id: &str) -> String {
    format!("/checkpoints/{}", urlencoding::encode(id))
}

#[derive(Debug, Default, Clone)]
pub struct RestoreCheckpoint {
    pub timeout_seconds: Option<u32>,
    pub auto_pause: Option<bool>,
}

impl RestoreCheckpoint {
    pub fn body(&self) -> Value {
        let mut body = json!({});
        if let Some(timeout) = self.timeout_seconds {
            body["timeoutSeconds"] = json!(timeout);
        }
        if let Some(auto_pause) = self.auto_pause {
            body["autoPause"] = json!(auto_pause);
        }
        body
    }
}

impl SteelClient {
    pub async fn create_checkpoint(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        computer_id: &str,
        name: Option<&str>,
    ) -> Result<Value, ApiError> {
        let mut body = json!({});
        if let Some(name) = name {
            body["name"] = json!(name);
        }
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/checkpoints", computer_path(computer_id)),
            Some(body),
            auth,
        )
        .await
    }

    pub async fn list_checkpoints(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            "/checkpoints",
            None,
            auth,
        )
        .await
    }

    pub async fn get_checkpoint(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &checkpoint_path(id),
            None,
            auth,
        )
        .await
    }

    pub async fn delete_checkpoint(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::DELETE,
            &checkpoint_path(id),
            None,
            auth,
        )
        .await
    }

    pub async fn restore_checkpoint(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
        request: &RestoreCheckpoint,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/computers", checkpoint_path(id)),
            Some(request.body()),
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_encoded() {
        assert_eq!(checkpoint_path("ckpt a"), "/checkpoints/ckpt%20a");
    }

    #[test]
    fn restore_body_only_carries_given_options() {
        assert_eq!(RestoreCheckpoint::default().body(), json!({}));
        let request = RestoreCheckpoint {
            timeout_seconds: Some(120),
            auto_pause: Some(true),
        };
        assert_eq!(
            request.body(),
            json!({ "timeoutSeconds": 120, "autoPause": true })
        );
    }
}
