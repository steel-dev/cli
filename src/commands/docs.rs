use clap::Parser;

#[derive(Parser)]
pub struct Args;

pub async fn run(_args: Args) -> anyhow::Result<()> {
    println!("Opening Docs...");
    open::that("https://docs.steel.dev/overview/intro-to-steel")?;
    Ok(())
}
