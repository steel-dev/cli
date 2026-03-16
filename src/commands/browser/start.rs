use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::session::CreateSessionOptions;
use crate::browser::lifecycle::{sanitize_connect_url, start_session};
use crate::browser::profile_store;
use crate::config::auth;
use crate::config::session_state::SessionStatePaths;
use crate::config::settings::{ApiMode, EnvVars};

#[derive(Parser)]
pub struct Args {
    /// Named session for reuse
    #[arg(short, long)]
    pub session: Option<String>,

    /// Use local runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,

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
    #[arg(long = "session-headless")]
    pub session_headless: Option<bool>,

    /// Preferred session region (create-time only)
    #[arg(long = "session-region")]
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

pub async fn run(args: Args) -> anyhow::Result<()> {
    let mode = ApiMode::resolve(args.local, args.api_url.as_deref());
    let auth = auth::resolve_auth();
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
    let local_config_url = config.as_ref().and_then(|c| c.local_api_url());
    let base_url = mode.resolve_base_url(args.api_url.as_deref(), &env_vars, local_config_url);

    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    // Resolve profile
    let mut resolved_profile_id = None;
    let mut stored_chrome_profile = None;
    if let Some(ref profile_name) = args.profile {
        if let Some(err) = profile_store::validate_profile_name(profile_name) {
            anyhow::bail!("{err}");
        }
        if let Some(data) = profile_store::read_profile(profile_name)? {
            resolved_profile_id = Some(data.profile_id);
            stored_chrome_profile = data.chrome_profile;
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

    let session = start_session(
        &client,
        &base_url,
        mode,
        &auth,
        &paths,
        args.session.as_deref(),
        &options,
    )
    .await?;

    // Write profile mapping back if a profile was specified
    if let Some(ref profile_name) = args.profile {
        if let Some(ref returned_profile_id) = session.profile_id {
            profile_store::write_profile(
                profile_name,
                returned_profile_id,
                stored_chrome_profile.as_deref(),
            )?;
        }
    }

    println!("id: {}", session.id);
    println!("mode: {:?}", session.mode);
    if let Some(ref name) = session.name {
        println!("name: {name}");
    }
    if let Some(ref url) = session.viewer_url {
        println!("live_url: {url}");
    }
    if let Some(ref url) = session.connect_url {
        println!("connect_url: {}", sanitize_connect_url(url));

        // Eagerly spawn daemon so subsequent commands are fast
        use crate::browser::daemon::process;
        if !process::socket_path(&session.id).exists() {
            if let Err(e) = process::spawn_daemon(&session.id, url) {
                eprintln!("Warning: failed to start browser daemon: {e}");
            } else if let Err(e) = process::wait_for_daemon(
                &session.id,
                std::time::Duration::from_secs(10),
            ).await {
                eprintln!("Warning: browser daemon did not become ready: {e}");
            }
        }
    }

    Ok(())
}
