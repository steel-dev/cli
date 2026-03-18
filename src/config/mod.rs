pub mod auth;
pub mod settings;

use std::path::{Path, PathBuf};

pub const DEFAULT_API_URL: &str = "https://api.steel.dev/v1";
pub const DEFAULT_LOCAL_API_URL: &str = "http://localhost:3000/v1";
pub const LOGIN_URL: &str = "https://app.steel.dev/sign-in";
pub const SUCCESS_URL: &str = "https://app.steel.dev/sign-in/cli-success";
pub const API_KEYS_URL: &str = "https://api.steel.dev/v1/api-keys";
pub const REPO_URL: &str = "https://github.com/steel-dev/steel-browser.git";

/// Resolve the Steel config directory.
/// `STEEL_CONFIG_DIR` env var → `~/.config/steel`
pub fn config_dir_with(env_val: Option<&str>) -> PathBuf {
    if let Some(dir) = env_val {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("steel")
}

pub fn config_dir() -> PathBuf {
    config_dir_with(std::env::var("STEEL_CONFIG_DIR").ok().as_deref())
}

pub fn config_path_in(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

pub fn config_path() -> PathBuf {
    config_path_in(&config_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_with_env() {
        let dir = config_dir_with(Some("/tmp/steel-test"));
        assert_eq!(dir, PathBuf::from("/tmp/steel-test"));
    }

    #[test]
    fn config_dir_with_empty_env() {
        let dir = config_dir_with(Some("  "));
        // Falls back to ~/.config/steel
        assert!(dir.ends_with(".config/steel"));
    }

    #[test]
    fn config_dir_with_none() {
        let dir = config_dir_with(None);
        assert!(dir.ends_with(".config/steel"));
    }

    #[test]
    fn config_path_in_dir() {
        let path = config_path_in(Path::new("/tmp/steel"));
        assert_eq!(path, PathBuf::from("/tmp/steel/config.json"));
    }
}
