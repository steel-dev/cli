use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Remove all cached files and directories
    #[arg(short, long)]
    pub clean: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("steel");

    if !args.clean {
        println!("Steel CLI cache directory: {}", cache_dir.display());
        println!("Use --clean to remove all cached files.");
        return Ok(());
    }

    if !cache_dir.exists() {
        println!("Cache directory does not exist. Nothing to clean.");
        return Ok(());
    }

    let mut count = 0u64;
    for entry in std::fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
        count += 1;
    }

    println!("Removed {count} item(s) from cache.");
    println!("Cache directory: {}", cache_dir.display());

    Ok(())
}
