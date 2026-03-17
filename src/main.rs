use clap::Parser;
use steel_cli::commands::{self, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = commands::run(cli).await {
        steel_cli::util::output::handle_error(&e);
    }
}
