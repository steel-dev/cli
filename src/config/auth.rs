use std::path::Path;

use crate::config::settings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthSource {
    Env,
    Config,
    None,
}

#[derive(Debug, Clone)]
pub struct Auth {
    pub api_key: Option<String>,
    pub source: AuthSource,
}

/// Resolve API key with explicit inputs (testable).
///
/// Priority: env var → config file → none.
/// Matches TS `resolveBrowserAuth()` in `auth.ts`.
pub fn resolve_auth_with(
    env_api_key: Option<&str>,
    config_api_key: Option<&str>,
) -> Auth {
    // 1. STEEL_API_KEY env var
    if let Some(key) = env_api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Auth {
                api_key: Some(trimmed.to_string()),
                source: AuthSource::Env,
            };
        }
    }

    // 2. config.json apiKey
    if let Some(key) = config_api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Auth {
                api_key: Some(trimmed.to_string()),
                source: AuthSource::Config,
            };
        }
    }

    Auth {
        api_key: None,
        source: AuthSource::None,
    }
}

/// Read the API key from a config file at the given path.
pub fn read_api_key_from_config(path: &Path) -> Option<String> {
    settings::read_config_from(path)
        .ok()
        .and_then(|c| c.api_key)
        .filter(|k| !k.trim().is_empty())
}

/// Resolve API key from environment and default config path.
pub fn resolve_auth() -> Auth {
    let env_key = std::env::var("STEEL_API_KEY").ok();
    let config_key = read_api_key_from_config(&super::config_path());

    resolve_auth_with(env_key.as_deref(), config_key.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn env_key_takes_priority() {
        let auth = resolve_auth_with(Some("env-key"), Some("config-key"));
        assert_eq!(auth.api_key.as_deref(), Some("env-key"));
        assert_eq!(auth.source, AuthSource::Env);
    }

    #[test]
    fn falls_back_to_config_key() {
        let auth = resolve_auth_with(None, Some("config-key"));
        assert_eq!(auth.api_key.as_deref(), Some("config-key"));
        assert_eq!(auth.source, AuthSource::Config);
    }

    #[test]
    fn no_key_returns_none() {
        let auth = resolve_auth_with(None, None);
        assert_eq!(auth.api_key, None);
        assert_eq!(auth.source, AuthSource::None);
    }

    #[test]
    fn empty_env_key_skipped() {
        let auth = resolve_auth_with(Some("  "), Some("config-key"));
        assert_eq!(auth.api_key.as_deref(), Some("config-key"));
        assert_eq!(auth.source, AuthSource::Config);
    }

    #[test]
    fn empty_config_key_skipped() {
        let auth = resolve_auth_with(None, Some("  "));
        assert_eq!(auth.api_key, None);
        assert_eq!(auth.source, AuthSource::None);
    }

    #[test]
    fn keys_are_trimmed() {
        let auth = resolve_auth_with(Some("  trimmed-key  "), None);
        assert_eq!(auth.api_key.as_deref(), Some("trimmed-key"));
    }

    #[test]
    fn read_api_key_from_config_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"apiKey": "from-file"}"#).unwrap();

        assert_eq!(read_api_key_from_config(&path).as_deref(), Some("from-file"));
    }

    #[test]
    fn read_api_key_missing_file() {
        assert_eq!(
            read_api_key_from_config(Path::new("/nonexistent/config.json")),
            None
        );
    }

    #[test]
    fn read_api_key_empty_value() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"apiKey": "  "}"#).unwrap();

        assert_eq!(read_api_key_from_config(&path), None);
    }
}
