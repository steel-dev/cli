use std::time::Duration;

use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::api::projects::{Project, environment_label, parse_projects, resolve_current_project};
use crate::config::auth;
use crate::config::settings::{ApiMode, read_config};
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let auth = auth::resolve_auth();
    let config = read_config().ok();
    let (mode, api_url) = crate::util::api::resolve();

    if !auth.has_credentials() && config.is_none() {
        output::success(
            json!({ "loggedIn": false, "apiUrl": api_url, "mode": mode.to_string() }),
            "You are not logged in. Please run `steel login` to authenticate.\n",
        );
        return Ok(());
    }

    let mut data = json!({
        "loggedIn": auth.has_credentials(),
        "apiUrl": api_url,
        "mode": mode.to_string(),
    });
    let mut lines = Vec::new();

    if let Some(ref key) = auth.api_key {
        let masked = mask(key);
        lines.push(format!("apiKey: {masked}"));
        lines.push(format!("source: {}", auth.source));
        data["apiKey"] = json!(masked);
        data["source"] = json!(auth.source.to_string());
    }
    if auth.access_token.is_some() {
        lines.push(format!("source: {} (token redacted)", auth.source));
        data["source"] = json!(auth.source.to_string());
    }
    lines.push(format!("apiUrl: {api_url} ({mode})"));

    if let Some(project) = current_project().await {
        let environment = environment_label(project.is_production);
        lines.push(format!("project: {} ({})", project.name, project.slug));
        lines.push(format!("environment: {environment}"));
        data["project"] = json!({
            "name": project.name,
            "slug": project.slug,
            "environment": environment,
        });
    }

    if let Some(url) = config
        .as_ref()
        .and_then(|cfg| cfg.browser.as_ref())
        .and_then(|browser| browser.api_url.as_deref())
    {
        lines.push(format!("browser.apiUrl: {url}"));
        data["browserApiUrl"] = json!(url);
    }

    if let Some(ref cfg) = config {
        lines.push(format!("telemetry.disabled: {}", cfg.telemetry_disabled()));
        data["telemetryDisabled"] = json!(cfg.telemetry_disabled());
    }

    output::success(data, &format!("{}\n", lines.join("\n")));
    Ok(())
}

fn mask(key: &str) -> String {
    if key.chars().count() > 7 {
        format!("{}...", key.chars().take(7).collect::<String>())
    } else {
        key.to_string()
    }
}

/// Fetch the project (environment) the API key is scoped to.
/// Best-effort: skipped in local mode, without a key, or on any fetch error,
/// so `steel config` stays usable offline.
async fn current_project() -> Option<Project> {
    let client = SteelClient::new().ok()?;
    if client.mode() != ApiMode::Cloud || !client.auth().has_credentials() {
        return None;
    }

    let fetch = client.get_projects();
    let Ok(Ok(data)) = tokio::time::timeout(Duration::from_secs(2), fetch).await else {
        return None;
    };

    let projects = parse_projects(&data);
    resolve_current_project(&projects).cloned()
}
