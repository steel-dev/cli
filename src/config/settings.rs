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
    /// Resolve the base URL for API requests.
    ///
    /// Priority: explicit `--api-url` → env var → config → default.
    /// Matches TS `resolveApiBaseUrl()` in `apiConfig.ts`.
    pub fn resolve_base_url(
        &self,
        explicit_url: Option<&str>,
        env_vars: &EnvVars,
        local_config_url: Option<&str>,
    ) -> String {
        if let Some(url) = explicit_url {
            return normalize_api_url(url);
        }

        match self {
            Self::Cloud => {
                let url = env_vars
                    .steel_api_url
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(DEFAULT_API_URL);
                normalize_api_url(url)
            }
            Self::Local => {
                // STEEL_BROWSER_API_URL → STEEL_LOCAL_API_URL → config → default
                let url = env_vars
                    .steel_browser_api_url
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| {
                        env_vars
                            .steel_local_api_url
                            .as_deref()
                            .filter(|s| !s.trim().is_empty())
                    })
                    .or_else(|| local_config_url.filter(|s| !s.trim().is_empty()))
                    .unwrap_or(DEFAULT_LOCAL_API_URL);
                normalize_api_url(url)
            }
        }
    }

    /// Determine mode from CLI flags.
    /// Matches TS `resolveSessionMode()` / `resolveMode()`.
    pub const fn resolve(local: bool, api_url: Option<&str>) -> Self {
        if local {
            return Self::Local;
        }
        if api_url.is_some() {
            return Self::Local;
        }
        Self::Cloud
    }
}

/// Env vars relevant to API URL resolution (injectable for testing).
#[derive(Debug, Default)]
pub struct EnvVars {
    pub steel_api_url: Option<String>,
    pub steel_browser_api_url: Option<String>,
    pub steel_local_api_url: Option<String>,
}

impl EnvVars {
    pub fn from_env() -> Self {
        Self {
            steel_api_url: std::env::var("STEEL_API_URL").ok(),
            steel_browser_api_url: std::env::var("STEEL_BROWSER_API_URL").ok(),
            steel_local_api_url: std::env::var("STEEL_LOCAL_API_URL").ok(),
        }
    }
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
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
}

impl Config {
    /// Extract the local API URL from config.browser.apiUrl
    pub fn local_api_url(&self) -> Option<&str> {
        self.browser
            .as_ref()
            .and_then(|b| b.api_url.as_deref())
            .filter(|s| !s.trim().is_empty())
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

pub fn write_config(config: &Config) -> Result<()> {
    write_config_to(&super::config_path(), config)
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
        };

        write_config_to(&path, &config).unwrap();
        let loaded = read_config_from(&path).unwrap();

        assert_eq!(loaded.api_key.as_deref(), Some("sk-test-123"));
        assert_eq!(loaded.name.as_deref(), Some("CLI"));
        assert_eq!(loaded.instance.as_deref(), Some("cloud"));
        assert_eq!(loaded.local_api_url(), Some("http://localhost:4000/v1"));
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
            r#"{"apiKey":"k","name":"n","instance":"cloud","browser":{"apiUrl":"http://x"}}"#,
        )
        .unwrap();

        let config = read_config_from(&path).unwrap();
        assert_eq!(config.api_key.as_deref(), Some("k"));
        assert_eq!(config.local_api_url(), Some("http://x"));
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

    // --- ApiMode tests ---

    #[test]
    fn resolve_mode_default_is_cloud() {
        assert_eq!(ApiMode::resolve(false, None), ApiMode::Cloud);
    }

    #[test]
    fn resolve_mode_local_flag() {
        assert_eq!(ApiMode::resolve(true, None), ApiMode::Local);
    }

    #[test]
    fn resolve_mode_api_url_implies_local() {
        assert_eq!(
            ApiMode::resolve(false, Some("http://localhost:3000")),
            ApiMode::Local
        );
    }

    #[test]
    fn cloud_base_url_default() {
        let env = EnvVars::default();
        let url = ApiMode::Cloud.resolve_base_url(None, &env, None);
        assert_eq!(url, "https://api.steel.dev/v1");
    }

    #[test]
    fn cloud_base_url_from_env() {
        let env = EnvVars {
            steel_api_url: Some("https://custom.api.dev/v1/".into()),
            ..Default::default()
        };
        let url = ApiMode::Cloud.resolve_base_url(None, &env, None);
        assert_eq!(url, "https://custom.api.dev/v1");
    }

    #[test]
    fn cloud_base_url_explicit_overrides_env() {
        let env = EnvVars {
            steel_api_url: Some("https://from-env.dev/".into()),
            ..Default::default()
        };
        let url = ApiMode::Cloud.resolve_base_url(Some("https://explicit.dev/v1/"), &env, None);
        assert_eq!(url, "https://explicit.dev/v1");
    }

    #[test]
    fn local_base_url_default() {
        let env = EnvVars::default();
        let url = ApiMode::Local.resolve_base_url(None, &env, None);
        assert_eq!(url, "http://localhost:3000/v1");
    }

    #[test]
    fn local_base_url_browser_api_url_takes_priority() {
        let env = EnvVars {
            steel_browser_api_url: Some("http://browser:4000/v1".into()),
            steel_local_api_url: Some("http://local:5000/v1".into()),
            ..Default::default()
        };
        let url = ApiMode::Local.resolve_base_url(None, &env, None);
        assert_eq!(url, "http://browser:4000/v1");
    }

    #[test]
    fn local_base_url_falls_to_local_env() {
        let env = EnvVars {
            steel_local_api_url: Some("http://local:5000/v1".into()),
            ..Default::default()
        };
        let url = ApiMode::Local.resolve_base_url(None, &env, None);
        assert_eq!(url, "http://local:5000/v1");
    }

    #[test]
    fn local_base_url_falls_to_config() {
        let env = EnvVars::default();
        let url = ApiMode::Local.resolve_base_url(None, &env, Some("http://from-config:6000/v1"));
        assert_eq!(url, "http://from-config:6000/v1");
    }

    #[test]
    fn local_base_url_empty_env_skipped() {
        let env = EnvVars {
            steel_browser_api_url: Some("  ".into()),
            steel_local_api_url: Some("".into()),
            ..Default::default()
        };
        let url = ApiMode::Local.resolve_base_url(None, &env, None);
        assert_eq!(url, "http://localhost:3000/v1");
    }

    #[test]
    fn trailing_slashes_stripped() {
        let env = EnvVars::default();
        let url = ApiMode::Cloud.resolve_base_url(Some("https://api.dev///"), &env, None);
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
