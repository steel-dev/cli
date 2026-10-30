use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::{DEFAULT_API_URL, DEFAULT_LOCAL_API_URL};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiMode {
    Cloud,
    Local,
}

impl std::fmt::Display for ApiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cloud => write!(f, "cloud"),
            Self::Local => write!(f, "local"),
        }
    }
}

impl ApiMode {
    pub fn for_url(url: &str) -> Self {
        let cloud = url::Url::parse(url)
            .ok()
            .and_then(|parsed| {
                parsed
                    .host_str()
                    .map(|host| host == "steel.dev" || host.ends_with(".steel.dev"))
            })
            .unwrap_or(false);
        if cloud { Self::Cloud } else { Self::Local }
    }
}

pub fn resolve_base_url(
    explicit: Option<&str>,
    local: bool,
    env_api_url: Option<&str>,
    local_config_url: Option<&str>,
) -> String {
    let url = non_empty(explicit)
        .or_else(|| local.then(|| non_empty(local_config_url).unwrap_or(DEFAULT_LOCAL_API_URL)))
        .or_else(|| non_empty(env_api_url))
        .unwrap_or(DEFAULT_API_URL);
    normalize_api_url(url)
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.trim().is_empty())
}

fn normalize_api_url(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<BrowserConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onboarding: Option<OnboardingConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

impl Config {
    /// Extract the local API URL from config.browser.apiUrl
    pub fn local_api_url(&self) -> Option<&str> {
        self.browser
            .as_ref()
            .and_then(|b| b.api_url.as_deref())
            .filter(|s| !s.trim().is_empty())
    }

    pub fn telemetry_disabled(&self) -> bool {
        self.telemetry
            .as_ref()
            .and_then(|t| t.disabled)
            .unwrap_or(false)
    }

    pub fn onboarding_source(&self) -> Option<&str> {
        self.onboarding
            .as_ref()
            .and_then(|o| o.source.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

pub fn read_config_from(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn read_config() -> Result<Config> {
    read_config_from(&super::config_path())
}

pub fn write_config_to(path: &Path, config: &Config) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
        }
    }
    let contents = serde_json::to_string_pretty(config)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &contents)?;
    std::fs::rename(&tmp, path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn config_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        let config = Config {
            api_key: Some("sk-test-123".into()),
            name: Some("CLI".into()),
            instance: Some("cloud".into()),
            browser: Some(BrowserConfig {
                api_url: Some("http://localhost:4000/v1".into()),
            }),
            telemetry: Some(TelemetryConfig {
                disabled: Some(true),
            }),
            onboarding: Some(OnboardingConfig {
                source: Some("claude-code".into()),
            }),
        };

        write_config_to(&path, &config).unwrap();
        let loaded = read_config_from(&path).unwrap();

        assert_eq!(loaded.api_key.as_deref(), Some("sk-test-123"));
        assert_eq!(loaded.name.as_deref(), Some("CLI"));
        assert_eq!(loaded.instance.as_deref(), Some("cloud"));
        assert_eq!(loaded.local_api_url(), Some("http://localhost:4000/v1"));
        assert!(loaded.telemetry_disabled());
        assert_eq!(loaded.onboarding_source(), Some("claude-code"));
    }

    #[test]
    fn onboarding_source_ignores_blank_values() {
        let config = Config {
            onboarding: Some(OnboardingConfig {
                source: Some("   ".into()),
            }),
            ..Default::default()
        };
        assert_eq!(config.onboarding_source(), None);
        assert_eq!(Config::default().onboarding_source(), None);
    }

    #[test]
    fn config_camel_case_serialization() {
        let config = Config {
            api_key: Some("key".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("apiKey"));
        assert!(!json.contains("api_key"));
    }

    #[test]
    fn config_reads_camel_case_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"apiKey":"k","name":"n","instance":"cloud","browser":{"apiUrl":"http://x"},"telemetry":{"disabled":true}}"#,
        )
        .unwrap();

        let config = read_config_from(&path).unwrap();
        assert_eq!(config.api_key.as_deref(), Some("k"));
        assert_eq!(config.local_api_url(), Some("http://x"));
        assert!(config.telemetry_disabled());
    }

    #[test]
    fn config_missing_file_returns_error() {
        let result = read_config_from(Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
    }

    #[test]
    fn config_empty_fields_not_serialized() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn local_api_url_empty_string_returns_none() {
        let config = Config {
            browser: Some(BrowserConfig {
                api_url: Some("  ".into()),
            }),
            ..Default::default()
        };
        assert_eq!(config.local_api_url(), None);
    }

    #[test]
    fn telemetry_disabled_defaults_false() {
        let config = Config::default();
        assert!(!config.telemetry_disabled());
    }

    // --- ApiMode tests ---

    #[test]
    fn steel_dev_hosts_are_cloud() {
        assert_eq!(ApiMode::for_url("https://api.steel.dev/v1"), ApiMode::Cloud);
        assert_eq!(
            ApiMode::for_url("https://api.staging.steel.dev/v1"),
            ApiMode::Cloud
        );
        assert_eq!(ApiMode::for_url("http://localhost:3000/v1"), ApiMode::Local);
        assert_eq!(
            ApiMode::for_url("https://browser.example.com/v1"),
            ApiMode::Local
        );
        assert_eq!(ApiMode::for_url("https://notsteel.dev/v1"), ApiMode::Local);
        assert_eq!(ApiMode::for_url("not a url"), ApiMode::Local);
    }

    #[test]
    fn base_url_defaults_to_cloud() {
        assert_eq!(
            resolve_base_url(None, false, None, None),
            "https://api.steel.dev/v1"
        );
    }

    #[test]
    fn base_url_reads_steel_api_url() {
        let url = resolve_base_url(None, false, Some("http://browser:4000/v1/"), None);
        assert_eq!(url, "http://browser:4000/v1");
    }

    #[test]
    fn local_flag_uses_config_or_localhost() {
        assert_eq!(
            resolve_base_url(None, true, None, None),
            "http://localhost:3000/v1"
        );
        let url = resolve_base_url(None, true, None, Some("http://from-config:6000/v1"));
        assert_eq!(url, "http://from-config:6000/v1");
        let url = resolve_base_url(None, true, Some("https://api.steel.dev/v1"), None);
        assert_eq!(url, "http://localhost:3000/v1");
    }

    #[test]
    fn explicit_url_wins() {
        let url = resolve_base_url(
            Some("https://explicit.dev/v1/"),
            true,
            Some("https://from-env.dev"),
            Some("http://from-config:6000/v1"),
        );
        assert_eq!(url, "https://explicit.dev/v1");
    }

    #[test]
    fn empty_values_are_skipped() {
        let url = resolve_base_url(Some(" "), true, Some(""), Some("  "));
        assert_eq!(url, "http://localhost:3000/v1");
        assert_eq!(
            resolve_base_url(None, false, Some(" "), None),
            "https://api.steel.dev/v1"
        );
    }

    #[test]
    fn trailing_slashes_stripped() {
        let url = resolve_base_url(Some("https://api.dev///"), false, None, None);
        assert_eq!(url, "https://api.dev");
    }

    #[test]
    fn api_mode_display_lowercase() {
        assert_eq!(ApiMode::Cloud.to_string(), "cloud");
        assert_eq!(ApiMode::Local.to_string(), "local");
    }

    #[test]
    fn api_mode_serde_roundtrip() {
        let cloud_json = serde_json::to_string(&ApiMode::Cloud).unwrap();
        assert_eq!(cloud_json, "\"cloud\"");
        let local_json = serde_json::to_string(&ApiMode::Local).unwrap();
        assert_eq!(local_json, "\"local\"");

        let parsed: ApiMode = serde_json::from_str("\"cloud\"").unwrap();
        assert_eq!(parsed, ApiMode::Cloud);
    }
}
