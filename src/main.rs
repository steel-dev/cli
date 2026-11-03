#[tokio::main]
async fn main() {
    let cli = steel_cli::parse();
    if let Err(e) = steel_cli::run(cli).await {
        steel_cli::handle_error(&e);
    }
}
