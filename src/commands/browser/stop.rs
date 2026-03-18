use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::stop_session;
use crate::config::session_state::SessionStatePaths;
use crate::util::{api, output};

#[derive(Parser)]
pub struct Args {
    /// Stop all sessions
    #[arg(short, long)]
    pub all: bool,
}

pub async fn run(args: Args, session: Option<&str>) -> anyhow::Result<()> {
    if args.all && session.is_some() {
        anyhow::bail!("Cannot combine `--all` with `--session`.");
    }

    let (mode, base_url, auth) = api::resolve_with_auth();

    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    let result = stop_session(&client, &base_url, mode, &auth, &paths, session, args.all).await?;

    // Stop daemons for released sessions
    for id in &result.stopped_session_ids {
        let _ = crate::browser::daemon::process::stop_daemon(id).await;
    }

    if output::is_json() {
        output::success_data(json!({
            "stoppedSessionIds": result.stopped_session_ids,
            "mode": result.mode.to_string(),
        }));
    } else if result.stopped_session_ids.is_empty() {
        println!("No active browser sessions to stop.");
    } else if result.all {
        println!(
            "Stopped {} sessions in {} mode.",
            result.stopped_session_ids.len(),
            result.mode
        );
    } else {
        println!("Stopped session {}.", result.stopped_session_ids[0]);
    }

    Ok(())
}
