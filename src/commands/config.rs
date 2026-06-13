use clap::Parser;

use crate::api::projects::environment_label;
use crate::config::auth;
use crate::config::settings::{Config, read_config};

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let auth = auth::resolve_auth();
    let config = read_config().ok();

    if auth.api_key.is_none() && config.is_none() {
        println!("You are not logged in. Please run `steel login` to authenticate.");
        return Ok(());
    }

    if let Some(ref key) = auth.api_key {
        let masked = if key.len() > 7 {
            format!("{}...", &key[..7])
        } else {
            key.clone()
        };
        println!("apiKey: {masked}");
        println!("source: {}", auth.source);
    }

    print_project_info(config.as_ref());

    if let Some(ref cfg) = config
        && let Some(ref browser) = cfg.browser
        && let Some(ref url) = browser.api_url
    {
        println!("browser.apiUrl: {url}");
    }

    if let Some(ref cfg) = config {
        println!("telemetry.disabled: {}", cfg.telemetry_disabled());
    }

    Ok(())
}

/// Print the active project (and its environment) recorded at login / selection.
fn print_project_info(config: Option<&Config>) {
    let Some(project) = config.and_then(|c| c.project.as_ref()) else {
        return;
    };
    let name = project.name.as_deref().unwrap_or(&project.id);
    println!(
        "project: {name} ({})",
        project.slug.as_deref().unwrap_or("")
    );
    println!("environment: {}", environment_label(project.is_production));
}
