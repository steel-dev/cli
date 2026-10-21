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
    /// Install all or selected Steel skills through npx skills
    Install {
        /// Skill name(s) to install
        #[arg(required_unless_present = "all")]
        names: Vec<String>,
        /// Install all Steel skills from the catalog
        #[arg(long)]
        all: bool,
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

pub const fn telemetry_name(command: &SkillsCommand) -> &'static str {
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
            all,
            agent,
            global,
            yes,
        } => {
            if all {
                if !names.is_empty() {
                    anyhow::bail!("cannot combine --all with explicit skill names");
                }
                install_all_with_options(agent.as_deref(), global, yes).await
            } else {
                install_names_with_options(&names, agent.as_deref(), global, yes).await
            }
        }
        SkillsCommand::Update { names, yes } => update(names, yes).await,
        SkillsCommand::Doctor { offline } => doctor(offline).await,
        SkillsCommand::Open { name } => open_skill(&name).await,
        SkillsCommand::Paths => paths().await,
    }
}

pub async fn install_names(names: &[String], yes: bool) -> anyhow::Result<()> {
    install_names_with_options(names, None, false, yes).await
}

pub fn validate_setup_agent(source: Option<&str>) -> anyhow::Result<&str> {
    match source {
        Some(agent @ ("claude-code" | "codex" | "cursor" | "opencode" | "pi")) => Ok(agent),
        _ => anyhow::bail!(
            "Setup blocked: select a skill target with init --agent --from <claude-code|codex|cursor|opencode|pi>. The target is preserved when setup resumes."
        ),
    }
}

pub fn setup_load_action(agent: &str) -> &'static str {
    match agent {
        "opencode" => "OpenCode: call the skill tool with id steel-browser.",
        "claude-code" => "Claude Code: invoke /steel-browser.",
        "codex" => "Codex: invoke $steel-browser.",
        "cursor" => {
            "Cursor: invoke /steel-browser in Agent chat. If it is not discovered, start a new Agent session to rescan skills."
        }
        "pi" => {
            "Pi: use /reload in the running client after installation, then /skill:steel-browser."
        }
        _ => "Load steel-browser with your client's supported skill mechanism.",
    }
}

pub fn check_install_prerequisites() -> anyhow::Result<()> {
    for program in ["node", "npx", "git"] {
        if !ProcessCommand::new(program)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            anyhow::bail!(
                "Setup blocked at skill prerequisites: `{program}` is unavailable. The native Steel binary needs no Node.js, but the skills installer needs Node.js/npm and git. Install them in the agent tool environment, then rerun init --agent. npm will enforce the selected skills package's current Node engine requirement; no runtime is installed automatically."
            );
        }
    }
    Ok(())
}

pub async fn install_setup_names(names: &[String], agent: &str) -> anyhow::Result<()> {
    let manifest = load_manifest(false).await?;
    for name in names {
        ensure_known_skill(&manifest, name)?;
        if !manifest.skills[name]
            .agents
            .iter()
            .any(|target| target == agent)
        {
            anyhow::bail!("Skill {name} does not support {agent}; choose a compatible skill.");
        }
    }
    install_names_with_options(names, Some(agent), true, true).await?;
    for name in names {
        let path = setup_skill_path(agent, name)?;
        fs::read_to_string(&path)
            .with_context(|| format!("Skill installer succeeded but {name} is not readable at {}. Rerun init --agent after resolving the install path.", path.display()))?;
        status!(
            "Skill installed at {} (client loading still unverified)",
            path.display()
        );
    }
    Ok(())
}

pub async fn install_setup_all(agent: &str) -> anyhow::Result<()> {
    let manifest = load_manifest(false).await?;
    let names = manifest
        .skills
        .iter()
        .filter(|(_, meta)| meta.cli.include && meta.agents.iter().any(|target| target == agent))
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    install_setup_names(&names, agent).await
}

fn setup_skill_path(agent: &str, name: &str) -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("Cannot find home directory for skill installation")?;
    let claude_home = std::env::var("CLAUDE_CONFIG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".claude"));
    Ok(global_skill_path(&home, &claude_home, agent, name))
}

fn global_skill_path(
    home: &std::path::Path,
    claude_home: &std::path::Path,
    agent: &str,
    name: &str,
) -> PathBuf {
    let root = match agent {
        "claude-code" => claude_home.join("skills"),
        // skills 1.7.2 installs universal clients in the canonical shared
        // root, not their agent-specific globalSkillsDir. Verified with real
        // installs: OpenCode and Cursor discover this compatibility location.
        _ => home.join(".agents/skills"),
    };
    root.join(name).join("SKILL.md")
}

pub async fn install_all(yes: bool) -> anyhow::Result<()> {
    install_all_with_options(None, false, yes).await
}

