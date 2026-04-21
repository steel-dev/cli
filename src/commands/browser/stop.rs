use clap::Parser;
use serde_json::json;

use crate::browser::daemon::process;
use crate::util::output;

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

    let stopped_count;
    if args.all {
        let names = process::list_daemon_names();
        if names.is_empty() {
            if output::is_json() {
                output::success_data(json!({ "stoppedSessions": [] }));
            } else {
                println!("No active browser sessions to stop.");
            }
            return Ok(());
        }

        for name in &names {
            let _ = process::stop_daemon(name).await;
        }
        stopped_count = names.len() as u64;

        if output::is_json() {
            output::success_data(json!({ "stoppedSessions": names }));
        } else {
            println!("Stopped {} sessions.", names.len());
        }
    } else {
        let session_name = session.unwrap_or("default");
        process::stop_daemon(session_name).await?;
        stopped_count = 1;

        if output::is_json() {
            output::success_data(json!({ "stoppedSessions": [session_name] }));
        } else {
            println!("Stopped session \"{session_name}\".");
        }
    }

    let mut properties = serde_json::Map::new();
    properties.insert("all".into(), json!(args.all));
    properties.insert("stopped_count".into(), json!(stopped_count));
    crate::telemetry::track_event("browser_session_stopped", properties);

    Ok(())
}
