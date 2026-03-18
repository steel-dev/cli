use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::top_level::get_hosted_url;
use crate::util::url::resolve_tool_url;
use crate::util::{api, output};

#[derive(Parser)]
pub struct Args {
    /// Target URL to capture
    pub url: Option<String>,

    /// Delay before capturing in milliseconds
    #[arg(short, long)]
    pub delay: Option<u64>,

    /// Capture a full-page screenshot
    #[arg(short = 'f', long)]
    pub full_page: bool,

    /// Use a Steel-managed residential proxy
    #[arg(long)]
    pub use_proxy: bool,

    /// Region identifier for request execution
    #[arg(short, long)]
    pub region: Option<String>,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let url = resolve_tool_url(args.url.as_deref())?;

    let (mode, base_url, auth) = api::resolve_with_auth();

    let client = SteelClient::new()?;
    let response = client
        .screenshot(
            &base_url,
            mode,
            &auth,
            &url,
            args.delay,
            args.full_page,
            args.use_proxy,
            args.region.as_deref(),
        )
        .await?;

    if output::is_json() {
        output::success_data(response);
    } else {
        match get_hosted_url(&response) {
            Some(hosted_url) => println!("{hosted_url}"),
            None => println!("{}", serde_json::to_string_pretty(&response)?),
        }
    }

    Ok(())
}
