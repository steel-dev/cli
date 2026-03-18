use clap::Parser;

use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::process;
use crate::browser::daemon::protocol::{DaemonCommand, SessionInfo};
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let names = process::list_daemon_names();

    let mut sessions: Vec<SessionInfo> = Vec::new();
    for name in &names {
        // Fast path: skip sockets whose daemon process is already dead
        if process::cleanup_if_dead(name) {
            continue;
        }
        match DaemonClient::connect(name).await {
            Ok(mut client) => {
                if let Ok(data) = client.send(DaemonCommand::GetSessionInfo).await
                    && let Ok(info) = serde_json::from_value::<SessionInfo>(data) {
                        sessions.push(info);
                    }
            }
            Err(_) => {
                // Socket exists but connection failed — clean up
                process::cleanup_stale(name);
            }
        }
    }

    let data: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| {
            let mut obj = serde_json::json!({
                "id": s.session_id,
                "name": s.session_name,
                "mode": s.mode.to_string(),
            });
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
            .map(|s| s.session_id.len())
            .max()
            .unwrap_or(2)
            .max(2);
        let max_name = sessions
            .iter()
            .map(|s| s.session_name.len())
            .max()
            .unwrap_or(4)
            .max(4);
        println!(
            "{:<max_id$}  {:<max_name$}  {:<6}  STATUS",
            "ID", "NAME", "MODE"
        );
        for s in &sessions {
            let status = s.status.as_deref().unwrap_or("live");
            println!(
                "{:<max_id$}  {:<max_name$}  {:<6}  {status}",
                s.session_id, s.session_name, s.mode
            );
        }
    }

    Ok(())
}
