//! OpenCode skill discovery uses only actual skill roots, never agent definitions.
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn run(home: &Path, cwd: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_steel"))
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env("STEEL_CONFIG_DIR", home.join("steel"))
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env("STEEL_API_KEY", "synthetic-key")
        .current_dir(cwd)
        .args(["--no-update-check", "--json", "skills"])
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn install(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        "---\nname: steel-browser\ndescription: fixture\n---\n",
    )
    .unwrap();
}

#[test]
fn project_xdg_and_global_roots_are_consistent_across_paths_list_and_doctor() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    install(&home.path().join(".config/opencode/agents/steel-browser.md"));
    let global = home.path().join(".agents/skills/steel-browser/SKILL.md");
    let xdg = home.path().join("xdg/opencode/skills/steel-browser.md");
    let local = project
        .path()
        .join(".opencode/skills/steel-browser/SKILL.md");
    install(&global);
    install(&xdg);
    install(&local);

    let paths = run(home.path(), project.path(), &["paths"])["data"].clone();
    let installed_count = paths
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["installed"] == true)
        .count();
    let found = paths
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["agent"] == "OpenCode" && row["installed"] == true)
        .count();
    assert_eq!(found, 3, "agent definitions must not count as skills");
    let list = run(home.path(), project.path(), &["list", "--offline"]);
    assert!(
        list["data"]["skills"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["name"] == "steel-browser" && row["installed"] == true)
    );
    let doctor = run(home.path(), project.path(), &["doctor", "--offline"]);
    assert_eq!(doctor["data"]["checks"]["installed_paths"], installed_count);
}

#[test]
fn nested_project_stops_discovery_at_git_worktree_boundary() {
    let home = tempfile::tempdir().unwrap();
    let outer = tempfile::tempdir().unwrap();
    install(&outer.path().join(".opencode/skills/steel-browser/SKILL.md"));
    let project = outer.path().join("project");
    let nested = project.join("nested/package");
    std::fs::create_dir_all(&nested).unwrap();
    // Worktrees use a .git file, not necessarily a directory.
    std::fs::write(project.join(".git"), "gitdir: synthetic-worktree\n").unwrap();
    let inside = project.join(".agents/skills/steel-browser/SKILL.md");
    install(&inside);

    let paths = run(home.path(), &nested, &["paths"])["data"].clone();
    let found: Vec<_> = paths
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["agent"] == "OpenCode" && row["installed"] == true)
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(
        Path::new(found[0]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        inside.canonicalize().unwrap()
    );
}
