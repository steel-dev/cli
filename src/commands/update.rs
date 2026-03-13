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

    // Check npm registry for latest version
    let response = client
        .get("https://registry.npmjs.org/@steel-dev/cli/latest")
        .header("Accept", "application/json")
        .send()
        .await;

    let current_version = env!("CARGO_PKG_VERSION");

    let latest_version = match response {
        Ok(resp) if resp.status().is_success() => {
            let data: serde_json::Value = resp.json().await.unwrap_or_default();
            data.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }
        _ => None,
    };

    let Some(latest) = latest_version else {
        println!("Could not check for updates. Please check your network connection.");
        return Ok(());
    };

    if args.check {
        if latest != current_version {
            println!("Update available!");
            println!("Current: v{current_version}");
            println!("Latest: v{latest}");
            println!("Run `steel update` to update to the latest version.");
        } else {
            println!("Current version: v{current_version}");
            println!("You're already on the latest version!");
        }
        return Ok(());
    }

    if latest == current_version && !args.force {
        println!("Current version: v{current_version}");
        println!("You're already on the latest version!");
        return Ok(());
    }

    // Update via npm
    let status = std::process::Command::new("npm")
        .args(["install", "-g", "@steel-dev/cli@latest"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Updated from v{current_version} to v{latest}");
        }
        _ => {
            anyhow::bail!("Failed to update. Try running: npm install -g @steel-dev/cli@latest");
        }
    }

    Ok(())
}
