pub mod exec;
pub mod ssh;
mod wsio;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::api::client::SteelClient;
use crate::api::computers::CreateComputer;
use crate::config::settings::{self, ComputerConfig};
use crate::status;
use crate::util::{api, output};

const WAIT_POLL_INTERVAL: Duration = Duration::from_secs(1);
const WAIT_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Subcommand)]
pub enum Command {
    /// Create a computer
    Create(CreateArgs),

    /// List computers
    List,

    /// Get one computer
    Get(IdArgs),

    /// Delete a computer
    Delete(IdArgs),

    /// Pause a running computer
    Pause(IdArgs),

    /// Resume a paused computer
    Resume(ResumeArgs),

    /// Remember a computer as the default for other commands
    Use(UseArgs),

    /// Run one command in a computer
    Exec(exec::Args),

    /// Open an SSH session to a computer
    Ssh(ssh::Args),
}

impl Command {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::Create(_) => "create",
            Self::List => "list",
            Self::Get(_) => "get",
            Self::Delete(_) => "delete",
            Self::Pause(_) => "pause",
            Self::Resume(_) => "resume",
            Self::Use(_) => "use",
            Self::Exec(_) => "exec",
            Self::Ssh(_) => "ssh",
        }
    }
}

#[derive(Parser)]
pub struct CreateArgs {
    /// Template name
    #[arg(long)]
    pub template: String,

    /// Region, for example us-east
    #[arg(long)]
    pub region: Option<String>,

    /// Number of vCPUs
    #[arg(long)]
    pub vcpu: Option<u32>,

    /// Memory in MiB
    #[arg(long = "memory", value_name = "MIB")]
    pub memory_mib: Option<u32>,

    /// Disk in MiB
    #[arg(long = "disk", value_name = "MIB")]
    pub disk_mib: Option<u32>,

    /// Stop the computer after this many seconds of running time
    #[arg(long = "timeout", value_name = "SECONDS")]
    pub timeout_seconds: Option<u32>,

    /// Pause instead of stopping when the timeout is reached
    #[arg(long = "auto-pause")]
    pub auto_pause: bool,

    /// Wait until the computer is running
    #[arg(long)]
    pub wait: bool,

    /// Make the new computer the default for other commands
    #[arg(long = "use")]
    pub use_as_default: bool,
}

#[derive(Parser)]
pub struct IdArgs {
    /// Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
    pub computer_id: Option<String>,
}

#[derive(Parser)]
pub struct ResumeArgs {
    /// Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
    pub computer_id: Option<String>,

    /// Wait until the computer is running
    #[arg(long)]
    pub wait: bool,
}

#[derive(Parser)]
pub struct UseArgs {
    /// Computer ID to remember
    pub computer_id: Option<String>,

    /// Forget the remembered computer
    #[arg(long, conflicts_with = "computer_id")]
    pub clear: bool,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Create(args) => run_create(args).await,
        Command::List => run_list().await,
        Command::Get(args) => run_get(args).await,
        Command::Delete(args) => run_delete(args).await,
        Command::Pause(args) => run_pause(args).await,
        Command::Resume(args) => run_resume(args).await,
        Command::Use(args) => run_use(args),
        Command::Exec(args) => exec::run(args).await,
        Command::Ssh(args) => ssh::run(args).await,
    }
}

pub fn resolve_computer_id(explicit: Option<&str>) -> Result<String> {
    let env = std::env::var("STEEL_COMPUTER_ID").ok();
    let stored = settings::read_config()
        .ok()
        .and_then(|config| config.default_computer_id().map(str::to_string));
    choose_computer_id(explicit, env.as_deref(), stored.as_deref())
}

pub fn choose_computer_id(
    explicit: Option<&str>,
    env: Option<&str>,
    stored: Option<&str>,
) -> Result<String> {
    for candidate in [explicit, env, stored] {
        if let Some(id) = candidate.map(str::trim).filter(|id| !id.is_empty()) {
            return Ok(id.to_string());
        }
    }
    bail!("No computer given. Pass an id, set STEEL_COMPUTER_ID, or run `steel computer use <id>`.")
}

fn remember_computer(id: Option<&str>) -> Result<()> {
    let mut config = settings::read_config().unwrap_or_default();
    config.computer = id.map(|id| ComputerConfig {
        default_id: Some(id.to_string()),
    });
    settings::write_config(&config).context("Failed to save the default computer")
}

fn status_of(computer: &Value) -> &str {
    computer["status"].as_str().unwrap_or("unknown")
}

fn id_of(computer: &Value) -> Result<String> {
    computer["id"]
        .as_str()
        .map(str::to_string)
        .context("the API returned a computer without an id")
}

async fn run_create(args: CreateArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let request = CreateComputer {
        template: args.template,
        region: args.region,
        vcpu: args.vcpu,
        memory_mib: args.memory_mib,
        disk_mib: args.disk_mib,
        timeout_seconds: args.timeout_seconds,
        auto_pause: args.auto_pause.then_some(true),
    };
    let created = client
        .create_computer(&base_url, mode, &auth, &request)
        .await?;
    let id = id_of(&created)?;
    let computer = if args.wait {
        status!("Created {id}, waiting until it is running.");
        wait_for(&client, &base_url, mode, &auth, &id, "running").await?
    } else {
        created
    };
    if args.use_as_default {
        remember_computer(Some(&id))?;
    }
    if output::is_json() {
        output::success_data(computer);
    } else {
        println!("{id} is {}.", status_of(&computer));
        if args.use_as_default {
            println!("It is now the default computer.");
        }
    }
    Ok(())
}

