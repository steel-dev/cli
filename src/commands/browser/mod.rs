pub mod action;
pub mod batch;
pub mod captcha;
pub mod live;
pub mod sessions;
pub mod start;
pub mod stop;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct BrowserArgs {
    /// Named session to target
    #[arg(long, global = true)]
    pub session: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create or attach to a browser session
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

    /// Run multiple browser commands in a single invocation
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

pub async fn run(args: BrowserArgs) -> anyhow::Result<()> {
    let session = args.session;
    if let Some(ref name) = session
        && let Some(err) = crate::browser::daemon::process::validate_session_name(name)
    {
        anyhow::bail!("{err}");
    }
    match args.command {
        Command::Start(args) => start::run(args, session.as_deref()).await,
        Command::Stop(args) => stop::run(args, session.as_deref()).await,
        Command::Sessions(args) => sessions::run(args).await,
        Command::Live(args) => live::run(args, session.as_deref()).await,
        Command::Captcha { command } => captcha::run(command, session.as_deref()).await,
        Command::Batch(args) => batch::run(args, session.as_deref()).await,
        Command::Action(action) => action::run(action, session.as_deref()).await,
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::action::{ActionCommand, SetCommand};
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
    fn session_after_setvalue_value_is_a_flag() {
        let args = parse(&["setvalue", "@e1", "42", "--session", "repro"]);
        assert_eq!(args.session.as_deref(), Some("repro"));
        let Command::Action(ActionCommand::SetValue(set)) = args.command else {
            panic!("expected setvalue");
        };
        assert_eq!(set.value, ["42"]);
    }

    #[test]
    fn session_after_user_agent_is_a_flag() {
        let args = parse(&[
            "set",
            "useragent",
            "Mozilla/5.0",
            "Test",
            "--session",
            "repro",
        ]);
        assert_eq!(args.session.as_deref(), Some("repro"));
        let Command::Action(ActionCommand::Set {
            command: SetCommand::UserAgent(ua),
        }) = args.command
        else {
            panic!("expected set useragent");
        };
        assert_eq!(ua.user_agent, ["Mozilla/5.0", "Test"]);
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
