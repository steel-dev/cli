use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::get_live_url;
use crate::config::auth;
use crate::config::session_state::SessionStatePaths;
use crate::config::settings::{ApiMode, EnvVars};
use crate::util::output;

#[derive(Parser)]
pub struct Args {
    /// Named session
    #[arg(short, long)]
    pub session: Option<String>,

    /// Use local runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,
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

    let live_url = get_live_url(
        &client,
        &base_url,
        mode,
        &auth,
        &paths,
        args.session.as_deref(),
    )
    .await?;

    match live_url {
        Some(url) => {
            output::success(json!(url), &format!("{url}\n"));
        }
        None => {
            let msg = match args.session.as_deref() {
                Some(name) => format!(
                    "No live session found for \"{name}\". \
                     Start one with `steel browser start --session {name}`."
                ),
                None => "No active live session found. Start one with `steel browser start`.".to_string(),
            };
            return Err(output::error(&msg));
        }
    }

    Ok(())
}
