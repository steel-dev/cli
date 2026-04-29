use std::ffi::OsStr;
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
// To bump: run scripts/bump-cookbook.sh [<ref>] (defaults to upstream main).
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

// `examples/playwright-ts` -> `playwright-ts`. Cookbook's verify_registry.py
// enforces basename uniqueness, so we use it as the CLI-facing identifier.
// Falls back to the full path on the (unexpected) case where a registry
// entry has no basename — better than panicking on data we don't own.
fn cli_slug(recipe: &Recipe) -> &str {
    Path::new(&recipe.path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(recipe.path.as_str())
}

fn cookbook_cache_dir() -> anyhow::Result<PathBuf> {
    let base =
        dirs::cache_dir().ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;
    Ok(base.join("steel/cookbook").join(&COOKBOOK_REF[..12]))
}

async fn ensure_cookbook() -> anyhow::Result<PathBuf> {
    let cache = cookbook_cache_dir()?;
    ensure_cookbook_at(&cache, COOKBOOK_REPO, COOKBOOK_REF).await?;
    Ok(cache)
}

// Fetch + extract the cookbook zip at `git_ref`, materializing it at
// `cache`. Idempotent: if `cache/registry.yaml` already exists, returns
// immediately.
//
// Concurrency: two parallel runs may both download. They each unpack
// into a sibling tempdir and race to rename their copy into place. The
// loser's rename fails benignly because the winner's directory is
// non-empty; the loser's tempdir is dropped and we use the winner's
// (byte-identical) copy.
//
// After a successful promotion, sibling SHA-shaped directories are
// pruned. This bounds cache growth across CLI upgrades without needing
// the user to run `steel cache --clean`.
async fn ensure_cookbook_at(cache: &Path, repo: &str, git_ref: &str) -> anyhow::Result<()> {
    if cache.join("registry.yaml").exists() {
        return Ok(());
    }

    let short = &git_ref[..12.min(git_ref.len())];
    status!("Fetching templates from {repo}@{short}...");

    let url = format!("https://codeload.github.com/{repo}/zip/{git_ref}");
    let bytes = reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let parent = cache
        .parent()
        .ok_or_else(|| anyhow::anyhow!("cache dir has no parent: {}", cache.display()))?;
    std::fs::create_dir_all(parent)?;

    // Stage in a sibling of `cache` so the final rename is on the same
    // filesystem (avoids EXDEV when the system temp dir is on a
    // different mount).
    let staging = tempfile::tempdir_in(parent)?;
    let staging_path = staging.path().to_path_buf();

    // Unzip is CPU-bound and synchronous; keep it off the tokio worker
    // pool so we don't stall other futures (negligible here, but the
    // habit is cheap to keep).
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut archive = ZipArchive::new(Cursor::new(&bytes))?;
        archive.extract(&staging_path)?;
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("unzip task failed: {e}"))??;

    // GitHub wraps everything in `<repo>-<ref>/`. There should be exactly
    // one directory at the top level — find it and use it as the source
    // of our move.
    let inner = std::fs::read_dir(staging.path())?
        .filter_map(Result::ok)
        .find(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .ok_or_else(|| anyhow::anyhow!("Unexpected archive layout from {url}"))?
        .path();

    match std::fs::rename(&inner, cache) {
        Ok(()) => {}
        // Race lost (winner's non-empty dir blocks our rename) or any
        // other transient failure that left a usable cache behind: trust
        // the SHA pin and reuse what's there.
        Err(_) if cache.join("registry.yaml").exists() => {}
        Err(e) => return Err(e.into()),
    }

    prune_old_cache_siblings(cache);

    Ok(())
}

// Delete sibling cache dirs whose names look like a 12-char hex pin and
// don't match the current one. Restricting to the SHA shape avoids
// stomping on other processes' in-flight `tempdir_in` staging dirs.
fn prune_old_cache_siblings(current: &Path) {
    let (Some(parent), Some(current_name)) = (current.parent(), current.file_name()) else {
        return;
    };

    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        if name == current_name || !looks_like_cookbook_pin(&name) {
            continue;
        }
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

// Lowercase-only on purpose: the cache key we produce is always
// lowercase (`&COOKBOOK_REF[..12]`), so any uppercase-hex sibling is
// not ours to touch.
fn looks_like_cookbook_pin(name: &OsStr) -> bool {
    name.to_str().is_some_and(|s| {
        s.len() == 12
            && s.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    })
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
    fn cli_slug_falls_back_to_full_path() {
        // Pathological registry entry: a path with no basename. We don't
        // expect this in practice (cookbook's verify_registry.py rejects
        // it) but the CLI should not panic on data it doesn't own.
        let recipe = Recipe {
            title: "x".into(),
            path: String::new(),
            language: "ts".into(),
        };
        assert_eq!(cli_slug(&recipe), "");

        let recipe = Recipe {
            title: "x".into(),
            path: "examples/playwright-ts".into(),
            language: "ts".into(),
        };
        assert_eq!(cli_slug(&recipe), "playwright-ts");
    }

    #[test]
    fn cookbook_pin_is_12_hex_chars() {
        // The cache key uses the first 12 chars of COOKBOOK_REF and the
        // prune logic only deletes directories matching that shape.
        // Catches a malformed pin (e.g. truncated paste) at compile/test
        // time instead of at the user's machine.
        assert!(
            COOKBOOK_REF.len() >= 12 && COOKBOOK_REF[..12].chars().all(|c| c.is_ascii_hexdigit())
        );
    }

    #[test]
    fn pin_shape_matches_prune_filter() {
        let key = std::ffi::OsString::from(&COOKBOOK_REF[..12]);
        assert!(looks_like_cookbook_pin(&key));
        assert!(!looks_like_cookbook_pin(std::ffi::OsStr::new("not-a-sha")));
        assert!(!looks_like_cookbook_pin(std::ffi::OsStr::new(
            "ABCDEF012345"
        )));
        assert!(!looks_like_cookbook_pin(std::ffi::OsStr::new(
            "abcdef0123456" // 13 chars
        )));
    }

    #[test]
    fn prune_keeps_current_and_non_pin_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path();
        let current = parent.join("aaaaaaaaaaaa");
        let old_pin = parent.join("bbbbbbbbbbbb");
        let unrelated = parent.join("not-a-sha");

        for d in [&current, &old_pin, &unrelated] {
            std::fs::create_dir_all(d).unwrap();
            std::fs::write(d.join("registry.yaml"), "[]").unwrap();
        }

        prune_old_cache_siblings(&current);

        assert!(current.exists(), "current pin must survive");
        assert!(
            unrelated.exists(),
            "non-SHA siblings are not ours to delete"
        );
        assert!(!old_pin.exists(), "old SHA pin should be pruned");
    }

    // Online: hits codeload.github.com. Validates that COOKBOOK_REF is
    // resolvable and that every recipe in registry.yaml maps to a real
    // directory in the archive. Run via `cargo nextest run --run-ignored
    // only -E 'test(cookbook_pin_resolves)'`.
    #[tokio::test]
    #[ignore = "online: hits codeload.github.com"]
    async fn cookbook_pin_resolves() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cache = tmp.path().join(&COOKBOOK_REF[..12]);

        ensure_cookbook_at(&cache, COOKBOOK_REPO, COOKBOOK_REF)
            .await
            .expect("cookbook fetch + extract");

        let recipes = load_recipes(&cache).expect("registry.yaml parses");
        assert!(!recipes.is_empty(), "registry has at least one recipe");

        let mut seen = std::collections::HashSet::new();
        for r in &recipes {
            let dir = cache.join(&r.path);
            assert!(
                dir.is_dir(),
                "recipe '{}' points at missing dir: {}",
                cli_slug(r),
                dir.display()
            );
            assert!(
                seen.insert(cli_slug(r).to_string()),
                "duplicate cli slug '{}' in registry",
                cli_slug(r)
            );
        }
    }
}
