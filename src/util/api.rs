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

fn is_local() -> bool {
    LOCAL.load(Ordering::Relaxed)
}

fn api_url() -> Option<&'static str> {
    API_URL.get().and_then(|o| o.as_deref())
}

/// Resolve the API mode from global flags.
pub fn mode() -> ApiMode {
    ApiMode::resolve(is_local(), api_url())
}

/// Resolve API mode and base URL from global flags + env + config.
pub fn resolve() -> (ApiMode, String) {
    let mode = ApiMode::resolve(is_local(), api_url());
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
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
