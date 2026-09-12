use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::api::checkpoints::RestoreCheckpoint;
use crate::api::client::SteelClient;
use crate::commands::computer;
use crate::status;
use crate::util::{api, output};

#[derive(Subcommand)]
pub enum Command {
    /// List checkpoints
    List,

    /// Get one checkpoint
    Get(IdArgs),

    /// Delete a checkpoint
    Delete(IdArgs),

    /// Start a new computer from a checkpoint
    Restore(RestoreArgs),
}

impl Command {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Get(_) => "get",
            Self::Delete(_) => "delete",
            Self::Restore(_) => "restore",
        }
    }
}

#[derive(Parser)]
pub struct IdArgs {
    /// Checkpoint ID
    pub checkpoint_id: String,
}

#[derive(Parser)]
pub struct RestoreArgs {
    /// Checkpoint ID
    pub checkpoint_id: String,

    /// Delete the computer after this many seconds of running time
    #[arg(long = "timeout", value_name = "SECONDS")]
    pub timeout_seconds: Option<u32>,

    /// Pause instead of deleting when the timeout is reached
    #[arg(long = "auto-pause")]
    pub auto_pause: bool,

    /// Pause after this many seconds without incoming traffic (0 disables it)
    #[arg(long = "idle-timeout", value_name = "SECONDS")]
    pub idle_timeout_seconds: Option<u32>,

    /// Wait until the computer is running
    #[arg(long)]
    pub wait: bool,

    /// Make the new computer the default for other commands
    #[arg(long = "use")]
    pub use_as_default: bool,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::List => run_list().await,
        Command::Get(args) => run_get(args).await,
        Command::Delete(args) => run_delete(args).await,
        Command::Restore(args) => run_restore(args).await,
    }
}

pub fn status_of(checkpoint: &Value) -> &str {
    checkpoint["status"].as_str().unwrap_or("unknown")
}

async fn run_list() -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client.list_checkpoints(&base_url, mode, &auth).await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        print_checkpoints(&data);
    }
    Ok(())
}

async fn run_get(args: IdArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .get_checkpoint(&base_url, mode, &auth, &args.checkpoint_id)
        .await?;
    output::success_data(data);
    Ok(())
}

async fn run_delete(args: IdArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .delete_checkpoint(&base_url, mode, &auth, &args.checkpoint_id)
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Deleted {}.", args.checkpoint_id);
    }
    Ok(())
}

async fn run_restore(args: RestoreArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let request = RestoreCheckpoint {
        timeout_seconds: args.timeout_seconds,
        auto_pause: args.auto_pause.then_some(true),
        idle_timeout_seconds: args.idle_timeout_seconds,
    };
    let restored = client
        .restore_checkpoint(&base_url, mode, &auth, &args.checkpoint_id, &request)
        .await?;
    let id = computer::id_of(&restored)?;
    let computer = if args.wait {
        status!("Restored {id}, waiting until it is running.");
        computer::wait_for(&client, &base_url, mode, &auth, &id, "running").await?
    } else {
        restored
    };
    if args.use_as_default {
        computer::remember_computer(Some(&id))?;
    }
    if output::is_json() {
        output::success_data(computer);
    } else {
        println!("{id} is {}.", computer::status_of(&computer));
        if args.use_as_default {
            println!("It is now the default computer.");
        }
    }
    Ok(())
}

pub async fn wait_until_ready(
    client: &SteelClient,
    base_url: &str,
    mode: crate::config::settings::ApiMode,
    auth: &crate::config::auth::Auth,
    id: &str,
) -> Result<Value> {
    let started = std::time::Instant::now();
    loop {
        let checkpoint = client.get_checkpoint(base_url, mode, auth, id).await?;
        match status_of(&checkpoint) {
            "ready" => return Ok(checkpoint),
            "creating" => {}
            status => bail!("{id} is {status}, it will not become ready."),
        }
        if started.elapsed() > computer::WAIT_TIMEOUT {
            bail!(
                "{id} is still creating after {}s.",
                computer::WAIT_TIMEOUT.as_secs()
            );
        }
        tokio::time::sleep(computer::WAIT_POLL_INTERVAL).await;
    }
}

fn print_checkpoints(data: &Value) {
    let checkpoints = data["checkpoints"].as_array().cloned().unwrap_or_default();
    if checkpoints.is_empty() {
        println!("No checkpoints.");
        return;
    }
    let rows: Vec<[String; 5]> = checkpoints
        .iter()
        .map(|checkpoint| {
            [
                checkpoint["id"].as_str().unwrap_or("").to_string(),
                status_of(checkpoint).to_string(),
                checkpoint["name"].as_str().unwrap_or("-").to_string(),
                checkpoint["computerId"].as_str().unwrap_or("-").to_string(),
                match checkpoint["sizeBytes"].as_u64() {
                    Some(bytes) => format!("{:.1}G", bytes as f64 / 1_073_741_824.0),
                    None => "-".to_string(),
                },
            ]
        })
        .collect();
    let header = ["ID", "STATUS", "NAME", "COMPUTER", "SIZE"];
    let widths: Vec<usize> = (0..header.len())
        .map(|column| {
            rows.iter()
                .map(|row| row[column].len())
                .chain(std::iter::once(header[column].len()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |cells: [&str; 5]| {
        cells
            .iter()
            .enumerate()
            .map(|(column, cell)| format!("{cell:<width$}", width = widths[column]))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_string()
    };
    println!("{}", line(header));
    for row in &rows {
        println!("{}", line([&row[0], &row[1], &row[2], &row[3], &row[4]]));
    }
}
