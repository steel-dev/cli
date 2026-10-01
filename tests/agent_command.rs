// ABOUTME: Black-box tests for `steel agent` with a stand-in agent script on PATH.
// ABOUTME: They check the arguments the agent receives, the defaults banner, and the exit status.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

/// Write an executable `name` script into `dir` that prints its arguments, one per line,
/// to `args.txt` and exits with `exit_code`.
fn write_fake_agent(dir: &Path, name: &str, exit_code: i32) {
    let script = format!(
        "#!/bin/sh\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > \"{}/args.txt\"\nexit {exit_code}\n",
        dir.display()
    );
    let path = dir.join(name);
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn run_agent(bin_dir: &Path, args: &[&str]) -> Output {
    let config = tempfile::tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_steel"))
        .env("PATH", bin_dir)
        .env("STEEL_CONFIG_DIR", config.path())
        .env("HOME", config.path())
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env("STEEL_FORCE_TTY", "1")
        .env_remove("STEEL_API_KEY")
        .arg("--no-update-check")
        .arg("agent")
        .args(args)
        .output()
        .expect("failed to execute steel binary")
}

fn received_args(bin_dir: &Path) -> Vec<String> {
    std::fs::read_to_string(bin_dir.join("args.txt"))
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn runs_claude_with_prompt_scoped_tools_and_extra_args() {
    let bin = tempfile::tempdir().unwrap();
    write_fake_agent(bin.path(), "claude", 0);

    let output = run_agent(
        bin.path(),
        &[
            "--no-session",
            "find",
            "cheap flights",
            "--",
            "--model",
            "opus",
        ],
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert!(
        stderr.contains("running Claude Code unattended"),
        "{stderr}"
    );
    assert!(stderr.contains("[--agent codex to change]"), "{stderr}");
    assert!(stderr.contains("[--yolo for all tools]"), "{stderr}");

    let args = received_args(bin.path());
    assert_eq!(args[0..2], ["-p", "find cheap flights"]);
    assert!(args.contains(&"--append-system-prompt".to_string()));
    assert!(args.contains(&"--allowedTools".to_string()));
    assert_eq!(args[args.len() - 2..], ["--model", "opus"]);
}

#[test]
fn uses_codex_when_claude_is_missing() {
    let bin = tempfile::tempdir().unwrap();
    write_fake_agent(bin.path(), "codex", 0);

    let output = run_agent(bin.path(), &["--no-session", "do", "something"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stderr.contains("running Codex unattended"), "{stderr}");
    let args = received_args(bin.path());
    assert_eq!(args[0], "exec");
    assert!(args.iter().any(|a| a.ends_with("# Task")));
    assert_eq!(args.last().unwrap(), "do something");
}

#[test]
fn agent_failure_fails_the_command() {
    let bin = tempfile::tempdir().unwrap();
    write_fake_agent(bin.path(), "claude", 3);

    let output = run_agent(bin.path(), &["--no-session", "task"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("claude exited with"), "{stderr}");
}

#[test]
fn no_agent_on_path_fails_before_starting_a_session() {
    let bin = tempfile::tempdir().unwrap();

    let output = run_agent(bin.path(), &["task"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No coding agent found"), "{stderr}");
    assert!(!stderr.contains("Starting browser session"), "{stderr}");
}
