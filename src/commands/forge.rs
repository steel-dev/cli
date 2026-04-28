use std::path::Path;

use clap::Parser;
use dialoguer::{Input, Select};
use include_dir::{Dir, include_dir};
use serde::Deserialize;
use serde_json::json;

use crate::status;
use crate::util::output;

static EXAMPLES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/external/cookbook/examples");
static REGISTRY_YAML: &str = include_str!("../../external/cookbook/registry.yaml");

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
// concerns. Plan: keep one major-version cycle, then drop.
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
        .unwrap_or(recipe.path.as_str())
}

fn load_recipes() -> anyhow::Result<Vec<Recipe>> {
    let recipes: Vec<Recipe> = serde_yaml::from_str(REGISTRY_YAML)?;
    Ok(recipes)
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let recipes = load_recipes()?;

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

    let template_dir = EXAMPLES
        .get_dir(slug)
        .ok_or_else(|| anyhow::anyhow!("Embedded template not found: {slug}"))?;

    let target_dir = std::env::current_dir()?.join(&project_name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists.", project_name);
    }

    status!(
        "Creating '{}' from template '{}'...",
        project_name,
        recipe.title
    );

    extract_dir(template_dir, &target_dir, slug)?;

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

// Walk an include_dir Dir and copy every file out to disk, rooting paths
// at `target` (so `playwright-ts/index.ts` becomes `<target>/index.ts`).
// `strip_prefix` is the slug we mounted at; include_dir paths are
// relative to EXAMPLES, so the slug always sits at the front.
fn extract_dir(dir: &Dir<'_>, target: &Path, strip_prefix: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(target)?;

    for file in dir.files() {
        let rel = file.path().strip_prefix(strip_prefix)?;
        let dst = target.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dst, file.contents())?;
    }

    for subdir in dir.dirs() {
        extract_dir(subdir, target, strip_prefix)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_parses_and_every_recipe_is_embedded() {
        let recipes = load_recipes().expect("registry.yaml parses");
        assert!(!recipes.is_empty(), "registry.yaml has at least one recipe");

        for recipe in &recipes {
            let slug = cli_slug(recipe);
            assert!(
                EXAMPLES.get_dir(slug).is_some(),
                "registry entry '{}' has no embedded directory at examples/{}",
                recipe.path,
                slug,
            );
        }
    }

    #[test]
    fn cli_slugs_are_unique() {
        let recipes = load_recipes().unwrap();
        let mut seen = std::collections::HashSet::new();
        for recipe in &recipes {
            let slug = cli_slug(recipe);
            assert!(
                seen.insert(slug.to_string()),
                "duplicate cli slug '{slug}' (cookbook verify_registry.py should prevent this)"
            );
        }
    }

    #[test]
    fn every_alias_resolves_to_a_real_slug() {
        let recipes = load_recipes().unwrap();
        let live: std::collections::HashSet<&str> = recipes.iter().map(cli_slug).collect();

        for (old, new) in ALIASES {
            assert!(
                live.contains(new),
                "alias '{old}' -> '{new}' but '{new}' is not in the current registry",
            );
        }
    }

    #[test]
    fn aliases_do_not_shadow_live_slugs() {
        let recipes = load_recipes().unwrap();
        let live: std::collections::HashSet<&str> = recipes.iter().map(cli_slug).collect();

        for (old, _) in ALIASES {
            assert!(
                !live.contains(old),
                "alias key '{old}' collides with a live slug — drop the alias",
            );
        }
    }
}
