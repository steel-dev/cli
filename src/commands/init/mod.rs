use std::path::Path;

use clap::Parser;

use crate::commands::{doctor, login, skills};
use crate::config;
use crate::config::settings::{OnboardingConfig, read_config_from, write_config_to};
use crate::status;

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
    if let Some(source) =
        onboarding_source_from_env(std::env::var("STEEL_ONBOARDING_FROM").ok().as_deref())
    {
        status!("Onboarding source: {source}");
        record_onboarding_source(&source);
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

fn onboarding_source_from_env(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|source| !source.is_empty())
        .map(str::to_string)
}

fn record_onboarding_source(source: &str) {
    let config_path = config::config_path_in(&config::config_dir());
    let _ = persist_onboarding_source(&config_path, source);
    crate::telemetry::set_onboarding_source(source);
}

fn persist_onboarding_source(config_path: &Path, source: &str) -> anyhow::Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut cfg = read_config_from(config_path).unwrap_or_default();
    cfg.onboarding = Some(OnboardingConfig {
        source: Some(source.to_string()),
    });
    write_config_to(config_path, &cfg)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn onboarding_source_from_env_trims_and_drops_blank() {
        assert_eq!(
            onboarding_source_from_env(Some(" claude-code ")).as_deref(),
            Some("claude-code")
        );
        assert_eq!(onboarding_source_from_env(Some("  ")), None);
        assert_eq!(onboarding_source_from_env(None), None);
    }

    #[test]
    fn persist_onboarding_source_creates_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("config.json");

        persist_onboarding_source(&path, "cursor").unwrap();

        let cfg = read_config_from(&path).unwrap();
        assert_eq!(cfg.onboarding_source(), Some("cursor"));
    }

    #[test]
    fn persist_onboarding_source_preserves_existing_fields() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"apiKey":"k","instance":"cloud"}"#).unwrap();

        persist_onboarding_source(&path, "codex").unwrap();

        let cfg = read_config_from(&path).unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("k"));
        assert_eq!(cfg.instance.as_deref(), Some("cloud"));
        assert_eq!(cfg.onboarding_source(), Some("codex"));
    }
}
