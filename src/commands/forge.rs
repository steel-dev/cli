use std::path::Path;

use clap::Parser;
use dialoguer::{Input, Select};
use include_dir::{Dir, include_dir};
use serde::Deserialize;

use crate::status;

static EXAMPLES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/external/cookbook/examples");
static REGISTRY_YAML: &str = include_str!("../../external/cookbook/registry.yaml");

#[derive(Parser)]
pub struct Args {
    /// Template to start from
    pub template: Option<String>,

    /// Project name
    #[arg(short, long)]
    pub name: Option<String>,
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

    let recipe = if let Some(ref template_arg) = args.template {
        recipes
            .iter()
            .find(|r| cli_slug(r) == template_arg)
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
}
