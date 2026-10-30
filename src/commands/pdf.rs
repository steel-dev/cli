use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::top_level::{Target, get_hosted_url};
use crate::util::output;
use crate::util::url::resolve_tool_url;

#[derive(Parser)]
pub struct Args {
    /// Target URL to generate PDF from
    pub url: Option<String>,

    /// Delay before generating in milliseconds
    #[arg(short, long)]
    pub delay: Option<u64>,

    /// Use a Steel-managed residential proxy
    #[arg(long)]
    pub use_proxy: bool,

    /// Region identifier for request execution
    #[arg(short, long)]
    pub region: Option<String>,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let url = resolve_tool_url(args.url.as_deref())?;

    let target = Target {
        url: &url,
        delay: args.delay,
        use_proxy: args.use_proxy,
        region: args.region.as_deref(),
    };
    let response = SteelClient::new()?.pdf(&target).await?;

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
