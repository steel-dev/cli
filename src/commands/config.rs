use clap::Parser;

use crate::config::auth;
use crate::config::settings::read_config;

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

    if let Some(ref cfg) = config {
        if let Some(ref browser) = cfg.browser {
            if let Some(ref url) = browser.api_url {
                println!("browser.apiUrl: {url}");
            }
        }
    }

    Ok(())
}
