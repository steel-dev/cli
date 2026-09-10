use std::path::Path;

use clap::Parser;

use crate::config;
use crate::config::settings::{read_config_from, write_config_to};
use crate::status;

const ENV_KEY_WARNING: &str = "warning: STEEL_API_KEY is set in your environment and will still be used for authentication. Run `unset STEEL_API_KEY` to fully log out.";

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let env_key_set = env_key_present(std::env::var("STEEL_API_KEY").ok().as_deref());
    let cleared = clear_saved_login(&config::config_path())?;

    match (cleared, env_key_set) {
        (true, _) => status!("Successfully logged out. Have a great day!"),
        (false, false) => status!("You are not logged in."),
        (false, true) => status!("No saved login to clear."),
    }

    if env_key_set {
        status!("{ENV_KEY_WARNING}");
    }

    Ok(())
}

fn env_key_present(env_api_key: Option<&str>) -> bool {
    env_api_key.is_some_and(|key| !key.trim().is_empty())
}

fn clear_saved_login(config_path: &Path) -> anyhow::Result<bool> {
    let Ok(mut cfg) = read_config_from(config_path) else {
        return Ok(false);
    };

    if cfg.api_key.is_none() {
        return Ok(false);
    }

    cfg.api_key = None;
    cfg.name = None;
    write_config_to(config_path, &cfg)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn env_key_present_detects_non_empty_value() {
        assert!(env_key_present(Some("sk-123")));
    }

    #[test]
    fn env_key_present_ignores_empty_or_whitespace() {
        assert!(!env_key_present(None));
        assert!(!env_key_present(Some("")));
        assert!(!env_key_present(Some("   ")));
    }

    #[test]
    fn clear_saved_login_removes_key_and_name() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"apiKey":"k","name":"n","instance":"cloud"}"#).unwrap();

        assert!(clear_saved_login(&path).unwrap());

        let cfg = read_config_from(&path).unwrap();
        assert!(cfg.api_key.is_none());
        assert!(cfg.name.is_none());
        assert_eq!(cfg.instance.as_deref(), Some("cloud"));
    }

    #[test]
    fn clear_saved_login_reports_false_without_key() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"instance":"cloud"}"#).unwrap();

        assert!(!clear_saved_login(&path).unwrap());
    }

    #[test]
    fn clear_saved_login_reports_false_when_config_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.json");

        assert!(!clear_saved_login(&path).unwrap());
    }
}
