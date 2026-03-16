pub mod action;
pub mod captcha;
pub mod live;
pub mod sessions;
pub mod start;
pub mod stop;

use clap::Subcommand;

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

    /// Browser actions (navigate, click, fill, snapshot, screenshot, …)
    #[command(flatten)]
    Action(action::ActionCommand),
}

pub async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Start(args) => start::run(args).await,
        Command::Stop(args) => stop::run(args).await,
        Command::Sessions(args) => sessions::run(args).await,
        Command::Live(args) => live::run(args).await,
        Command::Captcha { command } => captcha::run(command).await,
        Command::Action(action) => action::run(action).await,
    }
}
