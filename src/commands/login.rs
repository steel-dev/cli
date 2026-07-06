use clap::Parser;
use dialoguer::{Confirm, Input, Select};

use crate::api::device_auth::{self, OrgPayload};
use crate::commands::projects;
use crate::config;
use crate::config::auth;
use crate::config::settings::{OrgInfo, read_config_from, write_config_to};
use crate::status;
use crate::util::{api, output, style};

#[derive(Parser)]
pub struct Args {}

/// Result of authenticating: the account token and the org it belongs to.
pub struct AuthOutcome {
    pub account_token: String,
    pub device_name: String,
    pub org: OrgPayload,
    pub tos_required: bool,
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
        let choice = Select::with_theme(&*style::prompt_theme())
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

    enforce_tos(&base_url, mode, &outcome, false).await?;

    let project = projects::ensure_active_project(
        &base_url,
        mode,
        &outcome.account_token,
        &outcome.device_name,
    )
    .await?;

    crate::telemetry::track_event("login_completed", serde_json::Map::new());

    status!("{} Logged in to {}.", style::tick(), outcome.org_label());
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
        Input::with_theme(&*style::prompt_theme())
            .with_prompt("Device name")
            .default(default_device_name())
            .interact_text()?
    } else {
        default_device_name()
    };

    let authz = device_auth::request_device_authorization(base_url).await?;

    // Essential instructions: printed directly so they are visible regardless of
    // output mode (status! is suppressed when JSON output is active). Humans get
    // a spotlighted block; non-interactive/CI gets plain, parseable lines.
    if is_interactive {
        print_auth_prompt(&authz);
    } else {
        eprintln!();
        eprintln!("Visit {} to finish logging in.", authz.verification_uri);
        eprintln!(
            "Enter this code (expires in {} seconds): {}",
            authz.expires_in, authz.user_code
        );
        eprintln!();
    }

    let open_browser = if is_interactive {
        Confirm::with_theme(&*style::prompt_theme())
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

    let spinner = style::spinner("Waiting for authorization…");
    let token = device_auth::poll_for_token(
        base_url,
        &authz.device_code,
        &device_name,
        authz.interval,
        authz.expires_in,
    )
    .await;
    spinner.finish_and_clear();
    let token = token?;

    save_account(&token.access_token, &device_name, &token.org)?;

    Ok(AuthOutcome {
        account_token: token.access_token,
        device_name,
        org: token.org,
        tos_required: token.tos_required,
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

pub async fn enforce_tos(
    base_url: &str,
    mode: crate::config::settings::ApiMode,
    outcome: &AuthOutcome,
    auto_accept: bool,
) -> anyhow::Result<()> {
    if !outcome.tos_required {
        return Ok(());
    }

    if accept_tos_decision(auto_accept)? {
        if let Err(e) =
            device_auth::record_tos_acceptance(base_url, mode, &outcome.account_token).await
        {
            let _ = device_auth::revoke_account_token(base_url, mode, &outcome.account_token).await;
            clear_saved_account()?;
            anyhow::bail!(
                "Could not record your Terms of Service acceptance ({e}). Run `steel login` to try again."
            );
        }
        return Ok(());
    }

    let _ = device_auth::revoke_account_token(base_url, mode, &outcome.account_token).await;
    clear_saved_account()?;
    anyhow::bail!(
        "You must accept the Terms of Service to continue. Run `steel login` to try again."
    );
}

fn accept_tos_decision(auto_accept: bool) -> anyhow::Result<bool> {
    let url = config::TOS_URL;

    if auto_accept {
        status!("Accepting the Terms of Service at {url}.");
        return Ok(true);
    }

    if !interactive() {
        status!("By continuing you agree to the Terms of Service at {url}.");
        return Ok(true);
    }

    Ok(Confirm::with_theme(&*style::prompt_theme())
        .with_prompt(format!("Do you agree to our Terms of Service at {url}?"))
        .default(true)
        .interact()?)
}

/// Clear the saved account token + derived credentials (used when ToS is declined).
fn clear_saved_account() -> anyhow::Result<()> {
    let path = config::config_path();
    let mut cfg = read_config_from(&path).unwrap_or_default();
    cfg.account_token = None;
    cfg.api_key = None;
    cfg.api_key_id = None;
    cfg.name = None;
    cfg.org = None;
    cfg.project = None;
    write_config_to(&path, &cfg)?;
    Ok(())
}

fn interactive() -> bool {
    output::is_tty() && !output::is_json()
}

/// Pretty, interactive-only device-login instructions: a styled URL and the
/// one-time code spotlighted in a box so it's easy to spot and copy.
fn print_auth_prompt(authz: &device_auth::DeviceAuthorization) {
    eprintln!();
    eprintln!(
        "  To finish logging in, visit  {}",
        style::link(&authz.verification_uri)
    );
    eprintln!(
        "  and enter this code {}:",
        style::dim(&format!(
            "(expires in {})",
            humanize_duration(authz.expires_in)
        ))
    );
    eprintln!();
    eprintln!("{}", style::code_box(&authz.user_code, "  "));
    eprintln!();
}

/// Render a second count as a friendly duration, e.g. 900 -> "15 minutes".
fn humanize_duration(secs: u64) -> String {
    if secs >= 60 {
        let mins = secs / 60;
        let unit = if mins == 1 { "minute" } else { "minutes" };
        if secs.is_multiple_of(60) {
            format!("{mins} {unit}")
        } else {
            format!("~{mins} {unit}")
        }
    } else {
        let unit = if secs == 1 { "second" } else { "seconds" };
        format!("{secs} {unit}")
    }
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
    fn humanize_duration_formats() {
        assert_eq!(humanize_duration(900), "15 minutes");
        assert_eq!(humanize_duration(60), "1 minute");
        assert_eq!(humanize_duration(90), "~1 minute");
        assert_eq!(humanize_duration(150), "~2 minutes");
        assert_eq!(humanize_duration(45), "45 seconds");
        assert_eq!(humanize_duration(1), "1 second");
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
            tos_required: false,
        };
        assert_eq!(outcome.org_label(), "Acme");
    }
}
