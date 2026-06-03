use clap::Parser;

use crate::commands::{doctor, login, skills};
use crate::status;

#[derive(Parser)]
pub struct Args {
    /// Run in agent mode: auto-accept interactive prompts and print
    /// agent-friendly output. Designed for AI coding agents
    #[arg(long)]
    pub agent: bool,

    /// Install selected Steel skills via the skills installer. With no value,
    /// installs the recommended bootstrap set
    #[arg(long, num_args = 0..=1, value_delimiter = ',', default_missing_value = "steel-browser,steel-developer")]
    pub skills: Option<Vec<String>>,

    /// Skip Steel skill installation
    #[arg(long)]
    pub no_skills: bool,
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

    // Step 3: install Steel skills.
    status!("");
    install_skills(&args).await?;

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

async fn install_skills(args: &Args) -> anyhow::Result<()> {
    if args.no_skills {
        status!("Skipping Steel skill install (--no-skills).");
        return Ok(());
    }

    let selected = args
        .skills
        .clone()
        .unwrap_or_else(|| vec!["steel-browser".to_string(), "steel-developer".to_string()]);

    if selected.is_empty() {
        return Ok(());
    }

    match skills::install_names(&selected, args.agent).await {
        Ok(()) => Ok(()),
        Err(error) => {
            status!("Could not install Steel skills through npx skills: {error:#}");
            status!("Install manually with:");
            for name in &selected {
                status!("  npx skills add steel-dev/skills --skill {name}");
            }
            Ok(())
        }
    }
}
