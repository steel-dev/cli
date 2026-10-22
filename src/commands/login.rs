use crate::config::{auth, device};
use crate::util::output;
use clap::Parser;
use std::io::IsTerminal;

#[derive(Parser, Default)]
pub struct Args {
    /// Continue the saved authorization request; do not issue a new code
    #[arg(long)]
    pub resume: bool,
    /// Show the handoff without waiting; check an existing request once if due
    #[arg(long)]
    pub no_wait: bool,
    /// Do not open a browser on this host
    #[arg(long)]
    pub no_browser: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let existing = auth::resolve_auth();
    if existing.api_key.is_some() {
        output::success_data(serde_json::json!({"status":"authenticated","method":"api_key"}));
        return Ok(());
    }
    let interactive = std::io::stdout().is_terminal();
    let complete = device::login(
        args.no_wait || !interactive,
        args.no_browser || !interactive,
        args.resume,
    )
    .await?;
    if complete {
        let settings = device::saved_settings()?.expect("saved login");
        device::access_token_for(&settings.api_base_url).await?;
        if existing.access_token.is_none() {
            crate::telemetry::track_event("login_completed", serde_json::Map::new());
        }
    }
    output::success_data(serde_json::json!({
        "status":if complete {"authenticated"} else {"pending"},
        "method":"device",
        "next_action":if complete {"Run steel doctor --preflight to verify product API access"} else {"Approve the displayed code, then run steel login --resume or repeat steel init"}
    }));
    Ok(())
}
