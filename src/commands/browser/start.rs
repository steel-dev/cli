use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::api::session::CreateSessionOptions;
use crate::browser::lifecycle::{
    get_session_created_at_ms, get_session_inactivity_timeout, get_session_timeout,
    sanitize_connect_url, to_session_summary,
};
use crate::browser::profile_store;
use crate::browser::state::{self, BrowserSession};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::status;
use crate::util::{api, output};

pub const DEFAULT_INACTIVITY_TIMEOUT_MS: u64 = 600_000;

#[derive(Parser)]
pub struct Args {
    /// Enable stealth mode (humanize interactions + auto CAPTCHA)
    #[arg(long)]
    pub stealth: bool,

    /// Use a residential proxy
    #[arg(short = 'p', long)]
    pub proxy: Option<String>,

    /// Session timeout in milliseconds (create-time only)
    #[arg(long = "session-timeout")]
    pub session_timeout: Option<u64>,

    /// Inactivity timeout in milliseconds: release the session when no command, CDP message, or remote input arrives for this long. Defaults to 600000 (10 minutes); pass 0 to disable (create-time only)
    #[arg(long = "inactivity-timeout")]
    pub inactivity_timeout: Option<u64>,

    /// Create new sessions in headless mode (create-time only)
    #[arg(long = "session-headless", hide = true)]
    pub session_headless: Option<bool>,

    /// Preferred session region (create-time only)
    #[arg(long = "session-region", hide = true)]
    pub session_region: Option<String>,

    /// Enable manual CAPTCHA solving on new sessions (create-time only)
    #[arg(long = "session-solve-captcha")]
    pub session_solve_captcha: bool,

    /// Named profile to persist browser state across sessions
    #[arg(long)]
    pub profile: Option<String>,

    /// Save session state back to the profile when the session ends
    #[arg(long = "update-profile")]
    pub update_profile: bool,

    /// Credential namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Inject credentials
    #[arg(long)]
    pub credentials: bool,
}

pub fn default_options() -> CreateSessionOptions {
    CreateSessionOptions {
        inactivity_timeout_ms: Some(DEFAULT_INACTIVITY_TIMEOUT_MS),
        ..Default::default()
    }
}

fn now_ms() -> Option<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

pub async fn create(
    client: &SteelClient,
    mode: ApiMode,
    base_url: &str,
    auth: &Auth,
    name: &str,
    options: &CreateSessionOptions,
) -> anyhow::Result<(BrowserSession, Option<String>)> {
    let session = client.create_session(base_url, mode, options, auth).await?;
    let summary = to_session_summary(&session, mode, auth)?;
    let mut created = BrowserSession::new(
        name.to_string(),
        summary.id.clone(),
        mode,
        base_url.to_string(),
    );
    created.live_url = summary.viewer_url;
    created.profile_id = summary.profile_id;
    created.timeout_ms = get_session_timeout(&session).or(options.timeout_ms);
    created.inactivity_timeout_ms =
        get_session_inactivity_timeout(&session).or(options.inactivity_timeout_ms);
    created.created_at_ms = get_session_created_at_ms(&session).or_else(now_ms);
    state::save(&created)?;
    Ok((created, summary.connect_url))
}

pub async fn run(args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();

    let session_name = session.unwrap_or("default").to_string();
    if state::looks_like_session_id(&session_name) {
        anyhow::bail!(
            "Session names cannot look like session IDs. Use `--session {session_name}` on other commands to drive that session, or pick a name."
        );
    }

    let mut resolved_profile_id = None;
    let mut stored_browser_profile = None;
    let mut stored_browser = None;
    if let Some(ref profile_name) = args.profile {
        if let Some(err) = profile_store::validate_profile_name(profile_name) {
            anyhow::bail!("{err}");
        }
        if let Some(data) = profile_store::read_profile(profile_name)? {
            resolved_profile_id = Some(data.profile_id);
            stored_browser_profile = data.browser_profile;
            stored_browser = data.browser;
        }
    }

    let persist_profile = args.profile.is_some() && args.update_profile;
    let proxy_enabled = args.proxy.is_some();
    let namespace_set = args.namespace.is_some();
    let inactivity_timeout_ms = resolve_inactivity_timeout(args.inactivity_timeout);
    if let (Some(timeout), Some(inactivity)) = (args.session_timeout, inactivity_timeout_ms)
        && inactivity >= timeout
    {
        eprintln!(
            "warning: --inactivity-timeout ({inactivity}ms) >= --session-timeout ({timeout}ms); inactivity timeout has no effect because the session timeout elapses first"
        );
    }

    let client = SteelClient::new()?;

    if let Some(existing) = state::load(&session_name)? {
        status!("Replacing existing session \"{session_name}\"...");
        if let Err(err) = super::stop::release(&client, &auth, &existing).await {
            eprintln!("warning: could not release session {}: {err}", existing.id);
            state::remove(&session_name)?;
        }
    }

    let options = CreateSessionOptions {
        stealth: args.stealth,
        proxy_url: args.proxy,
        timeout_ms: args.session_timeout,
        inactivity_timeout_ms,
        headless: args.session_headless,
        region: args.session_region,
        solve_captcha: args.session_solve_captcha,
        profile_id: resolved_profile_id,
        persist_profile,
        namespace: args.namespace,
        credentials: args.credentials,
    };

    let (created, connect_url) =
        create(&client, mode, &base_url, &auth, &session_name, &options).await?;

    if let Some(ref profile_name) = args.profile
        && let Some(ref returned_profile_id) = created.profile_id
    {
        profile_store::write_profile(
            profile_name,
            returned_profile_id,
            stored_browser_profile.as_deref(),
            stored_browser.as_deref(),
        )?;
    }

    let mut properties = serde_json::Map::new();
    properties.insert("stealth".into(), json!(args.stealth));
    properties.insert("proxy_enabled".into(), json!(proxy_enabled));
    properties.insert("has_profile".into(), json!(args.profile.is_some()));
    properties.insert("persist_profile".into(), json!(persist_profile));
    properties.insert("credentials".into(), json!(args.credentials));
    properties.insert("namespace_set".into(), json!(namespace_set));
    properties.insert(
        "session_timeout_set".into(),
        json!(args.session_timeout.is_some()),
    );
    properties.insert("inactivity_timeout_ms".into(), json!(inactivity_timeout_ms));
    crate::telemetry::track_event("browser_session_started", properties);

    display_session_info(&created, connect_url.as_deref());

    Ok(())
}

