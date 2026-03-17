use clap::Parser;
use serde_json::json;

use crate::api::client::SteelClient;
use crate::browser::lifecycle::get_live_url;
use crate::config::session_state::SessionStatePaths;
use crate::util::{api, output};

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args, session: Option<&str>) -> anyhow::Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();

    let client = SteelClient::new()?;
    let paths = SessionStatePaths::default_paths();

    let live_url = get_live_url(
        &client,
        &base_url,
        mode,
        &auth,
        &paths,
        session,
    )
    .await?;

    match live_url {
        Some(url) => {
            output::success(json!(url), &format!("{url}\n"));
        }
        None => {
            let msg = match session {
                Some(name) => format!(
                    "No live session found for \"{name}\". \
                     Start one with `steel browser start --session {name}`."
                ),
                None => "No active live session found. Start one with `steel browser start`.".to_string(),
            };
            anyhow::bail!("{msg}");
        }
    }

    Ok(())
}
