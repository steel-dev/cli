use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::api::secrets::with_project;
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

pub fn environment_path(id: &str, project_id: Option<&str>) -> String {
    with_project(
        &format!("/environments/{}", urlencoding::encode(id)),
        project_id,
    )
}

#[derive(Debug, Default, Clone)]
pub struct EnvironmentSpec {
    pub template: Option<String>,
    pub vcpu: Option<u32>,
    pub memory_mib: Option<u32>,
    pub disk_mib: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub auto_pause: Option<bool>,
    pub env: BTreeMap<String, String>,
}

impl EnvironmentSpec {
    pub fn is_empty(&self) -> bool {
        self.template.is_none()
            && self.vcpu.is_none()
            && self.memory_mib.is_none()
            && self.disk_mib.is_none()
            && self.timeout_seconds.is_none()
            && self.auto_pause.is_none()
            && self.env.is_empty()
    }

    pub fn apply_to(&self, current: Value) -> Value {
        let mut spec = match current {
            Value::Object(fields) => fields,
            _ => Map::new(),
        };
        if let Some(template) = &self.template {
            spec.insert("template".into(), json!(template));
        }
        if let Some(vcpu) = self.vcpu {
            spec.insert("vcpu".into(), json!(vcpu));
        }
        if let Some(memory) = self.memory_mib {
            spec.insert("memoryMib".into(), json!(memory));
        }
        if let Some(disk) = self.disk_mib {
            spec.insert("diskMib".into(), json!(disk));
        }
        if let Some(timeout) = self.timeout_seconds {
            spec.insert("timeoutSeconds".into(), json!(timeout));
        }
        if let Some(auto_pause) = self.auto_pause {
            spec.insert("autoPause".into(), json!(auto_pause));
        }
        if !self.env.is_empty() {
            let mut env = match spec.remove("env") {
                Some(Value::Object(fields)) => fields,
                _ => Map::new(),
            };
            for (name, value) in &self.env {
                env.insert(name.clone(), json!(value));
            }
            spec.insert("env".into(), Value::Object(env));
        }
        Value::Object(spec)
    }

    pub fn body(&self) -> Value {
        self.apply_to(json!({}))
    }
}

#[derive(Debug, Default, Clone)]
pub struct CreateEnvironment {
    pub name: String,
    pub project_id: Option<String>,
    pub spec: Option<Value>,
    pub secrets: BTreeMap<String, Value>,
}

impl CreateEnvironment {
    pub fn body(&self) -> Value {
        let mut body = json!({ "name": self.name });
        if let Some(project) = &self.project_id {
            body["projectId"] = json!(project);
        }
        if let Some(spec) = &self.spec {
            body["spec"] = spec.clone();
        }
        if !self.secrets.is_empty() {
            body["secrets"] = json!(self.secrets);
        }
        body
    }
}

#[derive(Debug, Default, Clone)]
pub struct UpdateEnvironment {
    pub name: Option<String>,
    pub spec: Option<Value>,
    pub secrets: BTreeMap<String, Value>,
}

impl UpdateEnvironment {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.spec.is_none() && self.secrets.is_empty()
    }

    pub fn body(&self) -> Value {
        let mut body = json!({});
        if let Some(name) = &self.name {
            body["name"] = json!(name);
        }
        if let Some(spec) = &self.spec {
            body["spec"] = spec.clone();
        }
        if !self.secrets.is_empty() {
            body["secrets"] = json!(self.secrets);
        }
        body
    }
}

impl SteelClient {
    pub async fn list_environments(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        project_id: Option<&str>,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &with_project("/environments", project_id),
            None,
            auth,
        )
        .await
    }

    pub async fn create_environment(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        request: &CreateEnvironment,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/environments",
            Some(request.body()),
            auth,
        )
        .await
    }

    pub async fn get_environment(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
        project_id: Option<&str>,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &environment_path(id, project_id),
            None,
            auth,
        )
        .await
    }

    pub async fn update_environment(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
        project_id: Option<&str>,
        request: &UpdateEnvironment,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::PATCH,
            &environment_path(id, project_id),
            Some(request.body()),
            auth,
        )
        .await
    }

    pub async fn delete_environment(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
        project_id: Option<&str>,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::DELETE,
            &environment_path(id, project_id),
            None,
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_encode_ids_and_carry_the_project() {
        assert_eq!(environment_path("env 1", None), "/environments/env%201");
        assert_eq!(
            environment_path("env", Some("proj")),
            "/environments/env?projectId=proj"
        );
    }

    #[test]
    fn spec_merges_into_the_current_one() {
        let spec = EnvironmentSpec {
            memory_mib: Some(4096),
            auto_pause: Some(true),
            env: BTreeMap::from([("B".into(), "2".into())]),
            ..Default::default()
        };
        assert!(!spec.is_empty());
        assert!(EnvironmentSpec::default().is_empty());
        let merged = spec.apply_to(json!({
            "template": "steel",
            "vcpu": 2,
            "env": { "A": "1" }
        }));
        assert_eq!(
            merged,
            json!({
                "template": "steel",
                "vcpu": 2,
                "memoryMib": 4096,
                "autoPause": true,
                "env": { "A": "1", "B": "2" }
            })
        );
        assert_eq!(
            spec.body(),
            json!({ "memoryMib": 4096, "autoPause": true, "env": { "B": "2" } })
        );
    }

    #[test]
    fn bodies_only_carry_given_fields() {
        let create = CreateEnvironment {
            name: "dev".into(),
            ..Default::default()
        };
        assert_eq!(create.body(), json!({ "name": "dev" }));
        let full = CreateEnvironment {
            name: "dev".into(),
            project_id: Some("proj".into()),
            spec: Some(json!({ "template": "steel" })),
            secrets: BTreeMap::from([
                ("TOKEN".into(), json!({ "secretId": "id" })),
                ("KEY".into(), json!({ "value": "abc" })),
            ]),
        };
        assert_eq!(
            full.body(),
            json!({
                "name": "dev",
                "projectId": "proj",
                "spec": { "template": "steel" },
                "secrets": { "TOKEN": { "secretId": "id" }, "KEY": { "value": "abc" } }
            })
        );
        assert!(UpdateEnvironment::default().is_empty());
        let update = UpdateEnvironment {
            secrets: BTreeMap::from([("OLD".into(), Value::Null)]),
            ..Default::default()
        };
        assert_eq!(update.body(), json!({ "secrets": { "OLD": null } }));
    }
}
