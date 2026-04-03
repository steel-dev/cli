use std::time::{Duration, Instant};

use clap::Parser;

use crate::config;
use crate::config::auth;
use crate::config::settings::{Config, read_config_from, write_config_to};
use crate::status;

#[derive(Parser)]
pub struct Args {}

const DEVICE_CODE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

pub async fn run(_args: Args) -> anyhow::Result<()> {
    // Check if already logged in
    let existing_auth = auth::resolve_auth();
    if existing_auth.api_key.is_some() {
        status!("You are already logged in.");
        return Ok(());
    }

    let (api_key, name) = device_code_flow().await?;

    save_api_key(&api_key, &name)?;

    status!("Authentication successful! Your API key has been saved.");

    Ok(())
}

async fn device_code_flow() -> anyhow::Result<(String, String)> {
    let client = reqwest::Client::new();

    // Step 1: Request device code
    let resp = client
        .post(config::DEVICE_AUTH_URL)
        .json(&serde_json::json!({ "client_id": "cli" }))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "Failed to start device authorization: {} {}",
            resp.status().as_u16(),
            resp.status().canonical_reason().unwrap_or("")
        );
    }

    let data: serde_json::Value = resp.json().await?;

    let device_code = data["device_code"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing device_code in response"))?
        .to_string();
    let user_code = data["user_code"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing user_code in response"))?;
    let verification_uri = data["verification_uri"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing verification_uri in response"))?;
    let verification_uri_complete = data["verification_uri_complete"].as_str();
    let expires_in = data["expires_in"].as_u64().unwrap_or(900);
    let interval = data["interval"].as_u64().unwrap_or(5);

    // Step 2: Display instructions
    status!("Visit: {verification_uri}");
    status!("Enter code: {user_code}");

    // Try to open browser to the pre-filled URL
    if let Some(uri) = verification_uri_complete {
        if let Err(e) = open::that(uri) {
            status!("Warning: could not open browser automatically: {e}");
            status!("Please open the URL above manually.");
        }
    }

    // Step 3: Poll for token
    let mut poll_interval = Duration::from_secs(interval);
    let deadline = Instant::now() + Duration::from_secs(expires_in).min(DEVICE_CODE_TIMEOUT);

    loop {
        tokio::time::sleep(poll_interval).await;

        if Instant::now() > deadline {
            anyhow::bail!("Device code expired. Please try again.");
        }

        let resp = client
            .post(config::DEVICE_TOKEN_URL)
            .json(&serde_json::json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": device_code,
                "client_id": "cli",
            }))
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(_) => {
                // Connection error — exponential backoff, capped at 60s
                poll_interval = (poll_interval * 2).min(Duration::from_secs(60));
                continue;
            }
        };

        // Reset backoff on successful HTTP response
        poll_interval = Duration::from_secs(interval);

        let body: serde_json::Value = resp.json().await?;

        // Success: api_key present
        if let Some(api_key) = body.get("api_key").and_then(|v| v.as_str()) {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("CLI");
            return Ok((api_key.to_string(), name.to_string()));
        }

        // Error handling per RFC 8628
        match body.get("error").and_then(|v| v.as_str()) {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                poll_interval += Duration::from_secs(5);
                continue;
            }
            Some("expired_token") => anyhow::bail!("Device code expired. Please try again."),
            Some("access_denied") => anyhow::bail!("Authorization denied by user."),
            Some(other) => anyhow::bail!("Authentication failed: {other}"),
            None => anyhow::bail!("Unexpected response from server"),
        }
    }
}

fn save_api_key(api_key: &str, name: &str) -> anyhow::Result<()> {
    let config_dir = config::config_dir();
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config::config_path_in(&config_dir);

    let mut config = read_config_from(&config_path).unwrap_or_else(|_| Config::default());
    config.api_key = Some(api_key.to_string());
    config.name = Some(name.to_string());

    write_config_to(&config_path, &config)?;

    Ok(())
}
