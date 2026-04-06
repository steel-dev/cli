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