pub async fn install_catalog_flow() -> anyhow::Result<()> {
    run_npx(&["skills", "add", REPO_SPEC])
        .context("starting Steel skills installer. You can also run manually: npx skills add steel-dev/skills")?;
    output::success_silent();
    Ok(())
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
    if names.is_empty() {
        anyhow::bail!("provide a skill name or use --all");
    }

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

async fn install_all_with_options(
    agent: Option<&str>,
    global: bool,
    yes: bool,
) -> anyhow::Result<()> {
    let mut args = vec!["skills", "add", REPO_SPEC, "--all"];
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
    run_npx(&args).context("installing all Steel skills. You can also run manually: npx skills add steel-dev/skills --all")?;
    status!("Installed all Steel skills");
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
    let has_fail = !npx_ok || !auth.has_credentials() || installed_count == 0;

    if output::is_json() {
        output::success_data(json!({
            "overall": if has_fail { "fail" } else if !manifest_ok { "degraded" } else { "pass" },
            "checks": {
                "manifest": manifest_ok,
                "npx": npx_ok,
                "auth": auth.has_credentials(),
                "installed_paths": installed_count,
            },
            "paths": agent_paths,
            "skill_loading": "unverified",
        }));
    } else {
        println!("Steel Skills Doctor\n");
        println!("Manifest reachable: {}", yes_no(manifest_ok));
        println!("npx available: {}", yes_no(npx_ok));
        println!("Steel auth configured: {}", yes_no(auth.has_credentials()));
        println!("Installed Steel skill paths detected: {installed_count}");
        println!("Client skill loading: unverified (load steel-browser in the selected client)");
        if installed_count == 0 {
            println!("Fix: run `steel init --agent --from <client>` to install steel-browser.");
        }
        if !npx_ok {
            println!("\nFix: install Node.js/npm so `npx skills` is available.");
        }
        if !auth.has_credentials() {
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
    if !offline && let Ok(manifest) = fetch_manifest().await {
        let _ = write_cached_manifest(&manifest);
        return Ok(manifest);
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
    let mut paths: Vec<_> = ["claude-code", "cursor", "opencode", "codex", "pi"]
        .into_iter()
        .filter_map(|agent| setup_skill_path(agent, name).ok().map(|path| (agent, path)))
        .map(|(agent, path)| AgentPath {
            agent,
            skill: name.to_string(),
            installed: path.exists(),
            path,
        })
        .collect();
    // Keep detecting native global roots used by older/manual installs too.
    if let Some(home) = dirs::home_dir() {
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        for (agent, root) in [
            ("cursor", home.join(".cursor/skills")),
            ("opencode", config_home.join("opencode/skills")),
        ] {
            let path = root.join(name).join("SKILL.md");
            paths.push(AgentPath {
                agent,
                skill: name.to_string(),
                installed: path.is_file(),
                path,
            });
        }
    }
    paths
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
    check_install_prerequisites()?;
    let status = ProcessCommand::new("npx")
        .arg("--yes")
        .args(args)
        // Respect the package's actual engine metadata rather than guessing a
        // historical minimum. Unsupported Node must fail, not warn and continue.
        .env("npm_config_engine_strict", "true")
        .status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(anyhow::anyhow!(
            "skills installer exited with status {status}. If npm reports EBADENGINE, install a Node.js version satisfying the Required engine printed above in the agent environment and rerun init --agent. Otherwise resolve the installer error above and retry."
        )),
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

const fn yes_no(value: bool) -> &'static str {
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

    #[test]
    fn setup_targets_are_explicit() {
        for agent in ["claude-code", "cursor", "opencode", "codex", "pi"] {
            assert_eq!(validate_setup_agent(Some(agent)).unwrap(), agent);
        }
        assert!(validate_setup_agent(None).is_err());
        assert!(validate_setup_agent(Some("unknown")).is_err());
    }

    #[test]
    fn global_paths_match_skills_installer_not_rules_or_agents() {
        let home = std::path::Path::new("/isolated");
        let claude = home.join("custom-claude");
        for (agent, expected) in [
            (
                "claude-code",
                "/isolated/custom-claude/skills/steel-browser/SKILL.md",
            ),
            ("cursor", "/isolated/.agents/skills/steel-browser/SKILL.md"),
            (
                "opencode",
                "/isolated/.agents/skills/steel-browser/SKILL.md",
            ),
            ("codex", "/isolated/.agents/skills/steel-browser/SKILL.md"),
            ("pi", "/isolated/.agents/skills/steel-browser/SKILL.md"),
        ] {
            assert_eq!(
                global_skill_path(home, &claude, agent, "steel-browser"),
                PathBuf::from(expected)
            );
        }
    }
}
