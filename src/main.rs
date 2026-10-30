use clap::Parser;
use steel_cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = steel_cli::run(cli).await {
        steel_cli::handle_error(&e);
    }
}
