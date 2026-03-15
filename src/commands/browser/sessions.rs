use clap::Parser;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::list_sessions;
use crate::config::auth;
use crate::config::session_state::SessionStatePaths;
use crate::config::settings::{ApiMode, EnvVars};

#[derive(Parser)]
pub struct Args {
    /// Use local runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,

    /// Include full raw API payload for each session
    #[arg(long)]
    pub raw: bool,
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

    if args.raw {
        // Raw mode: print full API response
        let raw_sessions = client
            .list_sessions(&base_url, mode, &auth)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("{}", serde_json::to_string_pretty(&raw_sessions)?);
        return Ok(());
    }

    let sessions = list_sessions(&client, &base_url, mode, &auth, &paths).await?;

    let output: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| {
            let mut obj = serde_json::json!({
                "id": s.id,
                "mode": format!("{:?}", s.mode),
                "live": s.live,
            });
            if let Some(ref name) = s.name {
                obj["name"] = serde_json::json!(name);
            }
            if let Some(ref status) = s.status {
                obj["status"] = serde_json::json!(status);
            }
            if let Some(ref url) = s.viewer_url {
                obj["viewerUrl"] = serde_json::json!(url);
            }
            obj
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
