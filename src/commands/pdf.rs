use clap::Parser;

use crate::api::client::SteelClient;
use crate::api::top_level::get_hosted_url;
use crate::config::auth;
use crate::config::settings::{ApiMode, EnvVars};
use crate::util::url::resolve_tool_url;

#[derive(Parser)]
pub struct Args {
    /// Target URL to generate PDF from
    pub url: Option<String>,

    /// Target URL to generate PDF from
    #[arg(short = 'u', long = "url")]
    pub url_flag: Option<String>,

    /// Delay before generating in milliseconds
    #[arg(short, long)]
    pub delay: Option<u64>,

    /// Use a Steel-managed residential proxy
    #[arg(long)]
    pub use_proxy: bool,

    /// Region identifier for request execution
    #[arg(short, long)]
    pub region: Option<String>,

    /// Send request to local Steel runtime
    #[arg(short, long)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long)]
    pub api_url: Option<String>,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let url = resolve_tool_url(args.url_flag.as_deref(), args.url.as_deref())?;

    let mode = ApiMode::resolve(args.local, args.api_url.as_deref());
    let auth = auth::resolve_auth();
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
    let local_config_url = config.as_ref().and_then(|c| c.local_api_url());
    let base_url = mode.resolve_base_url(args.api_url.as_deref(), &env_vars, local_config_url);

    let client = SteelClient::new()?;
    let response = client
        .pdf(
            &base_url,
            mode,
            &auth,
            &url,
            args.delay,
            args.use_proxy,
            args.region.as_deref(),
        )
        .await?;

    match get_hosted_url(&response) {
        Some(hosted_url) => println!("{hosted_url}"),
        None => println!("{}", serde_json::to_string_pretty(&response)?),
    }

    Ok(())
}
