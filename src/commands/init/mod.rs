pub mod agents;

use clap::Parser;

use crate::commands::{doctor, login};
use crate::status;

#[derive(Parser)]
pub struct Args {
    /// Print the agent onboarding guide to stdout and exit.
    ///
    /// Use this when you want an AI coding agent (Claude Code, Cursor, etc.)
    /// to read the authoritative setup instructions before running `steel init`.
    #[arg(long)]
    pub agent: bool,
}

const AGENT_GUIDE: &str = include_str!("init_agent_guide.md");

pub async fn run(args: Args) -> anyhow::Result<()> {
    if args.agent {
        print!("{AGENT_GUIDE}");
        return Ok(());
    }

    status!("Steel CLI setup");
    status!("");

    // Step 1: login (no-ops if already logged in).
    login::run(login::Args {}).await?;

    // Step 2: preflight check (calls process::exit on failure).
    status!("");
    doctor::run(doctor::Args { preflight: true }).await?;

    // Step 3: auto-install the Steel skill into any detected coding agents.
    status!("");
    agents::install_skills_interactive()?;

    // Next steps.
    status!("");
    status!("Setup complete. Try one of these:");
    status!("  steel scrape https://example.com");
    status!("  steel browser start --session hello");
    status!("  steel --help");

    Ok(())
}
