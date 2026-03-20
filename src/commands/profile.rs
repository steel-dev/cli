use clap::{Parser, Subcommand};
use serde_json::json;

use crate::browser::{profile_porter, profile_store};
use crate::util::{api, output};

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

    /// Include all profile data (IndexedDB, History, Bookmarks, etc.)
    #[arg(long)]
    pub full: bool,
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

    /// Include all profile data (IndexedDB, History, Bookmarks, etc.)
    #[arg(long)]
    pub full: bool,
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
        println!("No profiles found. Use --profile <name> with steel browser start to create one.");
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
        anyhow::bail!("{err}");
    }

    if profile_store::delete_profile(&args.name)? {
        output::success(
            json!({"name": args.name, "deleted": true}),
            &format!(
                "Deleted profile \"{}\". Note: Browser state on Steel servers is not affected.\n",
                args.name
            ),
        );
    } else {
        anyhow::bail!("Profile \"{}\" not found.", args.name);
    }

    Ok(())
}

fn resolve_api() -> anyhow::Result<(String, String)> {
    let (_, base_url, auth) = api::resolve_with_auth();
    let api_key = auth.api_key.ok_or_else(|| {
        anyhow::anyhow!("Not authenticated. Run `steel login` or set STEEL_API_KEY.")
    })?;
    Ok((api_key, base_url))
}

fn parse_browser(name: &str) -> anyhow::Result<profile_porter::BrowserId> {
    profile_porter::BrowserId::from_str(name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown browser: \"{name}\". Supported: chrome, edge, brave, arc, opera, vivaldi"
        )
    })
}

struct BrowserProfileChoice {
    browser: profile_porter::BrowserId,
    profile: profile_porter::BrowserProfile,
}

fn discover_all_profiles(
    browser_filter: Option<&str>,
) -> anyhow::Result<Vec<BrowserProfileChoice>> {
    let browsers = if let Some(name) = browser_filter {
        vec![parse_browser(name)?]
    } else {
        let installed = profile_porter::detect_installed_browsers();
        if installed.is_empty() {
            anyhow::bail!("No supported browsers found.");
        }
        installed
    };

    let mut choices = Vec::new();
    for browser in browsers {
        for profile in profile_porter::find_browser_profiles(browser) {
            choices.push(BrowserProfileChoice { browser, profile });
        }
    }
    Ok(choices)
}

async fn run_import(args: ImportArgs) -> anyhow::Result<()> {
    // Validate
    if let Some(err) = profile_store::validate_profile_name(&args.name) {
        anyhow::bail!("{err}");
    }

    // Check for existing profile
    let existing = profile_store::read_profile(&args.name)?;
    if let Some(ref existing) = existing {
        eprintln!(
            "Profile \"{}\" already exists (id: {}). Overwriting.",
            args.name, existing.profile_id
        );
    }

    let (api_key, api_base) = resolve_api()?;

    // Discover profiles
    let choices = discover_all_profiles(args.browser.as_deref())?;
    if choices.is_empty() {
        anyhow::bail!("No browser profiles found.");
    }

    // Select profile
    let selected = if let Some(ref from) = args.from {
        // --from requires --browser (or exactly one installed browser)
        let browser_id = if let Some(ref b) = args.browser {
            parse_browser(b)?
        } else {
            let browsers: Vec<_> = choices
                .iter()
                .map(|c| c.browser)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            if browsers.len() == 1 {
                browsers[0]
            } else {
                anyhow::bail!("Multiple browsers found. Use --browser to specify which one.");
            }
        };
        choices
            .into_iter()
            .find(|c| c.browser == browser_id && c.profile.dir_name == *from)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{} profile \"{}\" not found.",
                    browser_id.display_name(),
                    from,
                )
            })?
    } else {
        // Interactive selection
        let running: std::collections::BTreeSet<_> = choices
            .iter()
            .map(|c| c.browser)
            .filter(|b| profile_porter::is_browser_running(*b))
            .collect();
        for b in &running {
            eprintln!(
                "Warning: {} is currently running. Some data may be locked.",
                b.display_name()
            );
        }

        let items: Vec<String> = choices
            .iter()
            .map(|c| {
                let name = &c.profile.display_name;
                let browser = c.browser.display_name();
                match &c.profile.email {
                    Some(email) => format!("{name} ({email}) · {browser}"),
                    None => format!("{name} · {browser}"),
                }
            })
            .collect();

        let selection = dialoguer::Select::new()
            .with_prompt("Select profile")
            .items(&items)
            .default(0)
            .interact()?;

        choices.into_iter().nth(selection).unwrap()
    };

    let browser_id = selected.browser;
    let selected = selected.profile;

    // Package
    println!(
        "Importing {} ({}) → {}...",
        selected.dir_name,
        browser_id.display_name(),
        args.name
    );
    let result =
        profile_porter::package_profile(browser_id, &selected.dir_name, args.full, &|msg| {
            println!("  {msg}");
        })?;

    let size_mb = result.zip_buffer.len() as f64 / 1024.0 / 1024.0;

    // Upload
    println!("  Uploading...");
    let profile_id = if let Some(ref existing) = existing {
        profile_porter::update_profile_on_steel(
            &existing.profile_id,
            result.zip_buffer,
            &api_key,
            &api_base,
        )
        .await?;
        existing.profile_id.clone()
    } else {
        profile_porter::upload_profile_to_steel(result.zip_buffer, &api_key, &api_base).await?
    };

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
    if result.cookies_skipped > 0 {
        println!(
            "  cookies: {} re-encrypted, {} skipped · {:.1} MB",
            result.cookies_reencrypted, result.cookies_skipped, size_mb
        );
    } else {
        println!(
            "  cookies: {} re-encrypted · {:.1} MB",
            result.cookies_reencrypted, size_mb
        );
    }
    println!();
    println!("  steel browser start --profile {}", args.name);

    Ok(())
}

async fn run_sync(args: SyncArgs) -> anyhow::Result<()> {
    // Validate
    if let Some(err) = profile_store::validate_profile_name(&args.name) {
        anyhow::bail!("{err}");
    }

    let (api_key, api_base) = resolve_api()?;

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
        parse_browser(b)?
    } else if let Some(ref stored_browser) = stored.browser {
        profile_porter::BrowserId::from_str(stored_browser)
            .unwrap_or(profile_porter::BrowserId::Chrome)
    } else {
        profile_porter::BrowserId::Chrome
    };

    // Determine profile dir source
    let profile_source = args
        .from
        .as_deref()
        .or(stored.browser_profile.as_deref())
        .ok_or_else(|| {
            anyhow::anyhow!("No source browser profile stored. Use --from to specify one.")
        })?
        .to_string();

    // Verify profile exists
    let browser_profiles = profile_porter::find_browser_profiles(browser_id);
    if !browser_profiles
        .iter()
        .any(|p| p.dir_name == profile_source)
    {
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
    let result = profile_porter::package_profile(browser_id, &profile_source, args.full, &|msg| {
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
    let source_changed =
        args.from.is_some() && args.from.as_deref() != stored.browser_profile.as_deref();
    let browser_changed =
        args.browser.is_some() && args.browser.as_deref() != stored.browser.as_deref();
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
    if result.cookies_skipped > 0 {
        println!(
            "  {} cookies re-encrypted, {} skipped · {:.1} MB",
            result.cookies_reencrypted, result.cookies_skipped, size_mb
        );
    } else {
        println!(
            "  {} cookies re-encrypted · {:.1} MB",
            result.cookies_reencrypted, size_mb
        );
    }

    Ok(())
}
