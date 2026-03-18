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

        if output::is_json() {
            output::success_data(json!({ "stoppedSessions": names }));
        } else {
            println!("Stopped {} sessions.", names.len());
        }
    } else {
        let session_name = session.unwrap_or("default");
        process::stop_daemon(session_name).await?;

        if output::is_json() {
            output::success_data(json!({ "stoppedSessions": [session_name] }));
        } else {
            println!("Stopped session \"{session_name}\".");
        }
    }

    Ok(())
}