fn display_session_info(info: &BrowserSession, connect_url: Option<&str>) {
    let remaining = remaining_time_str(info);

    if output::is_json() {
        let mut data = json!({
            "id": info.id,
            "mode": info.mode.to_string(),
        });
        data["name"] = json!(&info.name);
        if let Some(ref url) = info.live_url {
            data["liveUrl"] = json!(url);
        }
        if let Some(url) = connect_url {
            data["connectUrl"] = json!(sanitize_connect_url(url));
        }
        if let Some(ref rem) = remaining {
            data["remainingMs"] = json!(rem.0);
        }
        if let Some(ms) = info.inactivity_timeout_ms {
            data["inactivityTimeoutMs"] = json!(ms);
        }
        output::success_data(data);
    } else {
        println!("id: {}", info.id);
        println!("mode: {}", info.mode);
        println!("name: {}", info.name);
        if let Some(ref url) = info.live_url {
            println!("live_url: {url}");
        }
        if let Some(url) = connect_url {
            println!("connect_url: {}", sanitize_connect_url(url));
        }
        if let Some((ms, label)) = remaining {
            if ms < 120_000 {
                eprintln!("warning: session expires in {label}");
            } else {
                println!("expires_in: {label}");
            }
        }
        if let Some(ms) = info.inactivity_timeout_ms {
            println!("inactivity_timeout: {}", humanize_secs(ms / 1000));
        }
    }
}

fn remaining_time_str(info: &BrowserSession) -> Option<(u64, String)> {
    let timeout = info.timeout_ms?;
    let created = info.created_at_ms?;
    let now = now_ms()?;
    let expires_at = created.checked_add(timeout)?;
    if now >= expires_at {
        return Some((0, "expired".to_string()));
    }
    let remaining = expires_at - now;
    Some((remaining, humanize_secs(remaining / 1000)))
}

fn humanize_secs(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

const fn resolve_inactivity_timeout(explicit: Option<u64>) -> Option<u64> {
    match explicit {
        None => Some(DEFAULT_INACTIVITY_TIMEOUT_MS),
        Some(0) => None,
        Some(ms) => Some(ms),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactivity_default_when_unset() {
        assert_eq!(
            resolve_inactivity_timeout(None),
            Some(DEFAULT_INACTIVITY_TIMEOUT_MS)
        );
        assert_eq!(DEFAULT_INACTIVITY_TIMEOUT_MS, 600_000);
    }

    #[test]
    fn inactivity_explicit_value_wins() {
        assert_eq!(resolve_inactivity_timeout(Some(45000)), Some(45000));
    }

    #[test]
    fn inactivity_zero_disables() {
        assert_eq!(resolve_inactivity_timeout(Some(0)), None);
    }

    #[test]
    fn default_options_set_only_the_inactivity_timeout() {
        let options = default_options();
        assert_eq!(options.inactivity_timeout_ms, Some(600_000));
        assert_eq!(options.timeout_ms, None);
        assert!(!options.stealth);
    }

    #[test]
    fn humanize_secs_formats() {
        assert_eq!(humanize_secs(45), "45s");
        assert_eq!(humanize_secs(120), "2m 0s");
        assert_eq!(humanize_secs(3661), "1h 1m");
    }
}
