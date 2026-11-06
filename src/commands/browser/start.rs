use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::api::session::CreateSessionOptions;
use crate::browser::profile_store;
use crate::browser::state::{self, BrowserSession};
use crate::util::{metadata, output};

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
    #[arg(long, alias = "session-timeout")]
    pub timeout: Option<u64>,

    /// Inactivity timeout in milliseconds: release the session when no command, CDP message, or remote input arrives for this long. Defaults to 600000 (10 minutes); pass 0 to disable (create-time only)
    #[arg(long = "inactivity-timeout")]
    pub inactivity_timeout: Option<u64>,

    /// Create new sessions in headless mode (create-time only)
    #[arg(long, alias = "session-headless", hide = true)]
    pub headless: Option<bool>,

    /// Preferred session region (create-time only)
    #[arg(long, alias = "session-region", hide = true)]
    pub region: Option<String>,

    /// Enable manual CAPTCHA solving on new sessions (create-time only)
    #[arg(long, alias = "session-solve-captcha")]
    pub solve_captcha: bool,

    /// Named profile to load (see `steel profile list`)
    #[arg(long)]
    pub profile: Option<String>,

    /// Save session state back to the profile when the session ends; creates the profile if it does not exist
    #[arg(long = "update-profile", requires = "profile")]
    pub update_profile: bool,

    /// Credential namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Inject credentials
    #[arg(long)]
    pub credentials: bool,

    /// Attach metadata as KEY=VALUE, repeatable (Steel Cloud only)
    #[arg(long, value_name = "KEY=VALUE", value_parser = metadata::parse_pair)]
    pub metadata: Vec<(String, String)>,
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
    name: &str,
    options: &CreateSessionOptions,
) -> anyhow::Result<(BrowserSession, Option<String>)> {
    let session = client.create_session(options).await?;
    let mode = client.mode();
    let mut created = BrowserSession::new(
        name.to_string(),
        session.id.clone(),
        mode,
        client.base_url().to_string(),
    );
    created.live_url = session.viewer_url(mode);
    created.profile_id = session.profile_id.clone();
    created.timeout_ms = session.timeout_ms().or(options.timeout_ms);
    created.inactivity_timeout_ms = session
        .inactivity_timeout_ms()
        .or(options.inactivity_timeout_ms);
    created.created_at_ms = session.created_at_ms().or_else(now_ms);
    state::save(&created)?;
    Ok((created, session.connect_url(mode, client.auth())))
}

pub async fn run(args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let session_name = session.unwrap_or("default").to_string();
    if state::looks_like_session_id(&session_name) {
        anyhow::bail!(
            "Session names cannot look like session IDs. Use `--session {session_name}` on other commands to drive that session, or pick a name."
        );
    }
    let session_metadata = metadata::collect(args.metadata)?;

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
        } else if !args.update_profile {
            anyhow::bail!(
                "Profile \"{profile_name}\" not found. Import one with `steel profile import {profile_name}`, or add --update-profile to create it from this session."
            );
        }
    }

    let persist_profile = args.profile.is_some() && args.update_profile;
    let proxy_enabled = args.proxy.is_some();
    let namespace_set = args.namespace.is_some();
    let metadata_set = !session_metadata.is_empty();
    let inactivity_timeout_ms = resolve_inactivity_timeout(args.inactivity_timeout);
    if let (Some(timeout), Some(inactivity)) = (args.timeout, inactivity_timeout_ms)
        && inactivity >= timeout
    {
        eprintln!(
            "warning: --inactivity-timeout ({inactivity}ms) >= --timeout ({timeout}ms); inactivity timeout has no effect because the session timeout elapses first"
        );
    }

    let client = SteelClient::new()?;

    let replaced = match state::load(&session_name)? {
        Some(existing) => {
            eprintln!(
                "note: replacing session \"{session_name}\" ({})",
                existing.id
            );
            let released = match super::stop::release(&client, &existing).await {
                Ok(()) => true,
                Err(err) => {
                    eprintln!("warning: could not release session {}: {err}", existing.id);
                    state::remove(&session_name)?;
                    false
                }
            };
            Some((existing.id, released))
        }
        None => None,
    };

    let options = CreateSessionOptions {
        stealth: args.stealth,
        proxy_url: args.proxy,
        timeout_ms: args.timeout,
        inactivity_timeout_ms,
        headless: args.headless,
        region: args.region,
        solve_captcha: args.solve_captcha,
        profile_id: resolved_profile_id,
        persist_profile,
        namespace: args.namespace,
        credentials: args.credentials,
        metadata: session_metadata,
    };

    let (created, connect_url) = create(&client, &session_name, &options).await?;

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
    properties.insert("metadata_set".into(), json!(metadata_set));
    properties.insert("session_timeout_set".into(), json!(args.timeout.is_some()));
    properties.insert("inactivity_timeout_ms".into(), json!(inactivity_timeout_ms));
    crate::telemetry::track_event("browser_session_started", properties);

    display_session_info(&created, connect_url.as_deref(), replaced.as_ref());

    Ok(())
}

