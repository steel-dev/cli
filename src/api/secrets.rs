use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::util::pairs::parse_pairs;

pub fn with_project(path: &str, project_id: Option<&str>) -> String {
    match project_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(project) => format!("{path}?projectId={}", urlencoding::encode(project)),
        None => path.to_string(),
    }
}

pub fn secret_path(id: &str, project_id: Option<&str>) -> String {
    with_project(&format!("/secrets/{}", urlencoding::encode(id)), project_id)
}

pub fn is_secret_id(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    parts.len() == 5
        && parts.iter().zip([8, 4, 4, 4, 12]).all(|(part, len)| {
            part.len() == len && part.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
}

pub fn parse_secret_ids(pairs: &[String]) -> Result<BTreeMap<String, String>> {
    let bindings = parse_pairs("--secret", "NAME=SECRET_ID", pairs)?;
    for (name, id) in &bindings {
        if !is_secret_id(id) {
            bail!(
                "--secret {name} wants a secret id, got {id:?}. Store the value with `steel secret create` first."
            );
        }
    }
    Ok(bindings)
}

#[derive(Debug, Default, Clone)]
pub struct CreateSecret {
    pub name: String,
    pub value: String,
    pub project_id: Option<String>,
}

impl CreateSecret {
    pub fn body(&self) -> Value {
        let mut body = json!({ "name": self.name, "value": self.value });
        if let Some(project) = &self.project_id {
            body["projectId"] = json!(project);
        }
        body
    }
}

#[derive(Debug, Default, Clone)]
pub struct UpdateSecret {
    pub name: Option<String>,
    pub value: Option<String>,
}

impl UpdateSecret {
    pub const fn is_empty(&self) -> bool {
        self.name.is_none() && self.value.is_none()
    }

    pub fn body(&self) -> Value {
        let mut body = json!({});
        if let Some(name) = &self.name {
            body["name"] = json!(name);
        }
        if let Some(value) = &self.value {
            body["value"] = json!(value);
        }
        body
    }
}

impl SteelClient {
    pub async fn list_secrets(
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
            &with_project("/secrets", project_id),
            None,
            auth,
        )
        .await
    }

    pub async fn create_secret(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        request: &CreateSecret,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/secrets",
            Some(request.body()),
            auth,
        )
        .await
    }

    pub async fn get_secret(
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
            &secret_path(id, project_id),
            None,
            auth,
        )
        .await
    }

    pub async fn update_secret(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        id: &str,
        project_id: Option<&str>,
        request: &UpdateSecret,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::PATCH,
            &secret_path(id, project_id),
            Some(request.body()),
            auth,
        )
        .await
    }

    pub async fn delete_secret(
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
            &secret_path(id, project_id),
            None,
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b31";

    #[test]
    fn paths_encode_ids_and_carry_the_project() {
        assert_eq!(secret_path("a b", None), "/secrets/a%20b");
        assert_eq!(
            secret_path(ID, Some("proj 1")),
            format!("/secrets/{ID}?projectId=proj%201")
        );
        assert_eq!(with_project("/secrets", Some("  ")), "/secrets");
    }

    #[test]
    fn secret_ids_are_uuids() {
        assert!(is_secret_id(ID));
        assert!(is_secret_id("00000000-0000-0000-0000-000000000000"));
        assert!(!is_secret_id("sk-live-abc"));
        assert!(!is_secret_id("2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b3"));
        assert!(!is_secret_id("2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b3g"));
    }

    #[test]
    fn secret_bindings_refuse_plain_values() {
        let bindings = parse_secret_ids(&[format!("TOKEN={ID}")]).unwrap();
        assert_eq!(bindings["TOKEN"], ID);
        let err = parse_secret_ids(&["TOKEN=sk-live-abc".into()]).unwrap_err();
        assert!(err.to_string().contains("steel secret create"));
    }

    #[test]
    fn bodies_only_carry_given_fields() {
        let create = CreateSecret {
            name: "TOKEN".into(),
            value: "abc".into(),
            project_id: None,
        };
        assert_eq!(create.body(), json!({ "name": "TOKEN", "value": "abc" }));
        let scoped = CreateSecret {
            project_id: Some("proj".into()),
            ..create
        };
        assert_eq!(
            scoped.body(),
            json!({ "name": "TOKEN", "value": "abc", "projectId": "proj" })
        );
        assert!(UpdateSecret::default().is_empty());
        let update = UpdateSecret {
            name: Some("API_TOKEN".into()),
            value: None,
        };
        assert_eq!(update.body(), json!({ "name": "API_TOKEN" }));
    }
}
