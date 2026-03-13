use clap::Parser;

#[derive(Parser)]
pub struct Args;

pub async fn run(_args: Args) -> anyhow::Result<()> {
    println!("Opening Repository...");
    open::that("https://github.com/steel-dev/steel-browser")?;
    Ok(())
}
