use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::api::session::Session;
use crate::browser::state::{self, BrowserSession};
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let client = SteelClient::new()?;

    let mut sessions: Vec<(BrowserSession, String)> = Vec::new();
    for session in state::list() {
        match client
            .at(&session.api_url, session.mode)
            .cli_get_session(&session.id)
            .await
            .map(Session::parse)
        {
            Ok(Ok(remote)) if remote.is_live() => {
                let status = remote.status.unwrap_or_else(|| "live".to_string());
                sessions.push((session, status));
            }
            Ok(Ok(_)) => {
                state::remove(&session.name)?;
            }
            Err(err) if err.is_not_found() => {
                state::remove(&session.name)?;
            }
            Ok(Err(_)) | Err(_) => sessions.push((session, "unknown".to_string())),
        }
    }

    if output::is_json() {
        let data: Vec<serde_json::Value> = sessions
            .iter()
            .map(|(s, status)| {
                let mut obj = json!({
                    "id": s.id,
                    "name": s.name,
                    "mode": s.mode.to_string(),
                    "status": status,
                });
                if let Some(ref url) = s.live_url {
                    obj["viewerUrl"] = json!(url);
                }
                obj
            })
            .collect();
        output::success_data(json!(data));
    } else if sessions.is_empty() {
        println!("No active browser sessions.");
    } else {
        let rows: Vec<Vec<String>> = sessions
            .iter()
            .map(|(s, status)| {
                vec![
                    s.id.clone(),
                    s.name.clone(),
                    s.mode.to_string(),
                    status.clone(),
                ]
            })
            .collect();
        output::print_table(&["ID", "NAME", "MODE", "STATUS"], &rows);
    }

    Ok(())
}
