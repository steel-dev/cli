use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

pub fn computer_path(id: &str) -> String {
    format!("/computers/{}", urlencoding::encode(id))
}

#[derive(Debug, Default, Clone)]
pub struct CreateComputer {
    pub template: Option<String>,
    pub vcpu: Option<u32>,
    pub memory_mib: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub auto_pause: Option<bool>,
    pub idle_timeout_seconds: Option<u32>,
    pub env: BTreeMap<String, String>,
}

impl CreateComputer {
    pub fn body(&self) -> Value {
        let mut body = json!({});
        if let Some(template) = &self.template {
            body["template"] = json!(template);
        }
        if let Some(vcpu) = self.vcpu {
            body["vcpu"] = json!(vcpu);
        }
        if let Some(memory) = self.memory_mib {
            body["memoryMib"] = json!(memory);
        }
        if let Some(timeout) = self.timeout_seconds {
            body["timeoutSeconds"] = json!(timeout);
        }
        if let Some(auto_pause) = self.auto_pause {
            body["autoPause"] = json!(auto_pause);
        }
        if let Some(idle_timeout) = self.idle_timeout_seconds {
            body["idleTimeoutSeconds"] = json!(idle_timeout);
        }
        if !self.env.is_empty() {
            body["env"] = json!(self.env);
        }
        body
    }
}

impl SteelClient {
    pub async fn list_computers(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            "/computers",
            None,
            auth,
        )
        .await
    }

    pub async fn get_computer_quota(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            "/computers/quota",
            None,
            auth,
        )
        .await
    }

    pub async fn get_computer(
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
            &computer_path(id),
            None,
            auth,
        )
        .await
    }

    pub async fn create_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        request: &CreateComputer,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/computers",
            Some(request.body()),
            auth,
        )
        .await
    }

    pub async fn delete_computer(
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
            &computer_path(id),
            None,
            auth,
        )
        .await
    }

    pub async fn pause_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/pause", computer_path(id)),
            None,
            auth,
        )
        .await
    }

    pub async fn resume_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/resume", computer_path(id)),
            None,
            auth,
        )
        .await
    }

    pub async fn stop_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/stop", computer_path(id)),
            None,
            auth,
        )
        .await
    }

    pub async fn start_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/start", computer_path(id)),
            None,
            auth,
        )
        .await
    }

    pub async fn restart_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/restart", computer_path(id)),
            None,
            auth,
        )
        .await
    }

    pub async fn exec_computer(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
        body: Value,
    ) -> Result<reqwest::Response, ApiError> {
        self.request_raw(
            base_url,
            mode,
            reqwest::Method::POST,
            &format!("{}/exec", computer_path(id)),
            Some(body),
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_body_only_carries_given_fields() {
        assert_eq!(CreateComputer::default().body(), json!({}));

        let named = CreateComputer {
            template: Some("steel".into()),
            ..Default::default()
        };
        assert_eq!(named.body(), json!({ "template": "steel" }));

        let full = CreateComputer {
            template: Some("steel".into()),
            vcpu: Some(4),
            memory_mib: Some(4096),
            timeout_seconds: Some(600),
            auto_pause: Some(true),
            idle_timeout_seconds: Some(90),
            env: BTreeMap::from([("TOKEN".into(), "abc".into())]),
        };
        assert_eq!(
            full.body(),
            json!({
                "template": "steel",
                "vcpu": 4,
                "memoryMib": 4096,
                "timeoutSeconds": 600,
                "autoPause": true,
                "idleTimeoutSeconds": 90,
                "env": { "TOKEN": "abc" },
            })
        );
    }

    #[test]
    fn zero_idle_timeout_is_sent_and_empty_env_is_not() {
        let request = CreateComputer {
            idle_timeout_seconds: Some(0),
            ..Default::default()
        };
        assert_eq!(request.body(), json!({ "idleTimeoutSeconds": 0 }));
    }

    #[test]
    fn computer_paths_escape_ids() {
        assert_eq!(
            computer_path("cmp_0000123456789abcdefghjkmnpqrs"),
            "/computers/cmp_0000123456789abcdefghjkmnpqrs"
        );
        assert_eq!(computer_path("a/b"), "/computers/a%2Fb");
    }
}
