use std::time::Duration;

use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::projects::{environment_label, parse_projects, resolve_current_project};
use crate::config::auth::{self, Auth};
use crate::config::settings::{ApiMode, read_config};

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

    print_project_info(&auth).await;

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

/// Print the project (environment) the API key is scoped to.
/// Best-effort: skipped in local mode, without a key, or on any fetch error,
/// so `steel config` stays usable offline.
async fn print_project_info(auth: &Auth) {
    let (mode, base_url) = crate::util::api::resolve();
    if mode != ApiMode::Cloud || auth.api_key.is_none() {
        return;
    }
    let Ok(client) = SteelClient::new() else {
        return;
    };

    let fetch = client.get_projects(&base_url, mode, auth);
    let Ok(Ok(data)) = tokio::time::timeout(Duration::from_secs(2), fetch).await else {
        return;
    };

    let projects = parse_projects(&data);
    if let Some(project) = resolve_current_project(&projects) {
        println!("project: {} ({})", project.name, project.slug);
        println!("environment: {}", environment_label(project.is_production));
    }
}
