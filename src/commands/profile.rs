use clap::{Parser, Subcommand};
use serde_json::json;

use crate::browser::{profile_porter, profile_store};
use crate::config::auth;
use crate::config::settings::{ApiMode, EnvVars};
use crate::util::output;

#[derive(Subcommand)]
pub enum Command {
    /// List all saved Steel browser profiles
    List(ListArgs),

    /// Import a local browser profile into Steel
    Import(ImportArgs),

    /// Sync a local browser profile to an existing Steel profile
    Sync(SyncArgs),

    /// Delete a saved Steel browser profile
    Delete(DeleteArgs),
}

#[derive(Parser)]
pub struct ListArgs {}

#[derive(Parser)]
pub struct ImportArgs {
    /// Steel profile name to save as
    #[arg(long)]
    pub name: String,

    /// Browser profile directory to import from (e.g. "Default", "Profile 1")
    #[arg(long)]
    pub from: Option<String>,

    /// Browser to import from (chrome, edge, brave, arc, opera, vivaldi)
    #[arg(long)]
    pub browser: Option<String>,
}

#[derive(Parser)]
pub struct SyncArgs {
    /// Steel profile name to sync
    #[arg(long)]
    pub name: String,

    /// Browser profile directory to sync from (overrides stored source)
    #[arg(long)]
    pub from: Option<String>,

    /// Browser to sync from (overrides stored browser)
    #[arg(long)]
    pub browser: Option<String>,
}

#[derive(Parser)]
pub struct DeleteArgs {
    /// Name of the profile to delete
    #[arg(long)]
    pub name: String,
}

pub async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::List(args) => run_list(args).await,
        Command::Import(args) => run_import(args).await,
        Command::Sync(args) => run_sync(args).await,
        Command::Delete(args) => run_delete(args).await,
    }
}

async fn run_list(_args: ListArgs) -> anyhow::Result<()> {
    let profiles = profile_store::list_profiles()?;

    if output::is_json() {
        let data: Vec<serde_json::Value> = profiles
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "profileId": p.profile_id,
                })
            })
            .collect();
        output::success_data(json!(data));
        return Ok(());
    }

    if profiles.is_empty() {
        println!(
            "No profiles found. Use --profile <name> with steel browser start to create one."
        );
        return Ok(());
    }

    let max_name = profiles
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    println!("{:<max_name$}  PROFILE_ID", "NAME");
    for p in &profiles {
        println!("{:<max_name$}  {}", p.name, p.profile_id);
    }

    Ok(())
}

async fn run_delete(args: DeleteArgs) -> anyhow::Result<()> {
    if let Some(err) = profile_store::validate_profile_name(&args.name) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    if profile_store::delete_profile(&args.name)? {
        output::success(
            json!({"name": args.name, "deleted": true}),
            &format!("Deleted profile \"{}\". Note: Browser state on Steel servers is not affected.\n", args.name),
        );
    } else {
        return Err(output::error(&format!("Profile \"{}\" not found.", args.name)));
    }

    Ok(())
}

fn resolve_api_key() -> anyhow::Result<String> {
    let auth_info = auth::resolve_auth();
    auth_info
        .api_key
        .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run `steel login` or set STEEL_API_KEY."))
}

fn resolve_api_base() -> String {
    let mode = ApiMode::Cloud;
    let env_vars = EnvVars::from_env();
    let config = crate::config::settings::read_config().ok();
    let local_url = config.as_ref().and_then(|c| c.local_api_url());
    mode.resolve_base_url(None, &env_vars, local_url)
}

fn resolve_browser(browser_arg: Option<&str>) -> anyhow::Result<profile_porter::BrowserId> {
    if let Some(name) = browser_arg {
        return profile_porter::BrowserId::from_str(name).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown browser: \"{name}\". Supported: chrome, edge, brave, arc, opera, vivaldi"
            )
        });
    }

    // Detect installed browsers
    let installed = profile_porter::detect_installed_browsers();
    if installed.is_empty() {
        anyhow::bail!("No supported browsers found.");
    }
    if installed.len() == 1 {
        return Ok(installed[0]);
    }

    // Interactive selection
    let items: Vec<String> = installed.iter().map(|b| b.display_name().to_string()).collect();
    let selection = dialoguer::Select::new()
        .with_prompt("Select browser")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(installed[selection])
}

