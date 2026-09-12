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
    pub secrets: BTreeMap<String, String>,
    pub environment_id: Option<String>,
    pub project_id: Option<String>,
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
        if !self.secrets.is_empty() {
            body["secrets"] = json!(self.secrets);
        }
        if let Some(environment) = &self.environment_id {
            body["environmentId"] = json!(environment);
        }
        if let Some(project) = &self.project_id {
            body["projectId"] = json!(project);
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
            secrets: BTreeMap::from([(
                "API_KEY".into(),
                "2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b31".into(),
            )]),
            environment_id: Some("9b2d1c5e-1b2a-4c3d-8e4f-5a6b7c8d9e0f".into()),
            project_id: Some("0c1d2e3f-4a5b-4c6d-8e7f-8091a2b3c4d5".into()),
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
                "secrets": { "API_KEY": "2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b31" },
                "environmentId": "9b2d1c5e-1b2a-4c3d-8e4f-5a6b7c8d9e0f",
                "projectId": "0c1d2e3f-4a5b-4c6d-8e7f-8091a2b3c4d5",
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
