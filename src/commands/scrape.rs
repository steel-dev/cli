use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::top_level::{get_scrape_output_text, parse_scrape_formats};
use crate::util::{api, output};
use crate::util::url::resolve_tool_url;

#[derive(Parser)]
pub struct Args {
    /// Target URL to scrape
    pub url: Option<String>,

    /// Comma-separated output formats: html, readability, cleaned_html, markdown
    #[arg(long)]
    pub format: Option<String>,

    /// Delay before scraping in milliseconds
    #[arg(short, long)]
    pub delay: Option<u64>,

    /// Include a generated PDF in the response
    #[arg(long)]
    pub pdf: bool,

    /// Include a generated screenshot in the response
    #[arg(long)]
    pub screenshot: bool,

    /// Use a Steel-managed residential proxy
    #[arg(long)]
    pub use_proxy: bool,

    /// Region identifier for request execution
    #[arg(short, long)]
    pub region: Option<String>,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let url = resolve_tool_url(args.url.as_deref())?;

    let formats = match &args.format {
        Some(f) => parse_scrape_formats(f).map_err(|e| anyhow::anyhow!(e))?,
        None => vec!["markdown".into()],
    };

    let (mode, base_url, auth) = api::resolve_with_auth();

    let client = SteelClient::new()?;
    let response = client
        .scrape(
            &base_url,
            mode,
            &auth,
            &url,
            &formats,
            args.delay,
            args.pdf,
            args.screenshot,
            args.use_proxy,
            args.region.as_deref(),
        )
        .await?;

    if output::is_json() {
        output::success_data(response);
    } else {
        match get_scrape_output_text(&response, &formats) {
            Some(text) => println!("{text}"),
            None => println!("{}", serde_json::to_string_pretty(&response)?),
        }
    }

    Ok(())
}
