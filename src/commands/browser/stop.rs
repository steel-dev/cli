use clap::Parser;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::stop_session;
use crate::config::auth;
use crate::config::session_state::SessionStatePaths;
use crate::config::settings::{ApiMode, EnvVars};

#[derive(Parser)]
pub struct Args {
    /// Named session to stop
    #[arg(short, long)]
    pub session: Option<String>,

    /// Stop all sessions
    #[arg(short, long)]
    pub all: bool,

    /// Use local runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    if args.all && args.session.is_some() {
        anyhow::bail!("Cannot combine `--all` with `--session`.");
    }

    let mode = ApiMode::resolve(args.local, args.api_url.as_deref());
    let auth = auth::resolve_auth();
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
    let local_config_url = config.as_ref().and_then(|c| c.local_api_url());
    let base_url = mode.resolve_base_url(args.api_url.as_deref(), &env_vars, local_config_url);

    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    let result = stop_session(
        &client,
        &base_url,
        mode,
        &auth,
        &paths,
        args.session.as_deref(),
        args.all,
    )
    .await?;

    if result.stopped_session_ids.is_empty() {
        println!("No active browser sessions to stop.");
    } else if result.all {
        println!(
            "Stopped {} sessions in {:?} mode.",
            result.stopped_session_ids.len(),
            result.mode
        );
    } else {
        println!("Stopped session {}.", result.stopped_session_ids[0]);
    }

    Ok(())
}
