use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::get_viewer_url;
use crate::config::auth;
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let target = super::existing_session(session)?;

    let url = match target.live_url {
        Some(url) => Some(url),
        None => {
            let client = SteelClient::new()?;
            let remote = client
                .get_session(
                    &target.api_url,
                    target.mode,
                    &target.id,
                    &auth::resolve_auth(),
                )
                .await?;
            get_viewer_url(&remote, target.mode, &target.id)
        }
    };

    match url {
        Some(url) => output::success(json!(url), &format!("{url}\n")),
        None => anyhow::bail!("Session \"{}\" has no live viewer URL.", target.name),
    }

    Ok(())
}
