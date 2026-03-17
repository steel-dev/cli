use clap::Parser;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::list_sessions;
use crate::config::session_state::SessionStatePaths;
use crate::util::{api, output};

#[derive(Parser)]
pub struct Args {}


pub async fn run(_args: Args) -> anyhow::Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();

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
