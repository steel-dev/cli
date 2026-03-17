use clap::Parser;

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
    println!("Checking for updates...");

    let client = reqwest::Client::new();
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
        println!("Could not check for updates. Please check your network connection.");
        return Ok(());
    };

    if args.check {
        if latest != current_version {
            println!("Update available: v{current_version} -> v{latest}");
            println!("Run `steel update` to install.");
        } else {
            println!("v{current_version} (latest)");
        }
        return Ok(());
    }

    if latest == current_version && !args.force {
        println!("v{current_version} (latest)");
        return Ok(());
    }

    println!("Updating v{current_version} -> v{latest}...");

    // Use cargo-dist installer
    let status = std::process::Command::new("sh")
        .args([
            "-c",
            "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh | sh",
        ])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Updated to v{latest}");
        }
        _ => {
            anyhow::bail!(
                "Update failed. Try manually:\n  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh | sh"
            );
        }
    }

    Ok(())
}
