pub mod browser;
pub mod cache;
pub mod config;
pub mod credentials;
pub mod dev;
pub mod docs;
pub mod forge;
pub mod login;
pub mod logout;
pub mod pdf;
pub mod profile;
pub mod run;
pub mod scrape;
pub mod screenshot;
pub mod settings;
pub mod star;
pub mod support;
pub mod update;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "steel", version, about = "Steel CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Skip update check
    #[arg(long, global = true)]
    pub no_update_check: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Internal daemon process
    #[command(name = "__daemon", hide = true)]
    Daemon {
        #[arg(long)]
        session_id: String,
    },

    /// Scrape webpage content (markdown output by default)
    Scrape(scrape::Args),

    /// Capture a screenshot of a webpage
    Screenshot(screenshot::Args),

    /// Generate a PDF from a webpage
    Pdf(pdf::Args),

    /// Browser session management and automation
    Browser {
        #[command(subcommand)]
        command: browser::Command,
    },

    /// Login to Steel CLI
    #[command(alias = "auth")]
    Login(login::Args),

    /// Logout from Steel CLI
    Logout(logout::Args),

    /// Manage stored credentials
    Credentials {
        #[command(subcommand)]
        command: credentials::Command,
    },

    /// Local development runtime
    Dev {
        #[command(subcommand)]
        command: dev::Command,
    },

    /// Start a new project using the Steel CLI
    Forge(forge::Args),

    /// Run a Steel Cookbook automation
    Run(run::Args),

    /// Show current configuration
    Config(config::Args),

    /// Check for updates and install the latest version
    Update(update::Args),

    /// Manage Steel CLI cache
    Cache(cache::Args),

    /// Open Steel documentation in browser
    Docs(docs::Args),

    /// Open Steel Browser GitHub repository
    Star(star::Args),

    /// Open Steel Discord server
    Support(support::Args),

    /// Display and modify current settings
    Settings(settings::Args),

    /// Manage named Steel browser profiles
    Profile {
        #[command(subcommand)]
        command: profile::Command,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Daemon { session_id } => {
            let cdp_url = std::env::var("STEEL_DAEMON_CDP_URL")
                .map_err(|_| anyhow::anyhow!("Missing STEEL_DAEMON_CDP_URL"))?;
            crate::browser::daemon::server::run(session_id, cdp_url).await
        }
        Command::Scrape(args) => scrape::run(args).await,
        Command::Screenshot(args) => screenshot::run(args).await,
        Command::Pdf(args) => pdf::run(args).await,
        Command::Browser { command } => browser::run(command).await,
        Command::Login(args) => login::run(args).await,
        Command::Logout(args) => logout::run(args).await,
        Command::Credentials { command } => credentials::run(command).await,
        Command::Dev { command } => dev::run(command).await,
        Command::Forge(args) => forge::run(args).await,
        Command::Run(args) => run::run(args).await,
        Command::Config(args) => config::run(args).await,
        Command::Update(args) => update::run(args).await,
        Command::Cache(args) => cache::run(args).await,
        Command::Docs(args) => docs::run(args).await,
        Command::Star(args) => star::run(args).await,
        Command::Support(args) => support::run(args).await,
        Command::Settings(args) => settings::run(args).await,
        Command::Profile { command } => profile::run(command).await,
    }
}