async fn run_list() -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client.list_computers(&base_url, mode, &auth).await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        print_computers(&data);
    }
    Ok(())
}

async fn run_get(args: IdArgs) -> Result<()> {
    let id = resolve_computer_id(args.computer_id.as_deref())?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client.get_computer(&base_url, mode, &auth, &id).await?;
    output::success_data(data);
    Ok(())
}

async fn run_delete(args: IdArgs) -> Result<()> {
    let id = resolve_computer_id(args.computer_id.as_deref())?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client.delete_computer(&base_url, mode, &auth, &id).await?;
    if settings::read_config()
        .ok()
        .and_then(|config| config.default_computer_id().map(str::to_string))
        .is_some_and(|default| default == id)
    {
        remember_computer(None)?;
    }
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Deleted {id}.");
    }
    Ok(())
}

async fn run_pause(args: IdArgs) -> Result<()> {
    let id = resolve_computer_id(args.computer_id.as_deref())?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client.pause_computer(&base_url, mode, &auth, &id).await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("{id} is {}.", status_of(&data));
    }
    Ok(())
}

async fn run_resume(args: ResumeArgs) -> Result<()> {
    let id = resolve_computer_id(args.computer_id.as_deref())?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let resumed = client.resume_computer(&base_url, mode, &auth, &id).await?;
    let data = if args.wait {
        wait_for(&client, &base_url, mode, &auth, &id, "running").await?
    } else {
        resumed
    };
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("{id} is {}.", status_of(&data));
    }
    Ok(())
}

fn run_use(args: UseArgs) -> Result<()> {
    if args.clear {
        remember_computer(None)?;
        if output::is_json() {
            output::success_silent();
        } else {
            println!("Forgot the default computer.");
        }
        return Ok(());
    }
    let Some(id) = args
        .computer_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        bail!("Give a computer id, or --clear to forget the current one.");
    };
    remember_computer(Some(id))?;
    if output::is_json() {
        output::success_data(serde_json::json!({ "computerId": id }));
    } else {
        println!("{id} is now the default computer.");
    }
    Ok(())
}

async fn wait_for(
    client: &SteelClient,
    base_url: &str,
    mode: crate::config::settings::ApiMode,
    auth: &crate::config::auth::Auth,
    id: &str,
    target: &str,
) -> Result<Value> {
    let started = Instant::now();
    loop {
        let computer = client.get_computer(base_url, mode, auth, id).await?;
        let status = status_of(&computer);
        if status == target {
            return Ok(computer);
        }
        if matches!(status, "failed" | "deleted" | "deleting" | "stopped") {
            bail!("{id} is {status}, it will not become {target}.");
        }
        if started.elapsed() > WAIT_TIMEOUT {
            bail!("{id} is still {status} after {}s.", WAIT_TIMEOUT.as_secs());
        }
        tokio::time::sleep(WAIT_POLL_INTERVAL).await;
    }
}

fn print_computers(data: &Value) {
    let computers = data["computers"].as_array().cloned().unwrap_or_default();
    if computers.is_empty() {
        println!("No computers.");
        return;
    }
    let rows: Vec<[String; 6]> = computers
        .iter()
        .map(|computer| {
            [
                computer["id"].as_str().unwrap_or("").to_string(),
                status_of(computer).to_string(),
                computer["region"].as_str().unwrap_or("-").to_string(),
                computer["template"].as_str().unwrap_or("-").to_string(),
                format!(
                    "{}/{}",
                    computer["vcpu"].as_u64().unwrap_or(0),
                    computer["memoryMib"].as_u64().unwrap_or(0)
                ),
                computer["statusChangedAt"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            ]
        })
        .collect();
    let header = ["ID", "STATUS", "REGION", "TEMPLATE", "VCPU/MIB", "SINCE"];
    let widths: Vec<usize> = (0..header.len())
        .map(|column| {
            rows.iter()
                .map(|row| row[column].len())
                .chain(std::iter::once(header[column].len()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |cells: [&str; 6]| {
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
        println!(
            "{}",
            line([&row[0], &row[1], &row[2], &row[3], &row[4], &row[5]])
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_id_wins_and_blank_ids_are_ignored() {
        assert_eq!(
            choose_computer_id(Some(" cmp_a "), Some("cmp_env"), Some("cmp_cfg")).unwrap(),
            "cmp_a"
        );
        assert_eq!(
            choose_computer_id(Some("  "), Some("cmp_env"), Some("cmp_cfg")).unwrap(),
            "cmp_env"
        );
        assert_eq!(
            choose_computer_id(None, Some(""), Some("cmp_cfg")).unwrap(),
            "cmp_cfg"
        );
        assert!(choose_computer_id(None, None, None).is_err());
    }
}
