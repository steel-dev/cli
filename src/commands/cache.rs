use clap::Parser;
use serde_json::json;

use crate::util::output;

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
        output::success(
            json!({"cacheDir": cache_dir}),
            &format!(
                "Steel CLI cache directory: {}\nUse --clean to remove all cached files.\n",
                cache_dir.display()
            ),
        );
        return Ok(());
    }

    if !cache_dir.exists() {
        output::success(
            json!({"cacheDir": cache_dir, "removed": 0}),
            "Cache directory does not exist. Nothing to clean.\n",
        );
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

    output::success(
        json!({"cacheDir": cache_dir, "removed": count}),
        &format!(
            "Removed {count} item(s) from cache.\nCache directory: {}\n",
            cache_dir.display()
        ),
    );

    Ok(())
}
