pub mod action;
pub mod batch;
pub mod captcha;
pub mod live;
pub mod sessions;
pub mod start;
pub mod stop;

use clap::{Parser, Subcommand};

use crate::browser::state::{self, BrowserSession};

pub const DEFAULT_SESSION: &str = "default";

#[derive(Parser)]
pub struct BrowserArgs {
    /// Named session to target, or a session ID
    #[arg(long, global = true)]
    pub session: Option<String>,

    /// Print what the command changed on the page (a diff against the last snapshot)
    #[arg(long, global = true)]
    pub diff: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a browser session
    Start(start::Args),

    /// Stop a browser session
    Stop(stop::Args),

    /// List active browser sessions
    Sessions(sessions::Args),

    /// Open the live session viewer
    Live(live::Args),

    /// CAPTCHA management
    Captcha {
        #[command(subcommand)]
        command: captcha::Command,
    },

    /// Run several browser commands in one request
    Batch(batch::Args),

    /// Browser actions (navigate, click, fill, snapshot, screenshot, …)
    #[command(flatten)]
    Action(action::ActionCommand),
}

impl Command {
    pub fn telemetry_name(&self) -> String {
        match self {
            Self::Start(_) => "start".to_string(),
            Self::Stop(_) => "stop".to_string(),
            Self::Sessions(_) => "sessions".to_string(),
            Self::Live(_) => "live".to_string(),
            Self::Captcha { command } => format!("captcha.{}", command.telemetry_name()),
            Self::Batch(_) => "batch".to_string(),
            Self::Action(action) => action.telemetry_name().to_string(),
        }
    }
}

pub enum Target {
    Saved(Box<BrowserSession>),
    Id(String),
}

pub fn target(name: Option<&str>) -> anyhow::Result<Option<Target>> {
    let wanted = name.unwrap_or(DEFAULT_SESSION);
    if let Some(err) = state::validate_session_name(wanted) {
        anyhow::bail!("{err}");
    }
    if let Some(found) = state::load(wanted)? {
        return Ok(Some(Target::Saved(Box::new(found))));
    }
    if state::looks_like_session_id(wanted) {
        return Ok(Some(Target::Id(wanted.to_string())));
    }
    Ok(None)
}

pub fn not_running(name: Option<&str>) -> anyhow::Error {
    match name {
        Some(name) => anyhow::anyhow!(
            "Session \"{name}\" is not running. It may have expired. Start it with `steel browser start --session {name}`."
        ),
        None => anyhow::anyhow!("No active browser session. Start one with `steel browser start`."),
    }
}

pub fn existing_session(name: Option<&str>) -> anyhow::Result<BrowserSession> {
    Ok(match target(name)?.ok_or_else(|| not_running(name))? {
        Target::Saved(found) => *found,
        Target::Id(id) => {
            let (mode, base_url) = crate::util::api::resolve();
            BrowserSession::new(id.clone(), id, mode, base_url)
        }
    })
}

pub async fn run(args: BrowserArgs) -> anyhow::Result<()> {
    let session = args.session;
    if let Some(ref name) = session
        && let Some(err) = state::validate_session_name(name)
    {
        anyhow::bail!("{err}");
    }
    match args.command {
        Command::Start(start_args) => start::run(start_args, session.as_deref()).await,
        Command::Stop(stop_args) => stop::run(stop_args, session.as_deref()).await,
        Command::Sessions(sessions_args) => sessions::run(sessions_args).await,
        Command::Live(live_args) => live::run(live_args, session.as_deref()).await,
        Command::Captcha { command } => captcha::run(command, session.as_deref()).await,
        Command::Batch(batch_args) => batch::run(batch_args, session.as_deref(), args.diff).await,
        Command::Action(action) => action::run(action, session.as_deref(), args.diff).await,
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::action::ActionCommand;
    use super::{BrowserArgs, Command};
    use crate::commands::{self, Cli};

    fn parse(args: &[&str]) -> BrowserArgs {
        let argv = ["steel", "browser"].into_iter().chain(args.iter().copied());
        match Cli::try_parse_from(argv).unwrap().command {
            commands::Command::Browser(browser) => browser,
            _ => panic!("expected browser command"),
        }
    }

    #[test]
    fn target_rejects_paths_and_accepts_ids() {
        assert!(super::target(Some("../etc/passwd")).is_err());
        let id = "4f7c2b1e-9a3d-4e5f-8b6a-1c2d3e4f5a6b";
        assert!(
            matches!(super::target(Some(id)), Ok(Some(super::Target::Id(found))) if found == id)
        );
    }

    #[test]
    fn session_after_fill_value_is_a_flag() {
        let args = parse(&["fill", "@e1", "herman", "miller", "--session", "repro"]);
        assert_eq!(args.session.as_deref(), Some("repro"));
        let Command::Action(ActionCommand::Fill(fill)) = args.command else {
            panic!("expected fill");
        };
        assert_eq!(fill.value, ["herman", "miller"]);
    }

    #[test]
    fn flags_after_type_text_are_flags() {
        let args = parse(&[
            "type",
            "@e1",
            "hello",
            "--clear",
            "--delay",
            "50",
            "--session",
            "repro",
        ]);
        assert_eq!(args.session.as_deref(), Some("repro"));
        let Command::Action(ActionCommand::Type(typed)) = args.command else {
            panic!("expected type");
        };
        assert_eq!(typed.text, ["hello"]);
        assert!(typed.clear);
        assert_eq!(typed.delay, Some(50));
    }

    #[test]
    fn diff_is_a_flag_anywhere() {
        let args = parse(&["click", "e5", "--diff"]);
        assert!(args.diff);
        let args = parse(&["--diff", "press", "Enter"]);
        assert!(args.diff);
        assert!(!parse(&["press", "Enter"]).diff);
    }

    #[test]
    fn double_dash_keeps_flag_like_text() {
        let args = parse(&["fill", "@e1", "--", "--session", "repro"]);
        assert_eq!(args.session, None);
        let Command::Action(ActionCommand::Fill(fill)) = args.command else {
            panic!("expected fill");
        };
        assert_eq!(fill.value, ["--session", "repro"]);
    }
}
