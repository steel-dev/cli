use clap::Parser;
use steel_cli::commands::{self, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    commands::run(cli).await
}
