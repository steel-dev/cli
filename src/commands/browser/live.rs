use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::api::session::Session;
use crate::util::output;

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let target = super::existing_session(session)?;

    let url = match target.live_url {
        Some(url) => Some(url),
        None => {
            let remote = SteelClient::new()?
                .at(&target.api_url, target.mode)
                .cli_get_session(&target.id)
                .await?;
            Session::parse(remote)?.viewer_url(target.mode)
        }
    };

    match url {
        Some(url) => output::success(json!(url), &format!("{url}\n")),
        None => anyhow::bail!("Session \"{}\" has no live viewer URL.", target.name),
    }

    Ok(())
}
