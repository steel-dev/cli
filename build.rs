//! Pre-compile guard for the cookbook submodule.
//!
//! `forge` embeds `external/cookbook/examples/` into the binary via
//! `include_dir!`. The macro takes the directory contents as-is — there
//! is no built-in exclusion. So if a contributor runs `npm install`
//! inside any example folder, the next build silently bakes
//! `node_modules/` into the binary and ships it.
//!
//! This script walks the embed root with full gitignore semantics
//! (cookbook root `.gitignore` plus per-example `.gitignore` files,
//! each scoped hierarchically to its own subtree — same as git itself).
//! Anything on disk that git would not track is reported as a build
//! error. The list of "forbidden" names lives entirely in cookbook —
//! cli has no opinion of its own.
//!
//! `cargo:rerun-if-changed` on a directory is recursive, so adding a
//! gitignored path deep in the tree triggers a re-walk.

use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const EMBED_ROOT: &str = "external/cookbook/examples";

fn main() {
    println!("cargo:rerun-if-changed={EMBED_ROOT}");

    let root = Path::new(EMBED_ROOT);
    if !root.exists() {
        abort(&format!(
            "missing cookbook submodule at '{EMBED_ROOT}'.\n\
             Run: git submodule update --init --recursive"
        ));
    }

    if let Err(e) = check(root) {
        abort(&e);
    }
}

fn check(root: &Path) -> Result<(), String> {
    // Set of paths git would track. WalkBuilder respects every
    // .gitignore on the way down; `parents(true)` pulls in the cookbook
    // root .gitignore that sits above the embed root.
    //
    // Filters set individually, NOT via `standard_filters(true)` — that
    // convenience method re-enables `hidden(true)`, which would skip
    // tracked dotfiles like `.gitignore`, `.env.example`, and
    // `.gitattributes`.
    let allowed: HashSet<PathBuf> = WalkBuilder::new(root)
        .hidden(false)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .require_git(false)
        .build()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut offenders = Vec::new();
    walk_disk(root, &allowed, &mut offenders)?;

    if offenders.is_empty() {
        return Ok(());
    }

    let shown: Vec<String> = offenders
        .iter()
        .take(10)
        .map(|p| format!("  {}", p.display()))
        .collect();
    let more = if offenders.len() > 10 {
        format!("\n  ... and {} more", offenders.len() - 10)
    } else {
        String::new()
    };

    Err(format!(
        "Gitignored paths inside the cookbook submodule:\n\
         \n\
         {}{more}\n\
         \n\
         These would be embedded into the binary by include_dir!.\n\
         Delete them (or move them outside the embed root) and rebuild.",
        shown.join("\n"),
    ))
}

// Walk the filesystem tree under `dir`, comparing every entry against
// the gitignore-derived allow set. Anything not in the set is recorded
// and not recursed into (no point listing each file inside an offending
// node_modules — the directory itself is the actionable unit).
fn walk_disk(dir: &Path, allowed: &HashSet<PathBuf>, offenders: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read '{}': {e}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Submodule's own `.git` file/symlink — not part of the working
        // tree, not in any gitignore, but also not something the binary
        // should embed. Skip silently.
        if path.file_name().is_some_and(|n| n == ".git") {
            continue;
        }

        if !allowed.contains(&path) {
            offenders.push(path);
            continue;
        }

        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            walk_disk(&path, allowed, offenders)?;
        }
    }
    Ok(())
}

fn abort(msg: &str) -> ! {
    eprintln!("\n=== steel-cli build aborted ===");
    eprintln!("{msg}");
    eprintln!();
    std::process::exit(1);
}
