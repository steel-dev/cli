use std::path::Path;

use clap::Parser;
use serde_json::json;

use crate::commands::{doctor, login, skills};
use crate::config;
use crate::config::settings::{OnboardingConfig, read_config_from, write_config_to};
use crate::status;
use crate::util::output;

#[derive(Parser)]
pub struct Args {
    /// Run in agent mode: auto-accept interactive prompts and print
    /// agent-friendly output. Designed for AI coding agents
    #[arg(long)]
    pub agent: bool,

    /// Coding client for targeted agent setup (currently: opencode)
    #[arg(long = "from")]
    pub from: Option<String>,

    /// Open the Steel skills installer flow. With no value, lets you choose skills interactively
    #[arg(long, num_args = 0..=1, value_delimiter = ',', default_missing_value = "__all__")]
    pub skills: Option<Vec<String>>,

    /// Skip Steel skill installation
    #[arg(long)]
    pub no_skills: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    status!("Steel CLI setup");
    if let Some(source) = onboarding_source_from_env(args.from.as_deref()) {
        skills::validate_setup_agent(&source)?;
        status!("Onboarding source: {source}");
        record_onboarding_source(&source)?;
    }
    // The persisted explicit selection is the only resume target. Do not infer
    // a client from environment variables or provider credentials.
    let source = config::settings::read_config()
        .ok()
        .and_then(|cfg| cfg.onboarding_source().map(str::to_string));
    let targeted_agent = args.agent && source.as_deref() == Some("opencode");
    status!("");

    // A targeted installation checks its local tools before beginning device
    // authorization, so a pending approval is never created for a host that
    // cannot complete the requested setup.
    if targeted_agent && !args.no_skills {
        skills::check_install_prerequisites()?;
    }

    // Step 1: login (no-ops if already logged in).
    let login = login::authorize(login::Args {
        no_wait: args.agent,
        no_browser: args.agent,
        ..Default::default()
    })
    .await?;
    if login["status"] != "authenticated" || !config::auth::resolve_auth().has_credentials() {
        // The resumable handoff is this run's only result; do not report setup ready.
        let mut handoff = login;
        if let Some(source) = &source {
            handoff["client"] = json!(source);
        }
        output::success_data(handoff);
        return Ok(());
    }

    // Step 2: preflight check.
    status!("");
    let report = doctor::report(true).await;
    if report.failed() {
        return report.emit();
    }
    if !output::is_json() {
        report.print_text();
    }

    // Step 3: install Steel skills.
    status!("");
    let installed_skills = install_skills(&args, targeted_agent.then_some("opencode")).await?;

    // Next steps.
    status!("");
    if targeted_agent {
        status!("CLI/auth/API configured; skill loading and first task remain unverified.");
        status!("Load steel-browser through OpenCode's skill mechanism before using it.");
    } else if args.agent {
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

    if output::is_json() {
        if targeted_agent {
            output::success_data(json!({
                "status": "incomplete",
                "doctor": report.to_json(),
                "skills": installed_skills,
                "client": source,
                "skill_loading": "unverified",
                "first_task": "unverified",
            }));
        } else {
            output::success_data(json!({ "doctor": report.to_json(), "skills": installed_skills }));
        }
    }

    Ok(())
}

async fn install_skills(args: &Args, source: Option<&str>) -> anyhow::Result<&'static str> {
    if args.no_skills {
        status!("Skipping Steel skill install (--no-skills).");
        return Ok("skipped");
    }

    let install_result = if args.agent && source.is_some() {
        let names = match &args.skills {
            Some(selected) if selected.is_empty() => return Ok("skipped"),
            Some(selected) if is_all_selection(selected) => vec!["steel-browser".to_string()],
            Some(selected) => selected.clone(),
            None => vec!["steel-browser".to_string()],
        };
        skills::install_setup_names(&names, source.expect("checked above")).await
    } else {
        match &args.skills {
            Some(selected) if selected.is_empty() => return Ok("skipped"),
            Some(selected) if is_all_selection(selected) => skills::install_catalog_flow().await,
            Some(selected) => skills::install_names(selected, false).await,
            None => skills::install_catalog_flow().await,
        }
    };

    match install_result {
        Ok(()) => Ok("installed"),
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
            if source.is_some() {
                Err(anyhow::anyhow!(
                    "Setup incomplete at skill installation. Resolve the error above, then rerun init --agent --from opencode: {error:#}"
                ))
            } else {
                // Preserve the prior general `init` flow. Targeted setup is
                // deliberately fail-closed; un-targeted callers retain their
                // existing best-effort catalog installer behavior.
                Ok("failed")
            }
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
}
