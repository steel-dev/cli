use clap::Parser;

#[derive(Parser)]
pub struct Args;

pub async fn run(_args: Args) -> anyhow::Result<()> {
    println!("Opening Discord...");
    open::that("https://discord.com/invite/steel-dev")?;
    Ok(())
}
