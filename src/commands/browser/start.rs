use clap::Parser;

use serde_json::json;

use crate::api::client::SteelClient;
use crate::api::session::CreateSessionOptions;
use crate::browser::lifecycle::{sanitize_connect_url, start_session};
use crate::browser::profile_store;
use crate::config::session_state::SessionStatePaths;
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

    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    // Resolve profile
    let mut resolved_profile_id = None;
    let mut stored_chrome_profile = None;
    let mut stored_browser = None;
    if let Some(ref profile_name) = args.profile {
        if let Some(err) = profile_store::validate_profile_name(profile_name) {
            anyhow::bail!("{err}");
        }
        if let Some(data) = profile_store::read_profile(profile_name)? {
            resolved_profile_id = Some(data.profile_id);
            stored_chrome_profile = data.chrome_profile;
            stored_browser = data.browser;
        }
    }

    let persist_profile = args.profile.is_some() && args.update_profile;

    let options = CreateSessionOptions {
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

    let session = start_session(&client, &base_url, mode, &auth, &paths, session, &options).await?;

    // Write profile mapping back if a profile was specified
    if let Some(ref profile_name) = args.profile {
        if let Some(ref returned_profile_id) = session.profile_id {
            profile_store::write_profile(
                profile_name,
                returned_profile_id,
                stored_chrome_profile.as_deref(),
                stored_browser.as_deref(),
            )?;
        }
    }

    // Eagerly spawn daemon so subsequent commands are fast
    if let Some(ref url) = session.connect_url {
        use crate::browser::daemon::process;
        if !process::socket_path(&session.id).exists() {
            if let Err(e) = process::spawn_daemon(&session.id, url) {
                eprintln!("Warning: failed to start browser daemon: {e}");
            } else if let Err(e) =
                process::wait_for_daemon(&session.id, std::time::Duration::from_secs(10)).await
            {
                eprintln!("Warning: browser daemon did not become ready: {e}");
            }
        }
    }

    if output::is_json() {
        let mut data = json!({
            "id": session.id,
            "mode": session.mode.to_string(),
        });
        if let Some(ref name) = session.name {
            data["name"] = json!(name);
        }
        if let Some(ref url) = session.viewer_url {
            data["liveUrl"] = json!(url);
        }
        if let Some(ref url) = session.connect_url {
            data["connectUrl"] = json!(sanitize_connect_url(url));
        }
        output::success_data(data);
    } else {
        println!("id: {}", session.id);
        println!("mode: {}", session.mode);
        if let Some(ref name) = session.name {
            println!("name: {name}");
        }
        if let Some(ref url) = session.viewer_url {
            println!("live_url: {url}");
        }
        if let Some(ref url) = session.connect_url {
            println!("connect_url: {}", sanitize_connect_url(url));
        }
    }

    Ok(())
}
