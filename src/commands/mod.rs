pub mod browser;
pub mod cache;
pub mod completion;
pub mod config;
pub mod credentials;
pub mod describe;
pub mod dev;
pub mod doctor;
pub mod forge;
pub mod init;
pub mod login;
pub mod logout;
pub mod pdf;
pub mod profile;
pub mod scrape;
pub mod screenshot;
pub mod sessions;
pub mod skills;
pub mod update;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, builder::styling::Styles};

const ENVIRONMENT: &[(&str, &str)] = &[
    ("STEEL_API_KEY", "API key for Steel (overrides login)"),
    ("STEEL_API_URL", "Steel API endpoint"),
    ("STEEL_BROWSER_API_URL", "Steel Browser API endpoint"),
    ("STEEL_LOCAL_API_URL", "Local runtime API endpoint"),
    ("STEEL_CONFIG_DIR", "Custom config directory"),
    ("STEEL_TELEMETRY_HOST", "PostHog ingest host override"),
    (
        "STEEL_TELEMETRY_DISABLED",
        "Disable built-in telemetry for this machine/session",
    ),
    (
        "STEEL_FORCE_TTY",
        "Force text output when piped (disable auto-JSON)",
    ),
    ("NO_COLOR", "Disable colored output"),
];

const EXAMPLES: &[&str] = &[
    "steel init",
    "steel init --agent",
    "steel scrape https://example.com",
    "steel scrape https://example.com --format html,markdown",
    "steel screenshot https://example.com -f",
    "steel browser start --session my-session",
    "steel browser navigate https://example.com",
    "steel browser snapshot -i",
    "steel browser click e5",
    "steel browser fill e3 \"user@example.com\"",
    "steel browser click \"button#submit\" --diff",
    "steel browser type \"textarea\" \"Hello world\" --clear",
    "steel browser get text \"h1\"",
    "steel browser is visible \".modal\"",
    "steel browser wait --text \"Success\" --timeout 5000",
    "steel browser batch \"fill e3 Seoul\" \"click e5\" \"snapshot -i\"",
    "steel browser screenshot --full",
    "steel browser eval \"document.title\"",
    "steel browser tab new https://example.com",
    "steel browser stop --session my-session",
];

fn long_help(root: &clap::Command) -> String {
    let environment: Vec<String> = ENVIRONMENT
        .iter()
        .map(|(name, about)| format!("  {name:<38}{about}"))
        .collect();
    let examples: Vec<String> = EXAMPLES.iter().map(|e| format!("  {e}")).collect();
    format!(
        "All Commands:\n{}\n\nEnvironment:\n{}\n\nExamples:\n{}",
        describe::overview(root),
        environment.join("\n"),
        examples.join("\n")
    )
}

pub fn parse() -> Cli {
    let root = Cli::command();
    let help = long_help(&root);
    let mut matches = root.after_long_help(help).get_matches();
    Cli::from_arg_matches_mut(&mut matches).unwrap_or_else(|err| err.exit())
}

#[derive(Parser)]
#[command(
    name = "steel",
    version,
    about = "Steel CLI - browser automation for AI agents",
    long_about = "Steel CLI - browser automation for AI agents",
    styles = Styles::plain(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output all results as JSON (structured output for automation)
    #[arg(long, global = true)]
    pub json: bool,

    /// Skip update check (reserved for future use)
    #[arg(long, global = true, hide = true)]
    pub no_update_check: bool,

    /// Use local Steel runtime instead of cloud
    #[arg(long, global = true)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long, global = true)]
    pub api_url: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Scrape webpage content
    Scrape(scrape::Args),

    /// Capture a screenshot of a webpage
    Screenshot(screenshot::Args),

    /// Generate a PDF from a webpage
    Pdf(pdf::Args),

    /// Browser session management and automation
    Browser(browser::BrowserArgs),

    /// Cloud session management and debugging
    Sessions {
        #[command(subcommand)]
        command: sessions::Command,
    },

    /// One-command onboarding: login + verify + install agent skills
    Init(init::Args),

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

    /// Scaffold a new project from a template
    Forge(forge::Args),

    /// Show current configuration
    Config(config::Args),

    /// Update to the latest version
    Update(update::Args),

    /// Manage Steel CLI cache
    Cache(cache::Args),

    /// Manage named Steel browser profiles
    Profile {
        #[command(subcommand)]
        command: profile::Command,
    },

    /// Describe commands and parameters
    Describe(describe::Args),

    /// Check environment, auth, and connectivity
    Doctor(doctor::Args),

    /// Manage Steel agent skills
    Skills(skills::Args),

    /// Generate a shell completion script
    Completion(completion::Args),
}

