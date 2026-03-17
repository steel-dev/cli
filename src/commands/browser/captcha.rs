use clap::{Parser, Subcommand};
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::lifecycle;
use crate::config::auth;
use crate::config::session_state::SessionStatePaths;
use crate::config::settings::{ApiMode, EnvVars};
use crate::util::output;

#[derive(Subcommand)]
pub enum Command {
    /// Solve a CAPTCHA on the current page
    Solve(SolveArgs),

    /// Check CAPTCHA status
    Status(StatusArgs),
}

#[derive(Parser)]
pub struct SolveArgs {
    /// Session name
    #[arg(short, long)]
    pub session: Option<String>,

    /// Session ID
    #[arg(long)]
    pub session_id: Option<String>,

    /// Page ID
    #[arg(long)]
    pub page_id: Option<String>,

    /// Page URL for targeted CAPTCHA solving
    #[arg(long)]
    pub url: Option<String>,

    /// CAPTCHA task ID for targeted solving
    #[arg(long)]
    pub task_id: Option<String>,

    /// Use local runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Session name
    #[arg(short, long)]
    pub session: Option<String>,

    /// Session ID
    #[arg(long)]
    pub session_id: Option<String>,

    /// Page ID
    #[arg(long)]
    pub page_id: Option<String>,

    /// Wait for terminal status
    #[arg(short, long)]
    pub wait: bool,

    /// Timeout in milliseconds for --wait mode
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Poll interval in milliseconds for --wait mode
    #[arg(long)]
    pub interval: Option<u64>,

    /// Use local runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,
}

fn resolve_common(local: bool, api_url: Option<&str>) -> (ApiMode, auth::Auth, String) {
    let mode = ApiMode::resolve(local, api_url);
    let auth = auth::resolve_auth();
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
    let local_config_url = config.as_ref().and_then(|c| c.local_api_url());
    let base_url = mode.resolve_base_url(api_url, &env_vars, local_config_url);
    (mode, auth, base_url)
}

pub async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Solve(args) => run_solve(args).await,
        Command::Status(args) => run_status(args).await,
    }
}

async fn run_solve(args: SolveArgs) -> anyhow::Result<()> {
    let (mode, auth, base_url) = resolve_common(args.local, args.api_url.as_deref());
    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    let result = lifecycle::solve_captcha(
        &client,
        &base_url,
        mode,
        &auth,
        &paths,
        args.session_id.as_deref(),
        args.session.as_deref(),
        args.page_id.as_deref(),
        args.url.as_deref(),
        args.task_id.as_deref(),
    )
    .await?;

    if output::is_json() {
        let mut data = json!({
            "sessionId": result.session_id,
            "mode": format!("{:?}", result.mode),
            "success": result.success,
        });
        if let Some(ref msg) = result.message {
            data["message"] = json!(msg);
        }
        output::success_data(data);
    } else {
        println!("session_id: {}", result.session_id);
        println!("mode: {:?}", result.mode);
        println!("success: {}", result.success);
        if let Some(ref msg) = result.message {
            println!("message: {msg}");
        }
    }

    if !result.success {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_status(args: StatusArgs) -> anyhow::Result<()> {
    let (mode, auth, base_url) = resolve_common(args.local, args.api_url.as_deref());
    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    let result = lifecycle::captcha_status(
        &client,
        &base_url,
        mode,
        &auth,
        &paths,
        args.session_id.as_deref(),
        args.session.as_deref(),
        args.page_id.as_deref(),
        args.wait,
        args.timeout,
        args.interval,
    )
    .await?;

    if output::is_json() {
        let mut data = json!({
            "status": result.status,
        });
        if !result.types.is_empty() {
            data["types"] = json!(result.types);
        }
        output::success_data(data);
    } else {
        let type_suffix = if result.types.is_empty() {
            String::new()
        } else {
            format!(" {}", result.types.join(","))
        };
        println!("{}{type_suffix}", result.status);
    }

    if result.status != "solved" && result.status != "none" {
        std::process::exit(1);
    }

    Ok(())
}
