#[tokio::main]
async fn main() {
    if steel_cli::is_flush_child() {
        steel_cli::run_flush_child().await;
        return;
    }
    let cli = steel_cli::parse();
    if let Err(e) = steel_cli::run(cli).await {
        steel_cli::handle_error(&e);
    }
}
