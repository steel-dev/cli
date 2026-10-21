use crate::config::{auth, device};
use crate::util::output;
use clap::Parser;

#[derive(Parser, Default)]
pub struct Args {
    /// Continue the saved device request; do not issue a new code
    #[arg(long)]
    pub resume: bool,
    /// Return the handoff without waiting; check an existing pending request once if due
    #[arg(long)]
    pub no_wait: bool,
    /// Do not launch a browser on this host (remote/agent handoff)
    #[arg(long)]
    pub no_browser: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let auth = auth::resolve_auth();
    if auth.api_key.is_some() {
        output::success_data(
            serde_json::json!({"status":"authenticated", "method":"api_key", "next_action":"An API key is already configured. Unset STEEL_API_KEY / log out to use device login."}),
        );
        return Ok(());
    }
    let complete = device::login(args.no_wait, args.no_browser, args.resume).await?;
    if complete {
        let settings = device::saved_settings()?.expect("login settings");
        // Refresh if necessary before calling this authenticated. Product API
        // acceptance is checked separately by doctor, not inferred from tokens.
        device::access_token_for(&settings.api_base_url).await?;
        if auth.access_token.is_none() {
            crate::telemetry::track_event("login_completed", serde_json::Map::new());
        }
    }
    output::success_data(serde_json::json!({
        "status": if complete { "authenticated" } else { "pending" },
        "method": "device", "product_api_verified": false,
        "next_action": if complete { "Run steel doctor --preflight to verify product API authorization" } else { "Approve the displayed code in your browser, then run steel login --resume (or rerun steel init --agent)" },
    }));
    Ok(())
}