fn telemetry_command_path(command: &Command) -> Option<String> {
    match command {
        Command::Scrape(_) => Some("scrape".to_string()),
        Command::Screenshot(_) => Some("screenshot".to_string()),
        Command::Pdf(_) => Some("pdf".to_string()),
        Command::Browser(args) => Some(format!("browser.{}", args.command.telemetry_name())),
        Command::Sessions { command } => Some(format!("sessions.{}", command.telemetry_name())),
        Command::Init(_) => Some("init".to_string()),
        Command::Login(_) => Some("login".to_string()),
        Command::Logout(_) => Some("logout".to_string()),
        Command::Credentials { command } => {
            Some(format!("credentials.{}", command.telemetry_name()))
        }
        Command::Dev { command } => Some(format!("dev.{}", command.telemetry_name())),
        Command::Forge(_) => Some("forge".to_string()),
        Command::Config(_) => Some("config".to_string()),
        Command::Update(_) => Some("update".to_string()),
        Command::Cache(_) => Some("cache".to_string()),
        Command::Profile { command } => Some(format!("profile.{}", command.telemetry_name())),
        Command::Describe(_) => Some("describe".to_string()),
        Command::Doctor(_) => Some("doctor".to_string()),
        Command::Skills(args) => Some(format!("skills.{}", skills::telemetry_name(&args.command))),
        Command::Completion(_) => Some("completion".to_string()),
    }
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let telemetry_context =
        telemetry_command_path(&cli.command).map(|path| crate::telemetry::command_context(&path));

    crate::util::output::init(cli.json);
    crate::util::api::init(cli.local, cli.api_url);
    crate::telemetry::init_from_env();

    if let Some(ref context) = telemetry_context {
        crate::telemetry::track_command_started(context);
    }

    let started_at = std::time::Instant::now();

    let result = match cli.command {
        Command::Scrape(args) => scrape::run(args).await,
        Command::Screenshot(args) => screenshot::run(args).await,
        Command::Pdf(args) => pdf::run(args).await,
        Command::Browser(args) => browser::run(args).await,
        Command::Sessions { command } => sessions::run(command).await,
        Command::Init(args) => init::run(args).await,
        Command::Login(args) => login::run(args).await,
        Command::Logout(args) => logout::run(args).await,
        Command::Credentials { command } => credentials::run(command).await,
        Command::Dev { command } => dev::run(command).await,
        Command::Forge(args) => forge::run(args).await,
        Command::Config(args) => config::run(args).await,
        Command::Update(args) => update::run(args).await,
        Command::Cache(args) => cache::run(args).await,
        Command::Profile { command } => profile::run(command).await,
        Command::Describe(args) => describe::run(args).await,
        Command::Doctor(args) => doctor::run(args).await,
        Command::Skills(args) => skills::run(args.command).await,
        Command::Completion(args) => completion::run(args).await,
    };

    if let Some(ref context) = telemetry_context {
        let duration = started_at.elapsed();
        match &result {
            Ok(()) => crate::telemetry::track_command_completed(context, duration),
            Err(err) => crate::telemetry::track_command_failed(context, duration, err),
        }
    }

    crate::telemetry::flush_best_effort().await;

    result
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use tokio::sync::Mutex;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    async fn capture_events(server: &MockServer) -> Vec<serde_json::Value> {
        let mut out = Vec::new();
        for request in server.received_requests().await.unwrap_or_default() {
            let payload: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
            if let Some(batch) = payload.get("batch").and_then(|v| v.as_array()) {
                out.extend(batch.iter().cloned());
            }
        }
        out
    }

    #[test]
    fn long_help_examples_parse() {
        for example in EXAMPLES {
            let args = browser::batch::split_command_str(example);
            if let Err(err) = Cli::try_parse_from(&args) {
                panic!("{example}: {err}");
            }
        }
    }

    #[tokio::test]
    async fn successful_command_emits_started_and_completed_events() {
        let _guard = env_lock().lock().await;
        crate::telemetry::reset_for_test();

        let temp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;
        crate::telemetry::set_test_override(temp.path(), &server.uri());

        Mock::given(method("POST"))
            .and(path("/batch/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let result = run(Cli {
            command: Command::Cache(cache::Args { clean: false }),
            json: false,
            no_update_check: true,
            local: false,
            api_url: None,
        })
        .await;

        assert!(result.is_ok());

        let events = capture_events(&server).await;
        crate::telemetry::reset_for_test();

        assert_eq!(events.len(), 3);
        assert!(
            events
                .iter()
                .any(|event| event["event"] == "install_created")
        );
        assert!(events.iter().any(|event| {
            event["event"] == "command_started" && event["properties"]["command_path"] == "cache"
        }));
        assert!(events.iter().any(|event| {
            event["event"] == "command_completed"
                && event["properties"]["command_path"] == "cache"
                && event["properties"]["success"] == true
                && event["properties"]["mode"] == "cloud"
        }));
    }

    #[tokio::test]
    async fn failing_command_emits_started_and_failed_events() {
        let _guard = env_lock().lock().await;
        crate::telemetry::reset_for_test();

        let temp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;
        crate::telemetry::set_test_override(temp.path(), &server.uri());

        Mock::given(method("POST"))
            .and(path("/batch/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let result = run(Cli {
            command: Command::Browser(browser::BrowserArgs {
                session: Some("named-session".into()),
                diff: false,
                command: browser::Command::Stop(browser::stop::Args { all: true }),
            }),
            json: false,
            no_update_check: true,
            local: false,
            api_url: None,
        })
        .await;

        assert!(result.is_err());

        let events = capture_events(&server).await;
        crate::telemetry::reset_for_test();

        assert_eq!(events.len(), 3);
        assert!(
            events
                .iter()
                .any(|event| event["event"] == "install_created")
        );
        assert!(events.iter().any(|event| {
            event["event"] == "command_started"
                && event["properties"]["command_path"] == "browser.stop"
        }));
        assert!(events.iter().any(|event| {
            event["event"] == "command_failed"
                && event["properties"]["command_path"] == "browser.stop"
                && event["properties"]["success"] == false
                && event["properties"]["error_class"] == "internal_error"
        }));
    }

    #[tokio::test]
    async fn telemetry_delivery_failure_does_not_fail_command() {
        let _guard = env_lock().lock().await;
        crate::telemetry::reset_for_test();

        let temp = tempfile::TempDir::new().unwrap();
        crate::telemetry::set_test_override(temp.path(), "http://127.0.0.1:1");

        let result = run(Cli {
            command: Command::Cache(cache::Args { clean: false }),
            json: false,
            no_update_check: true,
            local: false,
            api_url: None,
        })
        .await;

        crate::telemetry::reset_for_test();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn install_created_emits_only_once_per_config_dir() {
        let _guard = env_lock().lock().await;
        crate::telemetry::reset_for_test();

        let temp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;
        crate::telemetry::set_test_override(temp.path(), &server.uri());

        Mock::given(method("POST"))
            .and(path("/batch/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let first = run(Cli {
            command: Command::Cache(cache::Args { clean: false }),
            json: false,
            no_update_check: true,
            local: false,
            api_url: None,
        })
        .await;
        assert!(first.is_ok());

        crate::telemetry::reset_for_test();
        crate::telemetry::set_test_override(temp.path(), &server.uri());

        let second = run(Cli {
            command: Command::Cache(cache::Args { clean: false }),
            json: false,
            no_update_check: true,
            local: false,
            api_url: None,
        })
        .await;
        assert!(second.is_ok());

        let events = capture_events(&server).await;
        crate::telemetry::reset_for_test();

        let install_created_count = events
            .iter()
            .filter(|event| event["event"] == "install_created")
            .count();
        let command_completed_count = events
            .iter()
            .filter(|event| event["event"] == "command_completed")
            .count();

        assert_eq!(install_created_count, 1);
        assert_eq!(command_completed_count, 2);
    }
}
