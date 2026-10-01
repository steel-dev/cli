// ABOUTME: `steel agent` runs a local coding agent (Claude Code or Codex) unattended on a task.
// ABOUTME: It prepares a Steel browser session, tells the agent how to use Steel, and cleans up after.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use serde_json::json;

use crate::browser::daemon::process;
use crate::commands::browser::start;
use crate::status;

/// Inactivity timeout for the agent session. Agents can think for minutes between browser
/// commands, so this is longer than the `browser start` default. It also releases the
/// session if `steel agent` is killed before it can stop the session.
const AGENT_INACTIVITY_TIMEOUT_MS: u64 = 600_000;

const BROWSER_SKILL: &str = "steel-browser";

#[derive(Parser)]
pub struct Args {
    /// Task for the agent, in natural language
    #[arg(required = true, num_args = 1..)]
    pub prompt: Vec<String>,

    /// Coding agent to run. Default: the first one found on PATH (claude, then codex)
    #[arg(long, value_enum)]
    pub agent: Option<AgentKind>,

    /// Do not start a browser session before the agent runs
    #[arg(long)]
    pub no_session: bool,

    /// Keep the browser session running after the agent exits
    #[arg(long, conflicts_with = "no_session")]
    pub keep_session: bool,

    /// Give the agent all tools with no approval prompts and no sandbox
    #[arg(long)]
    pub yolo: bool,

    /// Arguments passed to the agent unchanged (put them after `--`)
    #[arg(last = true)]
    pub agent_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentKind {
    Claude,
    Codex,
}

impl AgentKind {
    const ALL: [Self; 2] = [Self::Claude, Self::Codex];

    const fn program(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
        }
    }

    /// Agent name that `npx skills -a` expects.
    const fn skills_id(self) -> &'static str {
        match self {
            Self::Claude => "claude-code",
            Self::Codex => "codex",
        }
    }

    const fn other(self) -> Self {
        match self {
            Self::Claude => Self::Codex,
            Self::Codex => Self::Claude,
        }
    }
}

/// Tools that Claude Code can use without approval when `--yolo` is not set.
const CLAUDE_ALLOWED_TOOLS: &str = "Bash(steel:*),Read,Write,Edit,Glob,Grep,Skill";

pub async fn run(args: Args) -> anyhow::Result<()> {
    let path_env = std::env::var_os("PATH");
    let kind = resolve_agent(args.agent, path_env.as_deref())?;
    let prompt = args.prompt.join(" ");
    let cwd = std::env::current_dir()?;
    let skill_path = find_skill(kind, dirs::home_dir().as_deref(), &cwd);

    let session = if args.no_session {
        None
    } else {
        Some(start_session().await?)
    };

    print_banner(kind, &args, session.as_ref(), skill_path.as_deref());

    let preamble = build_preamble(
        session.as_ref().map(|s| s.name.as_str()),
        skill_path.as_deref(),
    );
    let agent_argv = build_agent_args(
        kind,
        &prompt,
        &preamble,
        args.yolo,
        &crate::config::config_dir(),
        &args.agent_args,
    );

    let mut properties = serde_json::Map::new();
    properties.insert("agent".into(), json!(kind.program()));
    properties.insert("session".into(), json!(session.is_some()));
    properties.insert("yolo".into(), json!(args.yolo));
    properties.insert("skill_installed".into(), json!(skill_path.is_some()));
    crate::telemetry::track_event("agent_started", properties);

    let result = run_agent(kind, &agent_argv, session.as_ref()).await;

    if let Some(ref session) = session {
        if args.keep_session {
            status!(
                "Browser session \"{}\" is still running. Stop it with `steel browser stop --session {}`.",
                session.name,
                session.name
            );
        } else {
            process::stop_daemon(&session.name).await?;
            status!("Stopped browser session \"{}\".", session.name);
        }
    }

    result
}

struct AgentSession {
    name: String,
    live_url: Option<String>,
}

async fn start_session() -> anyhow::Result<AgentSession> {
    let name = new_session_name();
    status!("Starting browser session \"{name}\"...");
    let info = start::create_session(
        start::Args {
            stealth: false,
            proxy: None,
            session_timeout: None,
            inactivity_timeout: Some(AGENT_INACTIVITY_TIMEOUT_MS),
            session_headless: None,
            session_region: None,
            session_solve_captcha: false,
            profile: None,
            update_profile: false,
            namespace: None,
            credentials: false,
        },
        &name,
    )
    .await?;
    Ok(AgentSession {
        name,
        live_url: info.viewer_url,
    })
}

fn new_session_name() -> String {
    let mut bytes = [0u8; 3];
    getrandom::fill(&mut bytes).expect("failed to generate random bytes");
    let suffix: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("agent-{suffix}")
}

