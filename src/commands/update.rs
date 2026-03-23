use clap::Parser;

use crate::status;

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
        status!("Could not check for updates. Please check your network connection.");
        return Ok(());
    };

    let is_newer = is_version_newer(&latest, current_version);

    if args.check {
        if is_newer {
            status!("Update available: v{current_version} -> v{latest}");
            status!("Run `steel update` to install.");
        } else {
            status!("v{current_version} (latest)");
        }
        return Ok(());
    }

    if !is_newer && !args.force {
        status!("v{current_version} (latest)");
        return Ok(());
    }

    status!("Updating v{current_version} -> v{latest}...");

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

/// Compare semver strings. Returns true if `latest` is strictly newer than `current`.
fn is_version_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    for i in 0..l.len().max(c.len()) {
        let lv = l.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if lv > cv {
            return true;
        }
        if lv < cv {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_patch() {
        assert!(is_version_newer("0.3.1", "0.3.0"));
    }

    #[test]
    fn newer_minor() {
        assert!(is_version_newer("0.4.0", "0.3.9"));
    }

    #[test]
    fn newer_major() {
        assert!(is_version_newer("1.0.0", "0.99.99"));
    }

    #[test]
    fn same_version() {
        assert!(!is_version_newer("0.3.0", "0.3.0"));
    }

    #[test]
    fn older_version() {
        assert!(!is_version_newer("0.2.0", "0.3.0"));
    }

    #[test]
    fn multi_digit_segments() {
        // This was the bug: string comparison gives wrong result
        assert!(is_version_newer("0.10.0", "0.9.1"));
    }
}
