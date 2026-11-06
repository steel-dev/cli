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
    // Device tokens are bound to the API transport they were issued for, so an
    // unconfigured invocation targets that transport rather than the default.
    let explicit = api_url().is_some_and(|s| !s.trim().is_empty())
        || is_local()
        || env_api_url.as_deref().is_some_and(|s| !s.trim().is_empty());
    if !explicit
        && auth::resolve_auth().access_token.is_some()
        && let Ok(Some(settings)) = crate::config::device::saved_settings()
    {
        return (
            ApiMode::for_url(&settings.api_base_url),
            settings.api_base_url,
        );
    }
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
    let auth = auth_for_target(auth::resolve_auth(), &base_url);
    (mode, base_url, auth)
}

/// The local runtime `--local` targets: `browser.apiUrl` from config, or the default.
fn local_runtime_url() -> String {
    let config = crate::config::settings::read_config().ok();
    resolve_base_url(
        None,
        true,
        None,
        config.as_ref().and_then(Config::local_api_url),
    )
}

/// Credentials to present to `base_url`.
///
/// A device login is bound to the API transport it was issued for and is refused
/// anywhere else. The configured local runtime is the one deliberate exception: it
/// is a separate target that never received the device login, so it gets the
/// API-key credentials it had before device login instead of a refusal. A device
/// login issued for that same local URL keeps its Bearer token.
pub fn auth_for_target(auth: Auth, base_url: &str) -> Auth {
    if auth.access_token.is_none() || !same_url(base_url, &local_runtime_url()) {
        return auth;
    }
    match crate::config::device::saved_settings() {
        Ok(Some(settings)) if same_url(&settings.api_base_url, base_url) => auth,
        _ => auth::resolve_api_key_auth(),
    }
}

fn same_url(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}
