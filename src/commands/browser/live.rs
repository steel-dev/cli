use clap::Parser;
use serde_json::json;

use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::protocol::{DaemonCommand, SessionInfo};
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let session_name = session.unwrap_or("default");

    let mut client = DaemonClient::connect(session_name).await.map_err(|_| {
        let msg = match session {
            Some(name) => format!(
                "No running session \"{name}\". \
                 Start one with `steel browser start --session {name}`."
            ),
            None => "No active browser session. Start one with `steel browser start`.".to_string(),
        };
        anyhow::anyhow!("{msg}")
    })?;

    let data = client.send(DaemonCommand::GetSessionInfo).await?;
    let info: SessionInfo = serde_json::from_value(data)?;

    match info.viewer_url {
        Some(url) => {
            output::success(json!(url), &format!("{url}\n"));
        }
        None => {
            anyhow::bail!(
                "Session \"{session_name}\" has no live viewer URL."
            );
        }
    }

    Ok(())
}
