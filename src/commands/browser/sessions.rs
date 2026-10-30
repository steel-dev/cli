use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::is_session_live;
use crate::browser::state::{self, BrowserSession};
use crate::config::auth;
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let client = SteelClient::new()?;
    let auth = auth::resolve_auth();

    let mut sessions: Vec<(BrowserSession, String)> = Vec::new();
    for session in state::list() {
        match client
            .get_session(&session.api_url, session.mode, &session.id, &auth)
            .await
        {
            Ok(remote) if is_session_live(&remote) => {
                let status = remote["status"].as_str().unwrap_or("live").to_string();
                sessions.push((session, status));
            }
            Ok(_) => {
                state::remove(&session.name)?;
            }
            Err(err) if err.is_not_found() => {
                state::remove(&session.name)?;
            }
            Err(_) => sessions.push((session, "unknown".to_string())),
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