/// Use the requested agent, or the first agent found on PATH.
fn resolve_agent(
    requested: Option<AgentKind>,
    path_env: Option<&OsStr>,
) -> anyhow::Result<AgentKind> {
    if let Some(kind) = requested {
        if find_on_path(kind.program(), path_env).is_none() {
            anyhow::bail!(
                "`{}` was not found on PATH. Install {} or use `--agent {}`.",
                kind.program(),
                kind.label(),
                kind.other().program()
            );
        }
        return Ok(kind);
    }
    AgentKind::ALL
        .into_iter()
        .find(|kind| find_on_path(kind.program(), path_env).is_some())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No coding agent found on PATH. Install Claude Code (https://claude.com/claude-code) or Codex (https://developers.openai.com/codex)."
            )
        })
}

fn find_on_path(program: &str, path_env: Option<&OsStr>) -> Option<PathBuf> {
    std::env::split_paths(path_env?)
        .map(|dir| dir.join(program))
        .find(|candidate| is_executable(candidate))
}

fn path_with_dir_first(dir: &Path, path_env: Option<&OsStr>) -> Option<OsString> {
    let rest = path_env.map(std::env::split_paths).into_iter().flatten();
    std::env::join_paths(std::iter::once(dir.to_path_buf()).chain(rest)).ok()
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Find the `steel-browser` skill file that this agent reads. A project skill wins over a
/// home skill. Skills installed only for other agents do not count.
fn find_skill(kind: AgentKind, home: Option<&Path>, cwd: &Path) -> Option<PathBuf> {
    let skill_dirs: &[&str] = match kind {
        AgentKind::Claude => &[".claude/skills"],
        AgentKind::Codex => &[".agents/skills", ".codex/skills"],
    };
    let roots = std::iter::once(cwd).chain(home);
    roots
        .flat_map(|root| {
            skill_dirs
                .iter()
                .map(move |dir| root.join(dir).join(BROWSER_SKILL).join("SKILL.md"))
        })
        .find(|path| path.is_file())
}

/// Instructions added to the prompt so the agent uses Steel for web work.
fn build_preamble(session: Option<&str>, skill_path: Option<&Path>) -> String {
    let mut text = String::from(
        "You run inside `steel agent` in unattended mode. No person will answer questions. \
Complete the task on your own and finish with a short summary of the result.\n\n\
Use the Steel CLI (`steel`) for all web browsing. It controls a real Steel cloud browser. \
Do not use other browser tools.\n",
    );

    if let Some(path) = skill_path {
        text.push_str(&format!(
            "Read the `{BROWSER_SKILL}` skill at `{}` for the full Steel command reference.\n",
            path.display()
        ));
    } else {
        text.push_str(
            "Run `steel browser --help` for the full command reference. The main commands are:\n\
  steel browser navigate <url>\n\
  steel browser snapshot -i          (list interactive elements)\n\
  steel browser click <selector>\n\
  steel browser fill <selector> <value>\n\
  steel browser get text <selector>\n\
  steel browser screenshot -o <path>\n\
  steel scrape <url>                 (get page content without a session)\n",
        );
    }

    match session {
        Some(name) => text.push_str(&format!(
            "\nA browser session named `{name}` is already running. Add `--session {name}` to \
every `steel browser` command. Do not start, stop, or replace browser sessions. `steel agent` \
stops this session when you finish.\n"
        )),
        None => text.push_str(
            "\nStart a browser session with `steel browser start --session <name>` and stop it \
with `steel browser stop --session <name>` when you finish.\n",
        ),
    }

    text
}

fn build_agent_args(
    kind: AgentKind,
    prompt: &str,
    preamble: &str,
    yolo: bool,
    steel_config_dir: &Path,
    extra: &[String],
) -> Vec<String> {
    let mut argv: Vec<String> = Vec::new();
    match kind {
        AgentKind::Claude => {
            argv.extend(["-p".into(), prompt.into()]);
            argv.extend(["--append-system-prompt".into(), preamble.into()]);
            if yolo {
                argv.push("--dangerously-skip-permissions".into());
            } else {
                argv.extend(["--allowedTools".into(), CLAUDE_ALLOWED_TOOLS.into()]);
            }
            argv.extend(extra.iter().cloned());
        }
        AgentKind::Codex => {
            argv.extend(["exec".into(), "--skip-git-repo-check".into()]);
            if yolo {
                argv.push("--dangerously-bypass-approvals-and-sandbox".into());
            } else {
                // The steel CLI needs the network and its config directory (daemon socket).
                argv.extend([
                    "--sandbox".into(),
                    "workspace-write".into(),
                    "-c".into(),
                    "sandbox_workspace_write.network_access=true".into(),
                    "--add-dir".into(),
                    steel_config_dir.display().to_string(),
                ]);
            }
            argv.extend(extra.iter().cloned());
            argv.push("--".into());
            argv.push(format!("{preamble}\n# Task\n\n{prompt}"));
        }
    }
    argv
}

fn print_banner(
    kind: AgentKind,
    args: &Args,
    session: Option<&AgentSession>,
    skill_path: Option<&Path>,
) {
    let mut lines = vec![format!(
        "steel agent: running {} unattended with these defaults:",
        kind.label()
    )];
    lines.push(format!(
        "  agent:   {}  [--agent {} to change]",
        kind.program(),
        kind.other().program()
    ));

    match session {
        Some(session) => {
            let live = session
                .live_url
                .as_deref()
                .map(|url| format!(", live view {url}"))
                .unwrap_or_default();
            let end = if args.keep_session {
                "kept after exit"
            } else {
                "stopped on exit [--keep-session to keep]"
            };
            lines.push(format!(
                "  browser: session {}{live}, {end} [--no-session to skip]",
                session.name
            ));
        }
        None => lines.push("  browser: no session started (--no-session)".to_string()),
    }

    let access = match (kind, args.yolo) {
        (_, true) => "all tools, no approvals, no sandbox (--yolo)".to_string(),
        (AgentKind::Claude, false) => {
            "steel CLI and file tools only [--yolo for all tools]".to_string()
        }
        (AgentKind::Codex, false) => {
            "workspace-write sandbox with network [--yolo to remove the sandbox]".to_string()
        }
    };
    lines.push(format!("  access:  {access}"));

    let skill = match skill_path {
        Some(path) => format!("{BROWSER_SKILL} skill at {}", path.display()),
        None => format!(
            "{BROWSER_SKILL} skill not installed for {}, basic commands added to the prompt [steel skills install {BROWSER_SKILL} -a {}]",
            kind.label(),
            kind.skills_id()
        ),
    };
    lines.push(format!("  steel:   {skill}"));
    lines.push("  extra agent flags go after `--`".to_string());

    // The banner goes to stderr even in JSON mode, so stdout is the agent's output only.
    eprintln!("{}\n", lines.join("\n"));
}

async fn run_agent(
    kind: AgentKind,
    argv: &[String],
    session: Option<&AgentSession>,
) -> anyhow::Result<()> {
    let mut command = tokio::process::Command::new(kind.program());
    command.args(argv);
    // The agent must use this `steel` binary, because it created the session.
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && let Some(path) = path_with_dir_first(dir, std::env::var_os("PATH").as_deref())
    {
        command.env("PATH", path);
    }
    if let Some(session) = session {
        command.env("STEEL_SESSION", &session.name);
    }

    let mut child = command
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start `{}`: {e}", kind.program()))?;

    // Ctrl-C goes to the whole process group, so the agent gets it too. Keep this process
    // alive until the agent exits, so that the browser session is stopped.
    let status = loop {
        tokio::select! {
            status = child.wait() => break status?,
            _ = tokio::signal::ctrl_c() => {}
        }
    };

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("{} exited with {status}", kind.program())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_bin_dir(programs: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for program in programs {
            let path = dir.path().join(program);
            std::fs::write(&path, "#!/bin/sh\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        dir
    }

    #[test]
    fn prefers_claude_when_both_are_installed() {
        let dir = fake_bin_dir(&["claude", "codex"]);
        let kind = resolve_agent(None, Some(dir.path().as_os_str())).unwrap();
        assert_eq!(kind, AgentKind::Claude);
    }

    #[test]
    fn falls_back_to_codex() {
        let dir = fake_bin_dir(&["codex"]);
        let kind = resolve_agent(None, Some(dir.path().as_os_str())).unwrap();
        assert_eq!(kind, AgentKind::Codex);
    }

    #[test]
    fn explicit_agent_wins() {
        let dir = fake_bin_dir(&["claude", "codex"]);
        let kind = resolve_agent(Some(AgentKind::Codex), Some(dir.path().as_os_str())).unwrap();
        assert_eq!(kind, AgentKind::Codex);
    }

    #[test]
    fn explicit_agent_missing_suggests_other() {
        let dir = fake_bin_dir(&["claude"]);
        let err = resolve_agent(Some(AgentKind::Codex), Some(dir.path().as_os_str())).unwrap_err();
        assert!(err.to_string().contains("--agent claude"), "{err}");
    }

    #[test]
    fn no_agent_found_errors() {
        let dir = fake_bin_dir(&[]);
        let err = resolve_agent(None, Some(dir.path().as_os_str())).unwrap_err();
        assert!(err.to_string().contains("No coding agent found"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_file_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("claude"), "").unwrap();
        assert!(find_on_path("claude", Some(dir.path().as_os_str())).is_none());
    }

    #[test]
    fn steel_dir_goes_first_on_path() {
        let path = path_with_dir_first(Path::new("/steel/bin"), Some(OsStr::new("/usr/bin:/bin")))
            .unwrap();
        assert_eq!(path, OsString::from("/steel/bin:/usr/bin:/bin"));
    }

    #[test]
    fn claude_args_are_scoped_by_default() {
        let argv = build_agent_args(
            AgentKind::Claude,
            "find flights",
            "PREAMBLE",
            false,
            Path::new("/cfg"),
            &["--model".into(), "opus".into()],
        );
        assert_eq!(
            argv,
            [
                "-p",
                "find flights",
                "--append-system-prompt",
                "PREAMBLE",
                "--allowedTools",
                CLAUDE_ALLOWED_TOOLS,
                "--model",
                "opus",
            ]
        );
    }

    #[test]
    fn claude_yolo_skips_permissions() {
        let argv = build_agent_args(AgentKind::Claude, "x", "P", true, Path::new("/cfg"), &[]);
        assert!(argv.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(!argv.contains(&"--allowedTools".to_string()));
    }

    #[test]
    fn codex_args_sandbox_with_network_by_default() {
        let argv = build_agent_args(
            AgentKind::Codex,
            "find flights",
            "PREAMBLE\n",
            false,
            Path::new("/cfg"),
            &[],
        );
        assert_eq!(
            argv,
            [
                "exec",
                "--skip-git-repo-check",
                "--sandbox",
                "workspace-write",
                "-c",
                "sandbox_workspace_write.network_access=true",
                "--add-dir",
                "/cfg",
                "--",
                "PREAMBLE\n\n# Task\n\nfind flights",
            ]
        );
    }

    #[test]
    fn codex_yolo_bypasses_sandbox() {
        let argv = build_agent_args(AgentKind::Codex, "x", "P", true, Path::new("/cfg"), &[]);
        assert!(argv.contains(&"--dangerously-bypass-approvals-and-sandbox".to_string()));
        assert!(!argv.contains(&"--sandbox".to_string()));
    }

    #[test]
    fn preamble_names_session_and_skill_path() {
        let text = build_preamble(Some("agent-abc123"), Some(Path::new("/h/SKILL.md")));
        assert!(text.contains("`agent-abc123` is already running"));
        assert!(text.contains("Add `--session agent-abc123` to every `steel browser` command"));
        assert!(text.contains("skill at `/h/SKILL.md`"));
        assert!(!text.contains("steel browser navigate"));
    }

    #[test]
    fn preamble_without_skill_or_session_inlines_commands() {
        let text = build_preamble(None, None);
        assert!(text.contains("steel browser navigate <url>"));
        assert!(text.contains("steel browser start --session"));
    }

    fn write_skill(root: &Path, rel: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "skill").unwrap();
    }

    #[test]
    fn skill_for_another_agent_does_not_count() {
        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        write_skill(home.path(), ".cursor/rules/steel-browser.mdc");
        write_skill(home.path(), ".claude/skills/steel-browser/SKILL.md");
        assert_eq!(
            find_skill(AgentKind::Codex, Some(home.path()), cwd.path()),
            None
        );
    }

    #[test]
    fn finds_skill_for_each_agent() {
        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        write_skill(home.path(), ".claude/skills/steel-browser/SKILL.md");
        write_skill(home.path(), ".agents/skills/steel-browser/SKILL.md");
        assert_eq!(
            find_skill(AgentKind::Claude, Some(home.path()), cwd.path()),
            Some(home.path().join(".claude/skills/steel-browser/SKILL.md"))
        );
        assert_eq!(
            find_skill(AgentKind::Codex, Some(home.path()), cwd.path()),
            Some(home.path().join(".agents/skills/steel-browser/SKILL.md"))
        );
    }

    #[test]
    fn project_skill_wins_over_home_skill() {
        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        write_skill(home.path(), ".agents/skills/steel-browser/SKILL.md");
        write_skill(cwd.path(), ".agents/skills/steel-browser/SKILL.md");
        assert_eq!(
            find_skill(AgentKind::Codex, Some(home.path()), cwd.path()),
            Some(cwd.path().join(".agents/skills/steel-browser/SKILL.md"))
        );
    }

    #[test]
    fn session_names_are_valid() {
        let name = new_session_name();
        assert!(name.starts_with("agent-"));
        assert!(process::validate_session_name(&name).is_none());
    }
}
