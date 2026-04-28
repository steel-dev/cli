use std::io::Cursor;
use std::path::{Path, PathBuf};

use clap::Parser;
use dialoguer::{Input, Select};
use serde::Deserialize;
use serde_json::json;
use zip::ZipArchive;

use crate::status;
use crate::util::output;

const COOKBOOK_REPO: &str = "steel-dev/steel-cookbook";
const COOKBOOK_REF: &str = "8fc2b7202e3adada392342d5f089c50c847358ce";

#[derive(Parser)]
pub struct Args {
    /// Template to start from
    pub template: Option<String>,

    /// Project name
    #[arg(short, long)]
    pub name: Option<String>,

    /// List available templates and exit (respects --json)
    #[arg(long, conflicts_with_all = ["template", "name"])]
    pub list: bool,
}

// Mirror of cookbook's registry.yaml entry shape. Cookbook owns the
// schema; only fields the CLI needs are pulled in. Extra fields
// (description, topics, authors, ...) are ignored.
#[derive(Deserialize)]
struct Recipe {
    title: String,
    path: String,
    language: String,
}

// Renames carried over from the pre-cookbook-rework registry. Covers
// both the old `steel-*-starter` slugs and the shorthand aliases that
// shipped in the embedded manifest.json. JS-only variants (playwright-js,
// puppeteer-js) were retired in favor of the TS cousins.
//
// This table only lives in the CLI — cookbook stays unaware of CLI
// concerns.
const ALIASES: &[(&str, &str)] = &[
    // Old `steel-*-starter` slugs
    ("steel-auth-context-starter", "auth-context"),
    ("steel-browser-use-starter", "browser-use"),
    (
        "steel-claude-computer-use-node-starter",
        "claude-computer-use-ts",
    ),
    (
        "steel-claude-computer-use-python-starter",
        "claude-computer-use-py",
    ),
    ("steel-credentials-starter", "credentials"),
    ("steel-files-api-starter", "files-api"),
    ("steel-magnitude-starter", "magnitude"),
    (
        "steel-oai-computer-use-node-starter",
        "openai-computer-use-ts",
    ),
    (
        "steel-oai-computer-use-python-starter",
        "openai-computer-use-py",
    ),
    ("steel-playwright-python-starter", "playwright-py"),
    ("steel-playwright-starter", "playwright-ts"),
    ("steel-playwright-starter-js", "playwright-ts"),
    ("steel-puppeteer-starter", "puppeteer-ts"),
    ("steel-puppeteer-starter-js", "puppeteer-ts"),
    ("steel-selenium-starter", "selenium"),
    ("steel-stagehand-node-starter", "stagehand-ts"),
    ("steel-stagehand-python-starter", "stagehand-py"),
    // Old shorthands
    ("auth", "auth-context"),
    ("claude-cua", "claude-computer-use-ts"),
    ("claude-cua-py", "claude-computer-use-py"),
    ("creds", "credentials"),
    ("files", "files-api"),
    ("oai-cua", "openai-computer-use-ts"),
    ("oai-cua-py", "openai-computer-use-py"),
    ("playwright", "playwright-ts"),
    ("playwright-js", "playwright-ts"),
    ("puppeteer", "puppeteer-ts"),
    ("puppeteer-js", "puppeteer-ts"),
    ("stagehand", "stagehand-ts"),
];

fn resolve_alias(input: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find(|(old, _)| *old == input)
        .map(|(_, new)| *new)
}

// `examples/playwright-ts` -> `playwright-ts`. The basename is unique
// across the registry (cookbook's verify_registry.py enforces this) so
// we use it as the CLI-facing identifier.
fn cli_slug(recipe: &Recipe) -> &str {
    Path::new(&recipe.path)
        .file_name()
        .and_then(|s| s.to_str())
        .expect("registry path must have a basename — verify_registry.py invariant")
}

fn cookbook_cache_dir() -> anyhow::Result<PathBuf> {
    let base =
        dirs::cache_dir().ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;
    Ok(base.join("steel/cookbook").join(&COOKBOOK_REF[..12]))
}

