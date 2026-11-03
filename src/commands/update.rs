use std::time::Duration;

use clap::Parser;

use crate::status;
use crate::util::output;
use crate::util::version::is_newer;

#[derive(Parser)]
pub struct Args {
    /// Force update even if already on latest version
    #[arg(short, long)]
    pub force: bool,

    /// Only check for updates without installing
    #[arg(short, long)]
    pub check: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    status!("Checking for updates...");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let current_version = env!("CARGO_PKG_VERSION");

    // Check GitHub releases for latest version
    let latest_version = match client
        .get("https://api.github.com/repos/steel-dev/cli/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", format!("steel-cli/{current_version}"))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let data: serde_json::Value = resp.json().await.unwrap_or_default();
            data.get("tag_name")
                .and_then(|v| v.as_str())
                .map(|s| s.strip_prefix('v').unwrap_or(s).to_string())
        }
        _ => None,
    };

    let Some(latest) = latest_version else {
        status!("Could not check for updates. Please check your network connection.");
        return Ok(());
    };

    let newer = is_newer(&latest, current_version);

    if args.check {
        if newer {
            status!("Update available: v{current_version} -> v{latest}");
            status!("Run `steel update` to install.");
        } else {
            status!("v{current_version} (latest)");
        }
        return Ok(());
    }

    if !newer && !args.force {
        status!("v{current_version} (latest)");
        return Ok(());
    }

    status!("Updating v{current_version} -> v{latest}...");

    // Use cargo-dist installer
    let stdout = if output::is_json() {
        std::process::Stdio::from(std::io::stderr())
    } else {
        std::process::Stdio::inherit()
    };
    let status = std::process::Command::new("sh")
        .args([
            "-c",
            "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh | sh",
        ])
        .stdout(stdout)
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {
            status!("Updated to v{latest}");
        }
        _ => {
            anyhow::bail!(
                "Update failed. Try manually:\n  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh | sh"
            );
        }
    }

    Ok(())
}
