use clap::Parser;
use serde_json::json;

use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::process;
use crate::browser::daemon::protocol::{DaemonCommand, DaemonCreateParams, SessionInfo};
use crate::browser::lifecycle::sanitize_connect_url;
use crate::browser::profile_store;
use crate::util::{api, output};

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

pub async fn run(args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();

    let session_name = session.unwrap_or("default").to_string();

    // Resolve profile
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

    // If a daemon is already running for this session name, stop it first.
    // `start` always creates a fresh session — use `steel browser sessions`
    // to inspect existing ones.
    if DaemonClient::connect(&session_name).await.is_ok() {
        eprintln!("Replacing existing session \"{session_name}\"...");
        process::stop_daemon(&session_name).await?;
    } else {
        // No live daemon, but stale files may remain
        process::cleanup_stale(&session_name);
    }

    // Build params and spawn daemon
    let params = DaemonCreateParams {
        api_key: auth.api_key,
        base_url,
        mode,
        session_name: session_name.clone(),
        stealth: args.stealth,
        proxy_url: args.proxy,
        timeout_ms: args.session_timeout,
        headless: args.session_headless,
        region: args.session_region,
        solve_captcha: args.session_solve_captcha,
        profile_id: resolved_profile_id,
        persist_profile,
        namespace: args.namespace,
        credentials: args.credentials,
    };

    let child = process::spawn_daemon(&session_name, &params)?;
    process::wait_for_daemon(&session_name, child, std::time::Duration::from_secs(30)).await?;

    // Connect and get session info
    let mut client = DaemonClient::connect(&session_name).await?;
    let info = get_session_info(&mut client).await?;

    // Profile write-back using session_info.profile_id
    if let Some(ref profile_name) = args.profile
        && let Some(ref returned_profile_id) = info.profile_id
    {
        profile_store::write_profile(
            profile_name,
            returned_profile_id,
            stored_browser_profile.as_deref(),
            stored_browser.as_deref(),
        )?;
    }

    display_session_info(&info);

    Ok(())
}

async fn get_session_info(client: &mut DaemonClient) -> anyhow::Result<SessionInfo> {
    let data = client.send(DaemonCommand::GetSessionInfo).await?;
    let info: SessionInfo = serde_json::from_value(data)?;
    Ok(info)
}

fn display_session_info(info: &SessionInfo) {
    let remaining = remaining_time_str(info);

    if output::is_json() {
        let mut data = json!({
            "id": info.session_id,
            "mode": info.mode.to_string(),
        });
        data["name"] = json!(&info.session_name);
        if let Some(ref url) = info.viewer_url {
            data["liveUrl"] = json!(url);
        }
        if let Some(ref url) = info.connect_url {
            data["connectUrl"] = json!(sanitize_connect_url(url));
        }
        if let Some(ref rem) = remaining {
            data["remainingMs"] = json!(rem.0);
        }
        output::success_data(data);
    } else {
        println!("id: {}", info.session_id);
        println!("mode: {}", info.mode);
        println!("name: {}", info.session_name);
        if let Some(ref url) = info.viewer_url {
            println!("live_url: {url}");
        }
        if let Some(ref url) = info.connect_url {
            println!("connect_url: {}", sanitize_connect_url(url));
        }
        if let Some((ms, label)) = remaining {
            if ms < 120_000 {
                eprintln!("warning: session expires in {label}");
            } else {
                println!("expires_in: {label}");
            }
        }
    }
}

/// Compute (remaining_ms, human label) from session info, if timeout is set.
fn remaining_time_str(info: &SessionInfo) -> Option<(u64, String)> {
    let timeout = info.timeout_ms?;
    let created = info.created_at_ms?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    let expires_at = created.checked_add(timeout)?;
    if now >= expires_at {
        return Some((0, "expired".to_string()));
    }
    let remaining = expires_at - now;
    let secs = remaining / 1000;
    let label = if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    };
    Some((remaining, label))
}