fn display_session_info(
    info: &BrowserSession,
    connect_url: Option<&str>,
    replaced: Option<&(String, bool)>,
) {
    let remaining = remaining_time_str(info);

    if output::is_json() {
        let mut data = json!({
            "id": info.id,
            "mode": info.mode.to_string(),
        });
        data["name"] = json!(&info.name);
        if let Some((id, released)) = replaced {
            data["replacedSessionId"] = json!(id);
            data["replacedSessionReleased"] = json!(released);
        }
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

/// Query parameters known to be safe to display; every other value is redacted.
const DISPLAYED_CONNECT_PARAMS: &[&str] = &["sessionId"];

/// Connection URLs are display metadata, not a credential export. Only allowlisted
/// query values are shown; userinfo and fragments are dropped. The URL the API
/// returned is never altered for transport.
fn sanitize_connect_url(url: &str) -> String {
    let Ok(mut parsed) = url::Url::parse(url) else {
        return "[redacted connection URL]".to_string();
    };
    let _ = parsed.set_username("");
    let _ = parsed.set_password(None);
    parsed.set_fragment(None);
    let pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .map(|(k, v)| {
            let shown = DISPLAYED_CONNECT_PARAMS.contains(&k.as_ref());
            (
                k.to_string(),
                if shown {
                    v.to_string()
                } else {
                    "[redacted]".to_string()
                },
            )
        })
        .collect();
    if !pairs.is_empty() {
        parsed.query_pairs_mut().clear().extend_pairs(&pairs);
    }
    parsed.to_string()
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
    fn sanitize_redacts_every_connection_credential() {
        let masked =
            sanitize_connect_url("wss://connect.steel.dev?apiKey=sk-12345678&sessionId=s1");
        assert!(!masked.contains("sk-1234"));
        assert!(masked.contains("sessionId=s1"));
        let short = "wss://connect.steel.dev?apiKey=short&sessionId=s1";
        assert!(!sanitize_connect_url(short).contains("short"));
        let token = sanitize_connect_url(
            "wss://user:pass@connect.steel.dev?sessionId=s1&token=synthetic-connect-token",
        );
        assert!(!token.contains("synthetic-connect-token"));
        assert!(!token.contains("user") && !token.contains("pass"));
        let local = "ws://localhost:3000/ws?sessionId=sess-1";
        assert_eq!(sanitize_connect_url(local), local);
        assert_eq!(
            sanitize_connect_url("not a valid url?token=secret"),
            "[redacted connection URL]"
        );
        let signed = sanitize_connect_url(
            "wss://connect.steel.dev/s?sessionId=s1&sig=a1&signature=a2&X-Amz-Signature=a3&authorization=a4&capability=a5#frag-secret",
        );
        assert_eq!(
            signed,
            "wss://connect.steel.dev/s?sessionId=s1&sig=%5Bredacted%5D&signature=%5Bredacted%5D&X-Amz-Signature=%5Bredacted%5D&authorization=%5Bredacted%5D&capability=%5Bredacted%5D"
        );
    }

    #[test]
    fn humanize_secs_formats() {
        assert_eq!(humanize_secs(45), "45s");
        assert_eq!(humanize_secs(120), "2m 0s");
        assert_eq!(humanize_secs(3661), "1h 1m");
    }
}
