// ABOUTME: Steel API /projects bindings and current-environment resolution.
// ABOUTME: Reports which project an API key is scoped to and whether it is production.

use serde::Deserialize;
use serde_json::Value;

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub is_production: bool,
    #[serde(default)]
    pub is_default: bool,
}

/// Parse the `/projects` response body into a list of projects.
/// Returns an empty list when the body has no parsable `projects` array.
pub fn parse_projects(data: &Value) -> Vec<Project> {
    data.get("projects")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Human label for a project's production flag.
pub const fn environment_label(is_production: bool) -> &'static str {
    if is_production {
        "production"
    } else {
        "development"
    }
}

impl SteelClient {
    /// Fetch the projects visible to the current API key.
    pub async fn get_projects(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            "/projects",
            None,
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- parse_projects ---

    #[test]
    fn parse_full_body() {
        let data = json!({
            "projects": [{
                "id": "b4b8ce24-70cc-45aa-9233-c5e01fe24168",
                "name": "Default project",
                "slug": "default",
                "isProduction": false,
                "isDefault": true,
                "promotedToProductionAt": null,
                "createdAt": "2026-06-07T12:14:56.913Z",
                "updatedAt": "2026-06-07T12:14:56.913Z"
            }]
        });

        let projects = parse_projects(&data);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Default project");
        assert_eq!(projects[0].slug, "default");
        assert!(!projects[0].is_production);
        assert!(projects[0].is_default);
    }

    #[test]
    fn parse_empty_list() {
        let data = json!({"projects": []});
        assert!(parse_projects(&data).is_empty());
    }

    #[test]
    fn parse_missing_key() {
        let data = json!({"sessions": []});
        assert!(parse_projects(&data).is_empty());
    }

    #[test]
    fn parse_malformed_entries() {
        let data = json!({"projects": [{"id": 42}]});
        assert!(parse_projects(&data).is_empty());
    }

    #[test]
    fn parse_missing_flags_default_false() {
        let data = json!({
            "projects": [{"id": "x", "name": "Bare", "slug": "bare"}]
        });

        let projects = parse_projects(&data);
        assert_eq!(projects.len(), 1);
        assert!(!projects[0].is_production);
        assert!(!projects[0].is_default);
    }

    // --- environment_label ---

    #[test]
    fn label_production() {
        assert_eq!(environment_label(true), "production");
    }

    #[test]
    fn label_development() {
        assert_eq!(environment_label(false), "development");
    }

    // --- get_projects against a mock server ---

    use crate::config::auth::AuthSource;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_projects_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/projects"))
            .and(header("Steel-Api-Key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "projects": [{
                    "id": "p1",
                    "name": "Default project",
                    "slug": "default",
                    "isProduction": false,
                    "isDefault": true
                }]
            })))
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let auth = Auth {
            api_key: Some("test-key".into()),
            source: AuthSource::Env,
        };

        let data = client
            .get_projects(&server.uri(), ApiMode::Cloud, &auth)
            .await
            .unwrap();

        let projects = parse_projects(&data);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].slug, "default");
        assert_eq!(environment_label(projects[0].is_production), "development");
    }
}
