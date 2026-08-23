//! Global API context resolution.
//!
//! Mirrors the `output` module pattern: call `init()` once at startup,
//! then `resolve()` / `resolve_with_auth()` from any command handler.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::auth::{self, Auth};
use crate::config::settings::{ApiMode, EnvVars};

static LOCAL: AtomicBool = AtomicBool::new(false);
static API_URL: OnceLock<Option<String>> = OnceLock::new();

/// Store the global `--local` and `--api-url` values. Called once at startup.
pub fn init(local: bool, api_url: Option<String>) {
    LOCAL.store(local, Ordering::Relaxed);
    API_URL.get_or_init(|| api_url);
}

fn api_url() -> Option<&'static str> {
    API_URL.get().and_then(|o| o.as_deref())
}

fn resolve_mode(local: bool, api_url: Option<&str>, configured_instance: Option<&str>) -> ApiMode {
    ApiMode::resolve(local || configured_instance == Some("local"), api_url)
}

/// Resolve the API mode from global flags + config.
pub fn mode() -> ApiMode {
    let config = crate::config::settings::read_config().ok();
    resolve_mode(
        LOCAL.load(Ordering::Relaxed),
        api_url(),
        config.as_ref().and_then(|c| c.instance.as_deref()),
    )
}

/// Resolve API mode and base URL from global flags + env + config.
pub fn resolve() -> (ApiMode, String) {
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
    let mode = resolve_mode(
        LOCAL.load(Ordering::Relaxed),
        api_url(),
        config.as_ref().and_then(|c| c.instance.as_deref()),
    );
    let local_config_url = config.as_ref().and_then(|c| c.local_api_url());
    let base_url = mode.resolve_base_url(api_url(), &env_vars, local_config_url);
    (mode, base_url)
}

/// Resolve API mode, base URL, and auth credentials.
pub fn resolve_with_auth() -> (ApiMode, String, Auth) {
    let (mode, base_url) = resolve();
    let auth = auth::resolve_auth();
    (mode, base_url, auth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_local_instance_selects_local_mode() {
        assert_eq!(resolve_mode(false, None, Some("local")), ApiMode::Local);
    }

    #[test]
    fn configured_cloud_instance_keeps_cloud_mode() {
        assert_eq!(resolve_mode(false, None, Some("cloud")), ApiMode::Cloud);
    }

    #[test]
    fn explicit_api_url_still_selects_local_mode() {
        assert_eq!(
            resolve_mode(false, Some("http://steel.example/v1"), Some("cloud")),
            ApiMode::Local
        );
    }

    #[test]
    fn local_flag_still_selects_local_mode() {
        assert_eq!(resolve_mode(true, None, Some("cloud")), ApiMode::Local);
    }
}
