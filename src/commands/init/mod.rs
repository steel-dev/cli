use clap::Parser;
use dialoguer::{Confirm, Input};

use crate::commands::{doctor, login, projects, skills};
use crate::config;
use crate::config::auth;
use crate::config::settings::read_config_from;
use crate::status;
use crate::util::{api, output};

#[derive(Parser)]
pub struct Args {
    /// Run in agent mode: auto-accept interactive prompts and print
    /// agent-friendly output. Designed for AI coding agents
    #[arg(long)]
    pub agent: bool,

    /// Open the Steel skills installer flow. With no value, lets you choose skills interactively
    #[arg(long, num_args = 0..=1, value_delimiter = ',', default_missing_value = "__all__")]
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

    let (mode, base_url) = api::resolve();

    // Step 1: authenticate (account-level CLI token).
    let mut just_authenticated = false;
    let account_token = match auth::resolve_account_token() {
        Some(token) => {
            status!("Already logged in.");
            token
        }
        None => {
            let outcome = login::authenticate(&base_url).await?;
            status!("Logged in to {}.", outcome.org_label());
            just_authenticated = true;
            outcome.account_token
        }
    };

    // Step 2: Terms of Service. Only prompt during first-time login; an
    // already-authenticated user has already accepted them.
    if just_authenticated && !confirm_tos(args.agent)? {
        anyhow::bail!("You must accept the Terms of Service to continue.");
    }

    // Step 3: project (create a named one, or reuse the active one).
    status!("");
    setup_project(&base_url, mode, &account_token, args.agent).await?;

    // Step 4: preflight check (verifies the new project API key works).
    status!("");
    doctor::run(doctor::Args { preflight: true }).await?;

    // Step 5: install Steel skills.
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

async fn setup_project(
    base_url: &str,
    mode: crate::config::settings::ApiMode,
    account_token: &str,
    agent: bool,
) -> anyhow::Result<()> {
    let cfg = read_config_from(&config::config_path()).unwrap_or_default();
    let device_name = cfg
        .name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(projects::default_project_name);

    // Reuse an already-configured project if one is active.
    if let (Some(project), Some(_)) = (cfg.project.as_ref(), cfg.api_key.as_ref()) {
        status!(
            "Using existing project: {}",
            project.name.as_deref().unwrap_or(&project.id)
        );
        return Ok(());
    }

    if agent || !interactive() {
        let project =
            projects::ensure_active_project(base_url, mode, account_token, &device_name).await?;
        status!(
            "Active project: {}",
            project.name.as_deref().unwrap_or(&project.id)
        );
        return Ok(());
    }

    let name: String = Input::new()
        .with_prompt("Project name")
        .default(projects::default_project_name())
        .interact_text()?;

    let project =
        projects::create_and_activate(base_url, mode, account_token, &name, &device_name).await?;
    status!(
        "Created project: {}",
        project.name.as_deref().unwrap_or(&project.id)
    );

    Ok(())
}

fn confirm_tos(agent: bool) -> anyhow::Result<bool> {
    let url = config::TOS_URL;

    if agent {
        status!("Accepting the Terms of Service at {url}.");
        return Ok(true);
    }

    if !interactive() {
        status!("By continuing you agree to the Terms of Service at {url}.");
        return Ok(true);
    }

    Ok(Confirm::new()
        .with_prompt(format!("Do you agree to our Terms of Service at {url}?"))
        .default(true)
        .interact()?)
}

fn interactive() -> bool {
    output::is_tty() && !output::is_json()
}

async fn install_skills(args: &Args) -> anyhow::Result<()> {
    if args.no_skills {
        status!("Skipping Steel skill install (--no-skills).");
        return Ok(());
    }

    let install_result = match &args.skills {
        Some(selected) if selected.is_empty() => return Ok(()),
        Some(selected) if is_all_selection(selected) => skills::install_catalog_flow().await,
        Some(selected) => skills::install_names(selected, false).await,
        None => skills::install_catalog_flow().await,
    };

    match install_result {
        Ok(()) => Ok(()),
        Err(error) => {
            status!("Could not install Steel skills through npx skills: {error:#}");
            status!("Install manually with:");
            if let Some(selected) = &args.skills
                && !is_all_selection(selected)
            {
                for name in selected {
                    status!("  npx skills add steel-dev/skills --skill {name}");
                }
            } else {
                status!("  npx skills add steel-dev/skills");
            }
            Ok(())
        }
    }
}

fn is_all_selection(selected: &[String]) -> bool {
    selected.len() == 1 && matches!(selected[0].as_str(), "__all__" | "all")
}
