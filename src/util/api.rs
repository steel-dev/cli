//! Global API context resolution.
//!
//! Mirrors the `output` module pattern: call `init()` once at startup,
//! then `resolve()` / `resolve_with_auth()` from any command handler.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::auth::{self, Auth};
use crate::config::settings::{ApiMode, Config, resolve_base_url};

static LOCAL: AtomicBool = AtomicBool::new(false);
static API_URL: OnceLock<Option<String>> = OnceLock::new();

const REMOVED_ENV_VARS: &[&str] = &["STEEL_BROWSER_API_URL", "STEEL_LOCAL_API_URL"];

/// Store the global `--local` and `--api-url` values. Called once at startup.
pub fn init(local: bool, api_url: Option<String>) {
    LOCAL.store(local, Ordering::Relaxed);
    API_URL.get_or_init(|| api_url);
    for name in REMOVED_ENV_VARS {
        if std::env::var_os(name).is_some() {
            eprintln!("warning: {name} is no longer read. Use STEEL_API_URL or --api-url.");
        }
    }
}

fn is_local() -> bool {
    LOCAL.load(Ordering::Relaxed)
}

fn api_url() -> Option<&'static str> {
    API_URL.get().and_then(|o| o.as_deref())
}

/// Resolve API mode and base URL from global flags + env + config.
pub fn resolve() -> (ApiMode, String) {
    let config = crate::config::settings::read_config().ok();
    let env_api_url = std::env::var("STEEL_API_URL").ok();
    let base_url = resolve_base_url(
        api_url(),
        is_local(),
        env_api_url.as_deref(),
        config.as_ref().and_then(Config::local_api_url),
    );
    (ApiMode::for_url(&base_url), base_url)
}

/// Resolve API mode, base URL, and auth credentials.
pub fn resolve_with_auth() -> (ApiMode, String, Auth) {
    let (mode, base_url) = resolve();
    let auth = auth::resolve_auth();
    (mode, base_url, auth)
}
