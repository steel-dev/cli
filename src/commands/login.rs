use clap::Parser;
use dialoguer::{Confirm, Input, Select};

use crate::api::device_auth::{self, OrgPayload};
use crate::commands::projects;
use crate::config;
use crate::config::auth;
use crate::config::settings::{OrgInfo, read_config_from, write_config_to};
use crate::status;
use crate::util::{api, output};

#[derive(Parser)]
pub struct Args {}

/// Result of authenticating: the account token and the org it belongs to.
pub struct AuthOutcome {
    pub account_token: String,
    pub device_name: String,
    pub org: OrgPayload,
}

impl AuthOutcome {
    pub fn org_label(&self) -> String {
        self.org.name.clone()
    }
}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    if auth::resolve_account_token().is_some() {
        status!("You are already logged in. Run `steel logout` first to switch accounts.");
        return Ok(());
    }

    let (mode, base_url) = api::resolve();

    if interactive() {
        let choice = Select::new()
            .with_prompt("Welcome to Steel! Would you like to log in?")
            .items([
                "Use Steel locally (no account)",
                "Log in or create an account",
            ])
            .default(1)
            .interact()?;

        if choice == 0 {
            print_local_guidance();
            return Ok(());
        }
    }

    let outcome = authenticate(&base_url).await?;

    let project = projects::ensure_active_project(
        &base_url,
        mode,
        &outcome.account_token,
        &outcome.device_name,
    )
    .await?;

    crate::telemetry::track_event("login_completed", serde_json::Map::new());

    status!("Logged in to {}.", outcome.org_label());
    if let Some(name) = project.name.as_deref() {
        status!("Active project: {name}");
    }
    status!("Saved credentials to {}", config::config_path().display());

    Ok(())
}

/// Run the device authorization flow and persist the account token.
///
/// Shared by `steel login` and `steel init`. Prints the verification URL + code
/// even in non-interactive / JSON mode so the user can always complete the flow.
pub async fn authenticate(base_url: &str) -> anyhow::Result<AuthOutcome> {
    let is_interactive = interactive();

    let device_name = if is_interactive {
        Input::new()
            .with_prompt("Device name")
            .default(default_device_name())
            .interact_text()?
    } else {
        default_device_name()
    };

    let authz = device_auth::request_device_authorization(base_url).await?;

    // Essential instructions: printed directly so they are visible regardless of
    // output mode (status! is suppressed when JSON output is active).
    eprintln!();
    eprintln!("Visit {} to finish logging in.", authz.verification_uri);
    eprintln!(
        "Enter this code (expires in {} seconds): {}",
        authz.expires_in, authz.user_code
    );
    eprintln!();

    let open_browser = if is_interactive {
        Confirm::new()
            .with_prompt("Open the browser?")
            .default(true)
            .interact()?
    } else {
        false
    };

    if open_browser {
        if let Err(e) = open::that(&authz.verification_uri_complete) {
            eprintln!("Could not open the browser automatically: {e}");
            eprintln!(
                "Open this URL manually: {}",
                authz.verification_uri_complete
            );
        }
    } else if !is_interactive {
        eprintln!(
            "Open this URL to authorize: {}",
            authz.verification_uri_complete
        );
    }

    status!("Waiting for authorization...");

    let token = device_auth::poll_for_token(
        base_url,
        &authz.device_code,
        &device_name,
        authz.interval,
        authz.expires_in,
    )
    .await?;

    save_account(&token.access_token, &device_name, &token.org)?;

    Ok(AuthOutcome {
        account_token: token.access_token,
        device_name,
        org: token.org,
    })
}

fn save_account(token: &str, device_name: &str, org: &OrgPayload) -> anyhow::Result<()> {
    let path = config::config_path();
    let mut cfg = read_config_from(&path).unwrap_or_default();
    cfg.account_token = Some(token.to_string());
    cfg.name = Some(device_name.to_string());
    cfg.org = Some(OrgInfo {
        id: org.id.clone(),
        slug: org.slug.clone(),
        name: Some(org.name.clone()),
    });
    cfg.instance = Some("cloud".to_string());
    write_config_to(&path, &cfg)?;
    Ok(())
}

fn interactive() -> bool {
    output::is_tty() && !output::is_json()
}

fn print_local_guidance() {
    status!("Running Steel locally - no account needed.");
    status!("");
    status!("  steel dev install     Install the local Steel Browser runtime");
    status!("  steel dev start       Start the local runtime");
    status!("  steel --local ...     Run any command against the local runtime");
    status!("");
    status!("Run `steel login` again any time to connect a cloud account.");
}

/// Best-effort device name from the host machine. Avoids `unsafe`/extra deps by
/// shelling out to `hostname`, then falling back to env vars.
fn default_device_name() -> String {
    if let Ok(output) = std::process::Command::new("hostname").output()
        && output.status.success()
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return name;
        }
    }

    for key in ["HOSTNAME", "COMPUTERNAME", "USER", "USERNAME"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    "Steel CLI".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_device_name_is_non_empty() {
        assert!(!default_device_name().is_empty());
    }

    #[test]
    fn org_label_uses_name() {
        let outcome = AuthOutcome {
            account_token: "ste-cli-x".into(),
            device_name: "Mac".into(),
            org: OrgPayload {
                id: "org-1".into(),
                slug: Some("acme".into()),
                name: "Acme".into(),
            },
        };
        assert_eq!(outcome.org_label(), "Acme");
    }
}
