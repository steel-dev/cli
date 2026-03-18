use clap::{Parser, Subcommand};
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::protocol::{DaemonCommand, SessionInfo};
use crate::browser::lifecycle;
use crate::util::{api, output};

#[derive(Subcommand)]
pub enum Command {
    /// Solve a CAPTCHA on the current page
    Solve(SolveArgs),

    /// Check CAPTCHA status
    Status(StatusArgs),
}

#[derive(Parser)]
pub struct SolveArgs {
    /// Explicit session ID (overrides --session name lookup)
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
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Explicit session ID (overrides --session name lookup)
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
}

pub async fn run(command: Command, session: Option<&str>) -> anyhow::Result<()> {
    match command {
        Command::Solve(args) => run_solve(args, session).await,
        Command::Status(args) => run_status(args, session).await,
    }
}

/// Resolve session_id: explicit --session-id flag, or query daemon.
async fn resolve_session_id(
    explicit_session_id: Option<&str>,
    session: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(id) = explicit_session_id {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let session_name = session.unwrap_or("default");
    let mut client = DaemonClient::connect(session_name).await.map_err(|_| {
        anyhow::anyhow!(
            "No running session \"{session_name}\". \
             Pass `--session-id`, or start a session with `steel browser start`."
        )
    })?;

    let data = client.send(DaemonCommand::GetSessionInfo).await?;
    let info: SessionInfo = serde_json::from_value(data)?;
    Ok(info.session_id)
}

async fn run_solve(args: SolveArgs, session: Option<&str>) -> anyhow::Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    let session_id = resolve_session_id(args.session_id.as_deref(), session).await?;

    let result = lifecycle::solve_captcha(
        &client,
        &base_url,
        mode,
        &auth,
        &session_id,
        args.page_id.as_deref(),
        args.url.as_deref(),
        args.task_id.as_deref(),
    )
    .await?;

    if output::is_json() {
        let mut data = json!({
            "sessionId": result.session_id,
            "mode": result.mode.to_string(),
            "success": result.success,
        });
        if let Some(ref msg) = result.message {
            data["message"] = json!(msg);
        }
        output::success_data(data);
    } else {
        println!("session_id: {}", result.session_id);
        println!("mode: {}", result.mode);
        println!("success: {}", result.success);
        if let Some(ref msg) = result.message {
            println!("message: {msg}");
        }
    }

    if !result.success {
        anyhow::bail!("CAPTCHA solve failed");
    }

    Ok(())
}

async fn run_status(args: StatusArgs, session: Option<&str>) -> anyhow::Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    let session_id = resolve_session_id(args.session_id.as_deref(), session).await?;

    let result = lifecycle::captcha_status(
        &client,
        &base_url,
        mode,
        &auth,
        &session_id,
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
        anyhow::bail!("CAPTCHA status: {}", result.status);
    }

    Ok(())
}
