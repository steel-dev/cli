use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

pub fn computer_path(id: &str) -> String {
    format!("/computers/{}", urlencoding::encode(id))
}

#[derive(Debug, Default, Clone)]
pub struct CreateComputer {
    pub template: String,
    pub region: Option<String>,
    pub vcpu: Option<u32>,
    pub memory_mib: Option<u32>,
    pub disk_mib: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub auto_pause: Option<bool>,
}

impl CreateComputer {
    pub fn body(&self) -> Value {
        let mut body = json!({ "template": self.template });
        if let Some(region) = &self.region {
            body["region"] = json!(region);
        }
        if let Some(vcpu) = self.vcpu {
            body["vcpu"] = json!(vcpu);
        }
        if let Some(memory) = self.memory_mib {
            body["memoryMib"] = json!(memory);
        }
        if let Some(disk) = self.disk_mib {
            body["diskMib"] = json!(disk);
        }
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
        let minimal = CreateComputer {
            template: "steel".into(),
            ..Default::default()
        };
        assert_eq!(minimal.body(), json!({ "template": "steel" }));

        let full = CreateComputer {
            template: "steel".into(),
            region: Some("us-east".into()),
            vcpu: Some(4),
            memory_mib: Some(4096),
            disk_mib: Some(10240),
            timeout_seconds: Some(600),
            auto_pause: Some(true),
        };
        assert_eq!(
            full.body(),
            json!({
                "template": "steel",
                "region": "us-east",
                "vcpu": 4,
                "memoryMib": 4096,
                "diskMib": 10240,
                "timeoutSeconds": 600,
                "autoPause": true,
            })
        );
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
