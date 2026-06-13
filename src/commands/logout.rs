use clap::Parser;

use crate::api::device_auth;
use crate::config;
use crate::config::settings::{read_config_from, write_config_to};
use crate::status;
use crate::util::{api, style};

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let config_path = config::config_path();

    let mut cfg = match read_config_from(&config_path) {
        Ok(c) => c,
        Err(_) => {
            status!("You are not logged in.");
            return Ok(());
        }
    };

    if cfg.account_token.is_none() && cfg.api_key.is_none() {
        status!("You are not logged in.");
        return Ok(());
    }

    let org_label = cfg.org.as_ref().and_then(|o| o.name.clone());

    // Best-effort server-side revocation of the account token.
    if let Some(token) = cfg.account_token.clone() {
        let (mode, base_url) = api::resolve();
        if let Err(e) = device_auth::revoke_account_token(&base_url, mode, &token).await {
            status!(
                "{} Could not revoke the CLI token on the server: {e}",
                style::bang()
            );
        }
    }

    cfg.account_token = None;
    cfg.api_key = None;
    cfg.name = None;
    cfg.org = None;
    cfg.project = None;

    write_config_to(&config_path, &cfg)?;

    match org_label {
        Some(name) => status!("{} Logged out of {name}.", style::tick()),
        None => status!("{} Logged out.", style::tick()),
    }
    status!("{}", style::dim("Run steel login to sign back in."));

    Ok(())
}
