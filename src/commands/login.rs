use crate::api::client::ApiError;
use crate::config::settings::ApiMode;
use crate::config::{auth, device};
use crate::util::output;
use clap::Parser;
use std::io::IsTerminal;

#[derive(Parser, Default)]
pub struct Args {
    /// Continue the saved authorization request; do not issue a new code
    #[arg(long)]
    pub resume: bool,
    /// Show the handoff without waiting; check an existing request once if due
    #[arg(long)]
    pub no_wait: bool,
    /// Do not open a browser on this host
    #[arg(long)]
    pub no_browser: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    output::success_data(authorize(args).await?);
    Ok(())
}

/// Authorize without reporting, so callers such as `init` own their single result.
/// The returned `status` is `authenticated` or `pending`.
pub async fn authorize(args: Args) -> anyhow::Result<serde_json::Value> {
    let existing = auth::resolve_auth();
    if existing.api_key.is_some() {
        return Ok(serde_json::json!({"status":"authenticated","method":"api_key"}));
    }
    let interactive = std::io::stdout().is_terminal();
    let complete = device::login(
        args.no_wait || !interactive,
        args.no_browser || !interactive,
        args.resume,
    )
    .await?;
    if complete {
        let settings = device::saved_settings()?.expect("saved login");
        device::access_token_for(&settings.api_base_url).await?;
        if existing.access_token.is_none() {
            crate::telemetry::track_event("login_completed", serde_json::Map::new());
        }
    }
    Ok(serde_json::json!({
        "status":if complete {"authenticated"} else {"pending"},
        "method":"device",
        "next_action":if complete {"Run steel doctor --preflight to verify product API access"} else {"Approve the displayed code, then run steel login --resume or repeat steel init"}
    }))
}

const WAITING_FOR_APPROVAL: &str = "Waiting for approval. This command continues once you approve.";

fn same_transport(base_url: &str, device_api: &str) -> bool {
    base_url.trim_end_matches('/') == device_api.trim_end_matches('/')
}

pub async fn ensure_signed_in() -> anyhow::Result<()> {
    let (mode, base_url, credentials) = crate::util::api::resolve_with_auth();
    if mode != ApiMode::Cloud || credentials.has_credentials() {
        return Ok(());
    }
    let saved = device::saved_settings().ok().flatten();
    let Ok(settings) = device::Settings::resolve(saved.as_ref()) else {
        return Ok(());
    };
    if !same_transport(&base_url, &settings.api_base_url) {
        return Ok(());
    }
    let interactive =
        !output::is_json() && std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let outcome = if interactive {
        device::authorize(true, true, false, Some(WAITING_FOR_APPROVAL)).await
    } else {
        device::authorize(false, false, false, None).await
    }
    .map_err(|err| {
        ApiError::DeviceAuth(err.context("Not signed in to Steel, and sign-in failed"))
    })?;
    match outcome {
        device::Outcome::Authorized => {
            let mut properties = serde_json::Map::new();
            properties.insert("trigger".into(), serde_json::json!("command"));
            crate::telemetry::track_event("login_completed", properties);
            Ok(())
        }
        device::Outcome::Pending(handoff) => Err(handoff.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_only_targets_the_transport_the_login_is_for() {
        let api = "https://api.steel.dev/v1";
        assert!(same_transport(api, api));
        assert!(same_transport("https://api.steel.dev/v1/", api));
        assert!(!same_transport("https://api.staging.steel.dev/v1", api));
    }
}
