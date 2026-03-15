use std::process::Command as ProcessCommand;

use clap::{Parser, Subcommand};

use crate::config;

#[derive(Subcommand)]
pub enum Command {
    /// Install the local Steel Browser runtime
    Install(InstallArgs),

    /// Start the local runtime containers
    Start(StartArgs),

    /// Stop the local runtime containers
    Stop(StopArgs),
}

#[derive(Parser)]
pub struct InstallArgs {
    /// Git repository URL for local Steel Browser runtime
    #[arg(long)]
    pub repo_url: Option<String>,

    /// Enable verbose git command output
    #[arg(short = 'V', long)]
    pub verbose: bool,
}

#[derive(Parser)]
pub struct StartArgs {
    /// API port for local Steel Browser runtime
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Enable verbose Docker command output
    #[arg(short = 'V', long)]
    pub verbose: bool,

    /// Only verify Docker availability and exit
    #[arg(short, long)]
    pub docker_check: bool,
}

#[derive(Parser)]
pub struct StopArgs {
    /// Enable verbose Docker command output
    #[arg(short = 'V', long)]
    pub verbose: bool,
}

fn repo_path() -> std::path::PathBuf {
    config::config_dir().join("steel-browser")
}

pub async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Install(args) => run_install(args),
        Command::Start(args) => run_start(args),
        Command::Stop(args) => run_stop(args),
    }
}

fn run_install(args: InstallArgs) -> anyhow::Result<()> {
    let repo_url = args
        .repo_url
        .as_deref()
        .unwrap_or(config::REPO_URL);
    let path = repo_path();

    if path.exists() {
        println!("Local Steel Browser runtime already installed.");
        println!("repo_path: {}", path.display());
        println!("repo_url: {repo_url}");
        return Ok(());
    }

    let mut cmd = ProcessCommand::new("git");
    cmd.args(["clone", repo_url, &path.to_string_lossy()]);

    if !args.verbose {
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to clone repository.");
    }

    println!("Local Steel Browser runtime installed.");
    println!("repo_path: {}", path.display());
    println!("repo_url: {repo_url}");

    Ok(())
}

fn run_start(args: StartArgs) -> anyhow::Result<()> {
    if args.docker_check {
        return if is_docker_running() {
            println!("Docker is running.");
            Ok(())
        } else {
            anyhow::bail!("Docker is not running.");
        };
    }

    let path = repo_path();
    if !path.exists() {
        anyhow::bail!(
            "Local Steel Browser runtime not installed. Run `steel dev install` first."
        );
    }

    let port = args.port.unwrap_or(3000);

    let mut cmd = ProcessCommand::new("docker");
    cmd.args(["compose", "up", "-d"])
        .current_dir(&path)
        .env("STEEL_API_PORT", port.to_string());

    if !args.verbose {
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to start local Steel Browser runtime.");
    }

    println!("Local Steel Browser runtime started.");
    println!("repo_path: {}", path.display());
    println!("api_port: {port}");

    Ok(())
}

fn run_stop(args: StopArgs) -> anyhow::Result<()> {
    let path = repo_path();
    if !path.exists() {
        anyhow::bail!(
            "Local Steel Browser runtime not installed. Run `steel dev install` first."
        );
    }

    let mut cmd = ProcessCommand::new("docker");
    cmd.args(["compose", "down"]).current_dir(&path);

    if !args.verbose {
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to stop local Steel Browser runtime.");
    }

    println!("Local Steel Browser runtime stopped.");
    println!("repo_path: {}", path.display());

    Ok(())
}

fn is_docker_running() -> bool {
    ProcessCommand::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
