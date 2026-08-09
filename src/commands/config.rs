use clap::Parser;

use crate::api::projects::environment_label;
use crate::config::auth;
use crate::config::settings::{Config, read_config};
use crate::util::style;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let auth = auth::resolve_auth();
    let config = read_config().ok();

    if auth.api_key.is_none() && config.is_none() {
        println!("You are not logged in. Please run `steel login` to authenticate.");
        return Ok(());
    }

    if style::color_enabled() {
        println!("{}", style::bold("Configuration"));
        println!();
    }

    if let Some(ref key) = auth.api_key {
        let masked = if key.len() > 7 {
            format!("{}...", &key[..7])
        } else {
            key.clone()
        };
        row("API key", "apiKey", &masked);
        row("Source", "source", &auth.source.to_string());
    }

    print_project_info(config.as_ref());

    if let Some(ref cfg) = config
        && let Some(ref browser) = cfg.browser
        && let Some(ref url) = browser.api_url
    {
        row("Browser API", "browser.apiUrl", url);
    }

    if let Some(ref cfg) = config {
        let disabled = cfg.telemetry_disabled();
        if style::color_enabled() {
            let state = if disabled { "disabled" } else { "enabled" };
            row("Telemetry", "telemetry.disabled", state);
        } else {
            // Keep the machine-parseable `key: value` form when piped.
            println!("telemetry.disabled: {disabled}");
        }
    }

    Ok(())
}

/// Print one config row. In an interactive terminal the key is a dim, aligned
/// label; when piped it stays the original `key: value` form so scripts that
/// grep the output keep working.
fn row(label: &str, key: &str, value: &str) {
    if style::color_enabled() {
        println!("  {}  {value}", style::dim(&format!("{label:<13}")));
    } else {
        println!("{key}: {value}");
    }
}

/// Print the active project (and its environment) recorded at login / selection.
fn print_project_info(config: Option<&Config>) {
    let Some(project) = config.and_then(|c| c.project.as_ref()) else {
        return;
    };
    let name = project.name.as_deref().unwrap_or(&project.id);
    let slug = project.slug.as_deref().unwrap_or("");
    row("Project", "project", &format!("{name} ({slug})"));
    row(
        "Environment",
        "environment",
        environment_label(project.is_production),
    );
}
