pub mod agents;

use clap::Parser;

use crate::commands::{doctor, login};
use crate::status;

#[derive(Parser)]
pub struct Args {
    /// Run in agent mode: auto-accept interactive prompts and print
    /// agent-friendly output. Designed for AI coding agents
    #[arg(long)]
    pub agent: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    status!("Steel CLI setup");
    if let Ok(from) = std::env::var("STEEL_ONBOARDING_FROM")
        && !from.is_empty()
    {
        status!("Onboarding source: {from}");
    }
    status!("");

    // Step 1: login (no-ops if already logged in).
    login::run(login::Args {}).await?;

    // Step 2: preflight check.
    status!("");
    doctor::run(doctor::Args { preflight: true }).await?;

    // Step 3: auto-install the Steel skill into any detected coding agents.
    status!("");
    agents::install_skills(args.agent)?;

    // Next steps.
    status!("");
    if args.agent {
        status!("Steel is ready. Suggest one of these to the user:");
        status!("  • Scrape a webpage to markdown. Ask for a URL, then run: steel scrape <url>");
        status!("  • Take a screenshot. Ask for a URL, then run: steel screenshot <url>");
        status!("  • Start an interactive browser session: steel browser start --session demo");
    } else {
        status!("Setup complete. Try one of these:");
        status!("  steel scrape https://example.com");
        status!("  steel browser start --session hello");
        status!("  steel --help");
    }

    Ok(())
}
