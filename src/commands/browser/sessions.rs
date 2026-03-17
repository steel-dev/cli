use clap::Parser;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::list_sessions;
use crate::config::auth;
use crate::config::session_state::SessionStatePaths;
use crate::config::settings::{ApiMode, EnvVars};
use crate::util::output;

#[derive(Parser)]
pub struct Args {
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

    let sessions = list_sessions(&client, &base_url, mode, &auth, &paths).await?;

    let data: Vec<serde_json::Value> = sessions
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

    output::success_data(serde_json::json!(data));

    Ok(())
}
