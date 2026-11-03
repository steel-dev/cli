use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::top_level::{Target, get_scrape_output_text, parse_scrape_formats};
use crate::util::output;
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

    let target = Target {
        url: &url,
        delay: args.delay,
        use_proxy: args.use_proxy,
        region: args.region.as_deref(),
    };
    let response = SteelClient::new()?
        .scrape(&target, &formats, args.pdf, args.screenshot)
        .await?;

    if output::is_json() {
        output::success_data(response);
        return Ok(());
    }

    match get_scrape_output_text(&response, &formats) {
        Some(text) => println!("{text}"),
        None => println!("{}", serde_json::to_string_pretty(&response)?),
    }
    for (requested, key) in [(args.screenshot, "screenshot"), (args.pdf, "pdf")] {
        if requested && let Some(url) = asset_url(&response[key]) {
            println!("\n{key}: {url}");
        }
    }

    Ok(())
}

fn asset_url(value: &serde_json::Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value["url"].as_str())
        .filter(|url| !url.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn asset_urls_are_read_from_strings_or_objects() {
        assert_eq!(
            asset_url(&json!({"url": "https://cdn/x.png"})),
            Some("https://cdn/x.png")
        );
        assert_eq!(
            asset_url(&json!("https://cdn/x.pdf")),
            Some("https://cdn/x.pdf")
        );
        assert_eq!(asset_url(&json!(null)), None);
        assert_eq!(asset_url(&json!({"url": " "})), None);
    }
}
