use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::state::{self, BrowserSession};
use crate::util::output;

#[derive(Parser)]
pub struct Args {
    /// Stop all sessions
    #[arg(short, long)]
    pub all: bool,
}

pub async fn release(client: &SteelClient, session: &BrowserSession) -> anyhow::Result<()> {
    match client
        .at(&session.api_url, session.mode)
        .cli_release_session(&session.id)
        .await
    {
        Ok(_) => {}
        Err(err) if err.is_not_found() => {}
        Err(err) => return Err(err.into()),
    }
    state::remove(&session.name)?;
    Ok(())
}

pub async fn run(args: Args, session: Option<&str>) -> anyhow::Result<()> {
    if args.all && session.is_some() {
        anyhow::bail!("Cannot combine `--all` with `--session`.");
    }

    let client = SteelClient::new()?;

    let stopped: Vec<String> = if args.all {
        let sessions = state::list();
        if sessions.is_empty() {
            if output::is_json() {
                output::success_data(json!({ "stoppedSessions": [] }));
            } else {
                println!("No active browser sessions to stop.");
            }
            return Ok(());
        }
        let mut stopped = Vec::new();
        for session in &sessions {
            match release(&client, session).await {
                Ok(()) => stopped.push(session.name.clone()),
                Err(err) => eprintln!("warning: could not stop \"{}\": {err}", session.name),
            }
        }
        stopped
    } else {
        let target = super::existing_session(session)?;
        release(&client, &target).await?;
        vec![target.name]
    };

    if output::is_json() {
        output::success_data(json!({ "stoppedSessions": stopped }));
    } else if let [name] = stopped.as_slice()
        && !args.all
    {
        println!("Stopped session \"{name}\".");
    } else {
        println!("Stopped {} sessions.", stopped.len());
    }

    let mut properties = serde_json::Map::new();
    properties.insert("all".into(), json!(args.all));
    properties.insert("stopped_count".into(), json!(stopped.len()));
    crate::telemetry::track_event("browser_session_stopped", properties);

    Ok(())
}
