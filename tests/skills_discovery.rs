//! Exercise real skill diagnostics with isolated HOME/XDG/project state.
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn run(home: &Path, cwd: &Path, args: &[&str]) -> Value {
    let result = Command::new(env!("CARGO_BIN_EXE_steel"))
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env("STEEL_CONFIG_DIR", home.join("steel"))
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env("STEEL_API_KEY", "synthetic-offline-key")
        .env_remove("STEEL_SKILLS_MANIFEST_URL")
        .current_dir(cwd)
        .args(["--no-update-check", "--json", "skills"])
        .args(args)
        .output()
        .unwrap();
    serde_json::from_slice(&result.stdout).unwrap_or_else(|_| {
        panic!(
            "Expected diagnostic JSON: {}",
            String::from_utf8_lossy(&result.stderr)
        )
    })
}

fn install(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        "---\nname: steel-browser\ndescription: Synthetic skill\n---\n",
    )
    .unwrap();
}

fn installed_paths(home: &Path, cwd: &Path) -> Vec<String> {
    run(home, cwd, &["paths"])["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["agent"] == "OpenCode"
                && row["skill"] == "steel-browser"
                && row["installed"] == true
        })
        .map(|row| {
            Path::new(row["path"].as_str().unwrap())
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn project_skill_is_visible_in_paths_list_and_doctor() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let skill = project
        .path()
        .join(".opencode/skills/steel-browser/SKILL.md");
    install(&skill);
    assert_eq!(
        installed_paths(home.path(), project.path()),
        [skill.canonicalize().unwrap().to_str().unwrap()]
    );
    let listed = run(home.path(), project.path(), &["list", "--offline"]);
    assert!(
        listed["data"]["skills"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["name"] == "steel-browser" && row["installed"] == true)
    );
    let doctor = run(home.path(), project.path(), &["doctor", "--offline"]);
    assert!(
        doctor["data"]["checks"]["installed_paths"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn global_skills_use_xdg_and_do_not_count_agent_definitions() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    install(&home.path().join(".config/opencode/agents/steel-browser.md"));
    install(
        &home
            .path()
            .join(".config/opencode/skills/steel-browser/SKILL.md"),
    );
    assert!(
        installed_paths(home.path(), project.path()).is_empty(),
        "XDG override must replace the default config root"
    );
    let skill = home.path().join("xdg/opencode/skills/steel-browser.md");
    install(&skill);
    assert_eq!(
        installed_paths(home.path(), project.path()),
        [skill.canonicalize().unwrap().to_str().unwrap()]
    );
}

#[test]
fn nested_project_discovers_ancestor_skills_but_not_above_worktree_root() {
    let home = tempfile::tempdir().unwrap();
    let outer = tempfile::tempdir().unwrap();
    install(&outer.path().join(".opencode/skills/steel-browser/SKILL.md"));
    let project = outer.path().join("project");
    std::fs::create_dir_all(project.join("nested/package")).unwrap();
    std::fs::write(project.join(".git"), "gitdir: synthetic-worktree\n").unwrap();
    let skill = project.join(".agents/skills/steel-browser/SKILL.md");
    install(&skill);
    assert_eq!(
        installed_paths(home.path(), &project.join("nested/package")),
        [skill.canonicalize().unwrap().to_str().unwrap()]
    );
}

#[cfg(unix)]
#[test]
fn symlinked_project_skill_is_detected() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let target = home.path().join("fixture/steel-browser/SKILL.md");
    install(&target);
    let link = project.path().join(".opencode/skills/steel-browser");
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(target.parent().unwrap(), &link).unwrap();
    assert_eq!(
        installed_paths(home.path(), project.path()),
        [link
            .join("SKILL.md")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()]
    );
}
