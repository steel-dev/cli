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
                "mode": s.mode.to_string(),
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

    if output::is_json() {
        output::success_data(serde_json::json!(data));
    } else if sessions.is_empty() {
        println!("No active browser sessions.");
    } else {
        // Human-readable table
        let max_id = sessions
            .iter()
            .map(|s| s.id.len())
            .max()
            .unwrap_or(2)
            .max(2);
        let max_name = sessions
            .iter()
            .filter_map(|s| s.name.as_ref())
            .map(|n| n.len())
            .max()
            .unwrap_or(4)
            .max(4);
        println!(
            "{:<max_id$}  {:<max_name$}  {:<6}  STATUS",
            "ID", "NAME", "MODE"
        );
        for s in &sessions {
            let name = s.name.as_deref().unwrap_or("-");
            let status = s
                .status
                .as_deref()
                .unwrap_or(if s.live { "live" } else { "-" });
            println!(
                "{:<max_id$}  {:<max_name$}  {:<6}  {status}",
                s.id, name, s.mode
            );
        }
    }

    Ok(())
}
