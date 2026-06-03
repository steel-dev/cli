use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;
use crate::status;
use crate::util::output;

const DEFAULT_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/steel-dev/skills/main/manifest.json";
const REPO_SPEC: &str = "steel-dev/skills";
const FALLBACK_MANIFEST: &str = r#"{
  "version": "2026.06.02",
  "schema": 1,
  "default_branch": "main",
  "skills": {
    "steel-browser": {
      "title": "Steel Browser",
      "description": "Skill for agent-driven web workflows using Steel cloud browsers and API tools.",
      "category": "operate",
      "stage": "ga",
      "path": "steel-browser",
      "version": "0.1.0",
      "owner": "Jun",
      "cli": { "include": true },
      "agents": ["claude-code", "cursor", "codex", "opencode", "pi"],
      "docs_url": "https://docs.steel.dev/overview/agent-skills/steel-browser",
      "install": {
        "skills_cli": "npx skills add steel-dev/skills --skill steel-browser",
        "steel_cli": "steel skills install steel-browser"
      },
      "requires": ["steel-cli"],
      "platform_features": ["browser-actions", "sessions", "captchas", "credentials", "profiles"]
    },
    "steel-developer": {
      "title": "Steel Developer",
      "description": "Skill for building reusable software on Steel cloud browsers.",
      "category": "build",
      "stage": "ga",
      "path": "steel-developer",
      "version": "0.1.0",
      "owner": "Dane",
      "cli": { "include": true },
      "agents": ["claude-code", "cursor", "codex", "opencode", "pi"],
      "docs_url": "https://docs.steel.dev/overview/agent-skills/steel-developer",
      "install": {
        "skills_cli": "npx skills add steel-dev/skills --skill steel-developer",
        "steel_cli": "steel skills install steel-developer"
      },
      "requires": ["steel-cli"],
      "platform_features": ["sdk", "sessions"]
    },
    "steel-session-debugging": {
      "title": "Steel Session Debugging",
      "description": "Skill for diagnosing failed Steel browser sessions.",
      "category": "debug",
      "stage": "beta",
      "path": "steel-session-debugging",
      "version": "0.1.0",
      "owner": "Nas",
      "cli": { "include": true },
      "agents": ["claude-code", "cursor", "codex", "opencode", "pi"],
      "docs_url": "https://docs.steel.dev/overview/agent-skills/steel-session-debugging",
      "install": {
        "skills_cli": "npx skills add steel-dev/skills --skill steel-session-debugging",
        "steel_cli": "steel skills install steel-session-debugging"
      },
      "requires": ["steel-cli"],
      "platform_features": ["sessions", "agent-traces"]
    },
    "steel-reliability": {
      "title": "Steel Reliability",
      "description": "Skill for diagnosing bot-detection, CAPTCHA, proxy, identity, and login reliability issues.",
      "category": "reliability",
      "stage": "beta",
      "path": "steel-reliability",
      "version": "0.1.0",
      "owner": "Nas",
      "cli": { "include": true },
      "agents": ["claude-code", "cursor", "codex", "opencode", "pi"],
      "docs_url": "https://docs.steel.dev/overview/agent-skills/steel-reliability",
      "install": {
        "skills_cli": "npx skills add steel-dev/skills --skill steel-reliability",
        "steel_cli": "steel skills install steel-reliability"
      },
      "requires": ["steel-cli", "steel-session-debugging"],
      "platform_features": ["proxies", "captchas", "profiles", "credentials"]
    },
    "steel-skill-creator": {
      "title": "Steel Skill Creator",
      "description": "Skill for turning repeated browser workflows into reusable skills.",
      "category": "create",
      "stage": "beta",
      "path": "steel-skill-creator",
      "version": "0.1.0",
      "owner": "Niko",
      "cli": { "include": true },
      "agents": ["claude-code"],
      "docs_url": "https://docs.steel.dev/overview/agent-skills/steel-skill-creator",
      "install": {
        "skills_cli": "npx skills add steel-dev/skills --skill steel-skill-creator",
        "steel_cli": "steel skills install steel-skill-creator"
      },
      "requires": ["steel-cli", "steel-browser"],
      "platform_features": ["sessions", "agent-traces", "browser-actions"]
    }
  }
}"#;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand)]
pub enum SkillsCommand {
    /// List available Steel skills
    List {
        /// Use cached or bundled manifest instead of fetching GitHub
        #[arg(long)]
        offline: bool,
    },
    /// Install one or more Steel skills through npx skills
    Install {
        /// Skill name(s) to install
        #[arg(required = true)]
        names: Vec<String>,
        /// Target agent passed to npx skills (-a)
        #[arg(short = 'a', long)]
        agent: Option<String>,
        /// Install globally through npx skills (-g)
        #[arg(short = 'g', long)]
        global: bool,
        /// Non-interactive yes passed to npx skills (-y)
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Update one or more installed Steel skills through npx skills
    Update {
        /// Skill name(s) to update; all known Steel skills when omitted
        names: Vec<String>,
        /// Non-interactive yes passed to npx skills (-y)
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Check Steel skills installation health
    Doctor {
        /// Use cached or bundled manifest instead of fetching GitHub
        #[arg(long)]
        offline: bool,
    },
    /// Open an installed skill or docs page
    Open {
        /// Skill name
        name: String,
    },
    /// Show detected agent install paths for Steel skills
    Paths,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Manifest {
    version: String,
    schema: u64,
    skills: BTreeMap<String, SkillMeta>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SkillMeta {
    title: String,
    description: String,
    category: String,
    stage: String,
    path: String,
    version: String,
    owner: String,
    cli: CliMeta,
    agents: Vec<String>,
    docs_url: String,
    install: InstallMeta,
    requires: Vec<String>,
    platform_features: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CliMeta {
    include: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct InstallMeta {
    skills_cli: String,
    steel_cli: String,
}

#[derive(Debug, Serialize)]
struct SkillRow {
    installed: bool,
    name: String,
    title: String,
    category: String,
    stage: String,
    version: String,
    docs_url: String,
}

#[derive(Debug, Serialize)]
struct AgentPath {
    agent: &'static str,
    skill: String,
    path: PathBuf,
    installed: bool,
}

pub fn telemetry_name(command: &SkillsCommand) -> &'static str {
    match command {
        SkillsCommand::List { .. } => "list",
        SkillsCommand::Install { .. } => "install",
        SkillsCommand::Update { .. } => "update",
        SkillsCommand::Doctor { .. } => "doctor",
        SkillsCommand::Open { .. } => "open",
        SkillsCommand::Paths => "paths",
    }
}

pub async fn run(command: SkillsCommand) -> anyhow::Result<()> {
    match command {
        SkillsCommand::List { offline } => list(offline).await,
        SkillsCommand::Install {
            names,
            agent,
            global,
            yes,
        } => install_names_with_options(&names, agent.as_deref(), global, yes).await,
        SkillsCommand::Update { names, yes } => update(names, yes).await,
        SkillsCommand::Doctor { offline } => doctor(offline).await,
        SkillsCommand::Open { name } => open_skill(&name).await,
        SkillsCommand::Paths => paths().await,
    }
}

pub async fn install_names(names: &[String], yes: bool) -> anyhow::Result<()> {
    install_names_with_options(names, None, true, yes).await
}

async fn list(offline: bool) -> anyhow::Result<()> {
    let manifest = load_manifest(offline).await?;
    let rows = skill_rows(&manifest);

    if output::is_json() {
        output::success_data(json!({
            "version": manifest.version,
            "skills": rows,
        }));
        return Ok(());
    }

    println!("Available Steel Skills\n");
    println!(
        "{:<10} {:<24} {:<12} {:<8}",
        "Installed", "Name", "Category", "Stage"
    );
    for row in rows {
        println!(
            "{:<10} {:<24} {:<12} {:<8}",
            if row.installed { "yes" } else { "no" },
            row.name,
            title_case(&row.category),
            row.stage.to_uppercase()
        );
    }
    Ok(())
}

async fn install_names_with_options(
    names: &[String],
    agent: Option<&str>,
    global: bool,
    yes: bool,
) -> anyhow::Result<()> {
    let manifest = load_manifest(false).await?;
    for name in names {
        ensure_known_skill(&manifest, name)?;
        let mut args = vec!["skills", "add", REPO_SPEC, "--skill", name.as_str()];
        if let Some(agent) = agent {
            args.push("-a");
            args.push(agent);
        }
        if global {
            args.push("-g");
        }
        if yes {
            args.push("-y");
        }
        run_npx(&args).with_context(|| {
            format!(
                "installing {name}. You can also run manually: {}",
                manifest.skills[name].install.skills_cli
            )
        })?;
        status!("Installed {name}");
    }
    output::success_silent();
    Ok(())
}

async fn update(names: Vec<String>, yes: bool) -> anyhow::Result<()> {
    let manifest = load_manifest(false).await?;
    let selected = if names.is_empty() {
        manifest.skills.keys().cloned().collect::<Vec<_>>()
    } else {
        names
    };
    for name in selected {
        ensure_known_skill(&manifest, &name)?;
        let mut args = vec!["skills", "update", name.as_str()];
        if yes {
            args.push("-y");
        }
        run_npx(&args).with_context(|| format!("updating {name}"))?;
        status!("Updated {name}");
    }
    output::success_silent();
    Ok(())
}

async fn doctor(offline: bool) -> anyhow::Result<()> {
    let (manifest, manifest_ok) = if offline {
        (
            read_cached_manifest().unwrap_or_else(|_| fallback_manifest()),
            false,
        )
    } else {
        match fetch_manifest().await {
            Ok(manifest) => {
                let _ = write_cached_manifest(&manifest);
                (manifest, true)
            }
            Err(_) => (
                read_cached_manifest().unwrap_or_else(|_| fallback_manifest()),
                false,
            ),
        }
    };
    let npx_ok = npx_available();
    let auth = crate::config::auth::resolve_auth();
    let agent_paths = agent_paths_for(&manifest);
    let installed_count = agent_paths.iter().filter(|path| path.installed).count();
    let has_fail = !npx_ok || auth.api_key.is_none();

    if output::is_json() {
        output::success_data(json!({
            "overall": if has_fail { "fail" } else if !manifest_ok { "degraded" } else { "pass" },
            "checks": {
                "manifest": manifest_ok,
                "npx": npx_ok,
                "auth": auth.api_key.is_some(),
                "installed_paths": installed_count,
            },
            "paths": agent_paths,
        }));
    } else {
        println!("Steel Skills Doctor\n");
        println!("Manifest reachable: {}", yes_no(manifest_ok));
        println!("npx available: {}", yes_no(npx_ok));
        println!("Steel auth configured: {}", yes_no(auth.api_key.is_some()));
        println!("Installed Steel skill paths detected: {installed_count}");
        if !npx_ok {
            println!("\nFix: install Node.js/npm so `npx skills` is available.");
        }
        if auth.api_key.is_none() {
            println!("Fix: run `steel login` or set STEEL_API_KEY.");
        }
        if !manifest_ok {
            println!("Note: using cached or bundled manifest fallback.");
        }
    }

    if has_fail {
        return Err(output::SilentExit(output::exit_code::GENERAL).into());
    }
    Ok(())
}

async fn open_skill(name: &str) -> anyhow::Result<()> {
    let manifest = load_manifest(false).await?;
    ensure_known_skill(&manifest, name)?;
    let installed = agent_paths_for(&manifest)
        .into_iter()
        .find(|path| path.skill == name && path.installed);
    let target = installed
        .map(|path| path.path.to_string_lossy().to_string())
        .unwrap_or_else(|| manifest.skills[name].docs_url.clone());
    open::that(&target).with_context(|| format!("opening {target}"))?;
    Ok(())
}

async fn paths() -> anyhow::Result<()> {
    let manifest = load_manifest(true).await?;
    let paths = agent_paths_for(&manifest);
    if output::is_json() {
        output::success_data(json!(paths));
    } else {
        for path in paths {
            println!(
                "{:<12} {:<24} {}{}",
                path.agent,
                path.skill,
                path.path.display(),
                if path.installed { "  (installed)" } else { "" }
            );
        }
    }
    Ok(())
}

async fn load_manifest(offline: bool) -> anyhow::Result<Manifest> {
    if !offline {
        if let Ok(manifest) = fetch_manifest().await {
            let _ = write_cached_manifest(&manifest);
            return Ok(manifest);
        }
    }
    if let Ok(manifest) = read_cached_manifest() {
        return Ok(manifest);
    }
    Ok(fallback_manifest())
}

async fn fetch_manifest() -> anyhow::Result<Manifest> {
    let url = std::env::var("STEEL_SKILLS_MANIFEST_URL")
        .unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_string());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    let manifest = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<Manifest>()
        .await?;
    Ok(manifest)
}

fn fallback_manifest() -> Manifest {
    serde_json::from_str(FALLBACK_MANIFEST).expect("embedded skills manifest is valid")
}

fn cache_path() -> PathBuf {
    config::config_dir().join("skills").join("manifest.json")
}

fn read_cached_manifest() -> anyhow::Result<Manifest> {
    let content = fs::read_to_string(cache_path())?;
    Ok(serde_json::from_str(&content)?)
}

fn write_cached_manifest(manifest: &Manifest) -> anyhow::Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

fn ensure_known_skill(manifest: &Manifest, name: &str) -> anyhow::Result<()> {
    if manifest.skills.contains_key(name) {
        return Ok(());
    }
    let known = manifest
        .skills
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    Err(anyhow::anyhow!(
        "Unknown Steel skill `{name}`. Known skills: {known}"
    ))
}

fn skill_rows(manifest: &Manifest) -> Vec<SkillRow> {
    manifest
        .skills
        .iter()
        .map(|(name, meta)| SkillRow {
            installed: is_skill_installed(name),
            name: name.clone(),
            title: meta.title.clone(),
            category: meta.category.clone(),
            stage: meta.stage.clone(),
            version: meta.version.clone(),
            docs_url: meta.docs_url.clone(),
        })
        .collect()
}

fn agent_paths_for(manifest: &Manifest) -> Vec<AgentPath> {
    manifest
        .skills
        .keys()
        .flat_map(|name| agent_paths_for_skill(name))
        .collect()
}

fn agent_paths_for_skill(name: &str) -> Vec<AgentPath> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };

    let paths = [
        (
            "Claude Code",
            home.join(".claude")
                .join("skills")
                .join(name)
                .join("SKILL.md"),
        ),
        (
            "Cursor",
            home.join(".cursor")
                .join("rules")
                .join(format!("{name}.mdc")),
        ),
        (
            "OpenCode",
            home.join(".config")
                .join("opencode")
                .join("agents")
                .join(format!("{name}.md")),
        ),
        (
            "Codex/Pi",
            home.join(".agents")
                .join("skills")
                .join(name)
                .join("SKILL.md"),
        ),
    ];

    paths
        .into_iter()
        .map(|(agent, path)| AgentPath {
            agent,
            skill: name.to_string(),
            installed: path.exists(),
            path,
        })
        .collect()
}

fn is_skill_installed(name: &str) -> bool {
    agent_paths_for_skill(name)
        .iter()
        .any(|path| path.installed)
}

fn npx_available() -> bool {
    ProcessCommand::new("npx")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn run_npx(args: &[&str]) -> anyhow::Result<()> {
    let status = ProcessCommand::new("npx").args(args).status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(anyhow::anyhow!("npx exited with status {status}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(anyhow::anyhow!(
            "`npx` was not found. Install Node.js/npm, then run `npx skills add {REPO_SPEC} --skill <name>`."
        )),
        Err(error) => Err(error.into()),
    }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_manifest_contains_launch_skills() {
        let manifest = fallback_manifest();
        for name in [
            "steel-browser",
            "steel-developer",
            "steel-session-debugging",
            "steel-reliability",
            "steel-skill-creator",
        ] {
            assert!(manifest.skills.contains_key(name));
        }
    }

    #[test]
    fn unknown_skill_lists_known_names() {
        let manifest = fallback_manifest();
        let err = ensure_known_skill(&manifest, "missing").unwrap_err();
        assert!(err.to_string().contains("steel-browser"));
    }
}