// Fetch + extract the cookbook tarball at COOKBOOK_REF, cached under
// the user's cache dir. Returns the path that contains `registry.yaml`
// and `examples/`.
//
// Concurrency: two parallel runs may both download. They each unpack
// to their own tempdir and race to rename it into place; the loser's
// rename fails benignly and we use whichever copy won.
async fn ensure_cookbook() -> anyhow::Result<PathBuf> {
    let cache = cookbook_cache_dir()?;
    if cache.join("registry.yaml").exists() {
        return Ok(cache);
    }

    status!(
        "Fetching templates from {COOKBOOK_REPO}@{}...",
        &COOKBOOK_REF[..12]
    );

    let url = format!("https://codeload.github.com/{COOKBOOK_REPO}/zip/{COOKBOOK_REF}");
    let bytes = reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let staging = tempfile::tempdir()?;
    let mut archive = ZipArchive::new(Cursor::new(&bytes))?;
    archive.extract(staging.path())?;

    // GitHub wraps everything in `<repo>-<ref>/`. There should be exactly
    // one entry at the top level — find it and use it as our cookbook
    // root.
    let inner = std::fs::read_dir(staging.path())?
        .filter_map(Result::ok)
        .find(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .ok_or_else(|| anyhow::anyhow!("Unexpected tarball layout from {url}"))?
        .path();

    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic-ish promotion. If a parallel process beat us, just use
    // theirs — both copies have the same SHA-pinned content.
    match std::fs::rename(&inner, &cache) {
        Ok(()) => {}
        Err(_) if cache.join("registry.yaml").exists() => {}
        Err(e) => return Err(e.into()),
    }

    Ok(cache)
}

fn load_recipes(cookbook: &Path) -> anyhow::Result<Vec<Recipe>> {
    let yaml = std::fs::read_to_string(cookbook.join("registry.yaml"))?;
    let recipes: Vec<Recipe> = serde_yaml::from_str(&yaml)?;
    Ok(recipes)
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let cookbook = ensure_cookbook().await?;
    let recipes = load_recipes(&cookbook)?;

    if recipes.is_empty() {
        anyhow::bail!("No templates available.");
    }

    if args.list {
        print_list(&recipes);
        return Ok(());
    }

    let recipe = if let Some(ref template_arg) = args.template {
        let resolved = resolve_alias(template_arg).unwrap_or(template_arg.as_str());
        if resolved != template_arg {
            eprintln!(
                "warning: '{template_arg}' was renamed to '{resolved}'. Continuing with '{resolved}'."
            );
        }
        recipes
            .iter()
            .find(|r| cli_slug(r) == resolved)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {template_arg}"))?
    } else {
        let items: Vec<String> = recipes
            .iter()
            .map(|r| format!("{} [{}]", r.title, r.language))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a template")
            .items(&items)
            .default(0)
            .interact()?;

        &recipes[selection]
    };

    let slug = cli_slug(recipe);

    let project_name = if let Some(ref name) = args.name {
        name.clone()
    } else {
        Input::new()
            .with_prompt("Project name")
            .default(slug.to_string())
            .interact_text()?
    };

    let template_src = cookbook.join(&recipe.path);
    if !template_src.exists() {
        anyhow::bail!(
            "Template directory not found in cookbook: {}",
            template_src.display()
        );
    }

    let target_dir = std::env::current_dir()?.join(&project_name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists.", project_name);
    }

    status!(
        "Creating '{}' from template '{}'...",
        project_name,
        recipe.title
    );

    copy_dir(&template_src, &target_dir)?;

    status!(
        "Project '{}' created at {}",
        project_name,
        target_dir.display()
    );
    status!("\nNext steps:");
    status!("  cd {project_name}");

    match recipe.language.to_lowercase().as_str() {
        "python" => status!("  pip install -r requirements.txt"),
        _ => status!("  npm install"),
    }

    Ok(())
}

// Plain stdout — `--list` is a discovery action, not a status update,
// so it goes to stdout (pipeable into grep/jq) rather than the status!
// macro that targets stderr. JSON mode emits the same shape as other
// list commands (profile list, credentials list).
fn print_list(recipes: &[Recipe]) {
    if output::is_json() {
        let data: Vec<serde_json::Value> = recipes
            .iter()
            .map(|r| {
                json!({
                    "slug": cli_slug(r),
                    "title": r.title,
                    "language": r.language,
                })
            })
            .collect();
        output::success_data(json!(data));
        return;
    }

    let max_slug = recipes
        .iter()
        .map(|r| cli_slug(r).len())
        .max()
        .unwrap_or(4)
        .max(4);
    let max_lang = recipes
        .iter()
        .map(|r| r.language.len())
        .max()
        .unwrap_or(8)
        .max(8);

    println!("{:<max_slug$}  {:<max_lang$}  TITLE", "SLUG", "LANGUAGE");
    for r in recipes {
        println!(
            "{:<max_slug$}  {:<max_lang$}  {}",
            cli_slug(r),
            r.language,
            r.title,
        );
    }
}

fn copy_dir(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_alias_maps_old_slugs() {
        assert_eq!(resolve_alias("playwright"), Some("playwright-ts"));
        assert_eq!(
            resolve_alias("steel-puppeteer-starter"),
            Some("puppeteer-ts")
        );
        assert_eq!(resolve_alias("creds"), Some("credentials"));
        assert_eq!(resolve_alias("does-not-exist"), None);
    }

    #[test]
    fn alias_keys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for (old, _) in ALIASES {
            assert!(
                seen.insert(*old),
                "duplicate alias key '{old}' — second entry shadows first",
            );
        }
    }

    #[test]
    fn alias_targets_look_like_live_slugs() {
        // Lightweight sanity: every alias target follows the
        // `<thing>[-lang]` shape that cookbook example dirs use.
        // Validation against the actual registry happens at runtime when
        // the user picks a template (and at CI-time when COOKBOOK_REF
        // is bumped).
        for (_, new) in ALIASES {
            assert!(
                new.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit()),
                "alias target '{new}' has unexpected characters",
            );
            assert!(
                !new.starts_with('-') && !new.ends_with('-'),
                "alias target '{new}' has leading/trailing dash",
            );
        }
    }
}