async fn run_import(args: ImportArgs) -> anyhow::Result<()> {
    // Validate
    if let Some(err) = profile_store::validate_profile_name(&args.name) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    let api_key = resolve_api_key()?;
    let api_base = resolve_api_base();

    // Resolve browser
    let browser_id = resolve_browser(args.browser.as_deref())?;

    // Find profiles for selected browser
    let profiles = profile_porter::find_browser_profiles(browser_id);
    if profiles.is_empty() {
        anyhow::bail!("No {} profiles found.", browser_id.display_name());
    }

    // Select profile
    let selected = if let Some(ref from) = args.from {
        profiles
            .iter()
            .find(|p| p.dir_name == *from)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{} profile \"{}\" not found. Available: {}",
                    browser_id.display_name(),
                    from,
                    profiles
                        .iter()
                        .map(|p| p.dir_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?
            .clone()
    } else {
        // Interactive selection
        if profile_porter::is_browser_running(browser_id) {
            eprintln!(
                "Warning: {} is currently running. Some data may be locked.",
                browser_id.display_name()
            );
        }

        let items: Vec<String> = profiles
            .iter()
            .map(|p| {
                if p.display_name != p.dir_name {
                    format!("{} ({})", p.display_name, p.dir_name)
                } else {
                    p.dir_name.clone()
                }
            })
            .collect();

        let selection = dialoguer::Select::new()
            .with_prompt(&format!("Select {} profile to import", browser_id.display_name()))
            .items(&items)
            .default(0)
            .interact()?;

        profiles[selection].clone()
    };

    // Package
    println!(
        "Importing {} ({}) → {}...",
        selected.dir_name,
        browser_id.display_name(),
        args.name
    );
    let result = profile_porter::package_profile(browser_id, &selected.dir_name, &|msg| {
        println!("  {msg}");
    })?;

    let size_mb = result.zip_buffer.len() as f64 / 1024.0 / 1024.0;

    // Upload
    println!("  Uploading...");
    let profile_id =
        profile_porter::upload_profile_to_steel(result.zip_buffer, &api_key, &api_base).await?;

    // Save metadata
    profile_store::write_profile(
        &args.name,
        &profile_id,
        Some(&selected.dir_name),
        Some(browser_id.as_str()),
    )?;

    println!();
    println!("  {}", args.name);
    println!("  id: {profile_id}");
    println!(
        "  cookies: {} re-encrypted · {:.1} MB",
        result.cookies_reencrypted, size_mb
    );
    println!();
    println!("  steel browser start --profile {}", args.name);

    Ok(())
}

async fn run_sync(args: SyncArgs) -> anyhow::Result<()> {
    // Validate
    if let Some(err) = profile_store::validate_profile_name(&args.name) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    let api_key = resolve_api_key()?;
    let api_base = resolve_api_base();

    // Read existing profile
    let stored = profile_store::read_profile(&args.name)?.ok_or_else(|| {
        anyhow::anyhow!(
            "Profile \"{}\" not found. Run `steel profile import --name {}` first.",
            args.name,
            args.name,
        )
    })?;

    // Resolve browser: CLI flag → stored → default to chrome
    let browser_id = if let Some(ref b) = args.browser {
        profile_porter::BrowserId::from_str(b)
            .ok_or_else(|| anyhow::anyhow!("Unknown browser: \"{b}\". Supported: chrome, edge, brave, arc, opera, vivaldi"))?
    } else if let Some(ref stored_browser) = stored.browser {
        profile_porter::BrowserId::from_str(stored_browser).unwrap_or(profile_porter::BrowserId::Chrome)
    } else {
        profile_porter::BrowserId::Chrome
    };

    // Determine profile dir source
    let profile_source = args
        .from
        .as_deref()
        .or(stored.chrome_profile.as_deref())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No source browser profile stored. Use --from to specify one."
            )
        })?
        .to_string();

    // Verify profile exists
    let browser_profiles = profile_porter::find_browser_profiles(browser_id);
    if !browser_profiles.iter().any(|p| p.dir_name == profile_source) {
        anyhow::bail!(
            "{} profile \"{}\" not found.",
            browser_id.display_name(),
            profile_source
        );
    }

    if profile_porter::is_browser_running(browser_id) {
        eprintln!(
            "Warning: {} is currently running. Some data may be locked.",
            browser_id.display_name()
        );
    }

    // Package
    println!(
        "Syncing {} ({}) → {}...",
        profile_source,
        browser_id.display_name(),
        args.name
    );
    let result = profile_porter::package_profile(browser_id, &profile_source, &|msg| {
        println!("  {msg}");
    })?;

    let size_mb = result.zip_buffer.len() as f64 / 1024.0 / 1024.0;

    // Update on Steel
    println!("  Uploading...");
    profile_porter::update_profile_on_steel(
        &stored.profile_id,
        result.zip_buffer,
        &api_key,
        &api_base,
    )
    .await?;

    // Update stored metadata if source changed
    let source_changed = args.from.is_some() && args.from.as_deref() != stored.chrome_profile.as_deref();
    let browser_changed = args.browser.is_some() && args.browser.as_deref() != stored.browser.as_deref();
    if source_changed || browser_changed {
        profile_store::write_profile(
            &args.name,
            &stored.profile_id,
            Some(&profile_source),
            Some(browser_id.as_str()),
        )?;
    }

    println!();
    println!("  Synced {}", args.name);
    println!(
        "  {} cookies re-encrypted · {:.1} MB",
        result.cookies_reencrypted, size_mb
    );

    Ok(())
}
