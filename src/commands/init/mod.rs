use std::path::Path;

use clap::Parser;

use crate::commands::{doctor, login, skills};
use crate::config;
use crate::config::settings::{OnboardingConfig, read_config_from, write_config_to};
use crate::status;
use crate::util::output;

#[derive(Parser)]
pub struct Args {
    /// Run non-interactive setup for an AI coding agent, with resumable device authorization
    #[arg(long)]
    pub agent: bool,

    /// Coding agent to install skills for (also persisted for resumed setup)
    #[arg(long = "from")]
    pub from: Option<String>,

    /// Select Steel skills (all compatible skills if no value). Without this
    /// flag, agent mode installs steel-browser; human mode opens the picker
    #[arg(long, num_args = 0..=1, value_delimiter = ',', default_missing_value = "__all__")]
    pub skills: Option<Vec<String>>,

    /// Skip Steel skill installation
    #[arg(long)]
    pub no_skills: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    status!("Steel CLI setup");
    if let Some(source) = onboarding_source_from_env(args.from.as_deref()).or_else(|| {
        onboarding_source_from_env(std::env::var("STEEL_ONBOARDING_FROM").ok().as_deref())
    }) {
        status!("Onboarding source: {source}");
        record_onboarding_source(&source)?;
    }
    status!("");

    let source = crate::config::settings::read_config()
        .ok()
        .and_then(|cfg| cfg.onboarding_source().map(str::to_string));
    if args.agent && !args.no_skills {
        skills::validate_setup_agent(source.as_deref())?;
        skills::check_install_prerequisites()?;
    }

    login::run(login::Args {
        no_wait: args.agent,
        no_browser: args.agent,
        resume: false,
    })
    .await?;
    if !crate::config::auth::resolve_auth().has_credentials() {
        output::success_data(
            serde_json::json!({"status":"incomplete", "stage":"authorization", "next_action":"Approve the displayed code, then rerun init --agent. The pending login is saved; no localhost callback is required."}),
        );
        return Ok(());
    }

    // Step 2: preflight check.
    status!("");
    doctor::run(doctor::Args { preflight: true }).await.map_err(|error| {
        anyhow::anyhow!("Setup blocked at API preflight. Resolve the doctor failure, then rerun init --agent: {error}")
    })?;

    // Step 3: install Steel skills.
    status!("");
    install_skills(&args, source.as_deref()).await?;
    crate::telemetry::track_event(
        "setup_configured",
        serde_json::Map::from_iter([
            (
                "skills_installed".to_string(),
                serde_json::json!(!args.no_skills),
            ),
            (
                "skill_loading_verified".to_string(),
                serde_json::json!(false),
            ),
            (
                "task_success_verified".to_string(),
                serde_json::json!(false),
            ),
        ]),
    );

    // Next steps.
    status!("");
    if args.agent {
        let next = if args.no_skills {
            "Skills skipped. Rerun init --agent without --no-skills to install steel-browser."
        } else {
            "Load steel-browser through your client's skill mechanism. Installation is not proof of loading. Then ask: Use Steel to open https://books.toscrape.com/, find A Light in the Attic, open its detail page, and tell me its price and availability. Stop the session afterward."
        };
        let load_action = source.as_deref().map(skills::setup_load_action);
        status!(
            "Setup incomplete: CLI/auth/API configured; skill loading and first task are not verified."
        );
        status!("{next}");
        if let Some(action) = load_action {
            status!("{action}");
        }
        output::success_data(serde_json::json!({
            "status": "incomplete",
            "cli_version": env!("CARGO_PKG_VERSION"),
            "skills_installed": !args.no_skills,
            "skill_loading": "unverified",
            "first_task": "unverified",
            "client": source,
            "skill_load_action": load_action,
            "next_action": next,
        }));
    } else {
        status!("Setup complete. Try one of these:");
        status!("  steel scrape https://example.com");
        status!("  steel browser start --session hello");
        status!("  steel --help");
    }

    Ok(())
}

async fn install_skills(args: &Args, source: Option<&str>) -> anyhow::Result<()> {
    if args.no_skills {
        status!("Skipping Steel skill install (--no-skills).");
        return Ok(());
    }

    let install_result = match &args.skills {
        selected if args.agent => {
            let agent = skills::validate_setup_agent(source)?;
            if selected
                .as_ref()
                .is_some_and(|names| is_all_selection(names))
            {
                skills::install_setup_all(agent).await
            } else {
                let names = selected
                    .clone()
                    .unwrap_or_else(|| vec!["steel-browser".to_string()]);
                skills::install_setup_names(&names, agent).await
            }
        }
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
            Err(anyhow::anyhow!(
                "Setup incomplete at skill installation. Resolve the error above, then rerun init --agent: {error:#}"
            ))
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

fn record_onboarding_source(source: &str) -> anyhow::Result<()> {
    let config_path = config::config_path_in(&config::config_dir());
    persist_onboarding_source(&config_path, source)?;
    crate::telemetry::set_onboarding_source(source);
    Ok(())
}

fn persist_onboarding_source(config_path: &Path, source: &str) -> anyhow::Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut cfg = if config_path.exists() {
        read_config_from(config_path)?
    } else {
        Default::default()
    };
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

    #[test]
    fn corrupt_configuration_is_not_overwritten_during_setup() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "invalid user config").unwrap();
        assert!(persist_onboarding_source(&path, "opencode").is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "invalid user config"
        );
    }
}
