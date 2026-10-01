// ABOUTME: Black-box tests for `steel agent` with a stand-in agent script on PATH.
// ABOUTME: They check the arguments the agent receives, the defaults banner, and the exit status.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

/// Write an executable `name` script into `dir` that prints its arguments, one per line,
/// to `args.txt` and exits with `exit_code`.
fn write_fake_agent(dir: &Path, name: &str, exit_code: i32) {
    write_fake_agent_with(dir, name, &format!("exit {exit_code}"));
}

/// Like `write_fake_agent`, but runs `body` after it records its arguments.
fn write_fake_agent_with(dir: &Path, name: &str, body: &str) {
    let script = format!(
        "#!/bin/sh\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > \"{}/args.txt\"\n{body}\n",
        dir.display()
    );
    let path = dir.join(name);
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn run_agent(bin_dir: &Path, args: &[&str]) -> Output {
    let (mut command, _config) = steel_agent_command(bin_dir, args);
    command.output().expect("failed to execute steel binary")
}

/// Build a `steel agent` command with an isolated config directory. Keep the returned
/// directory alive until the command finishes.
fn steel_agent_command(bin_dir: &Path, args: &[&str]) -> (Command, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_steel"));
    command
        .env("PATH", bin_dir)
        .env("STEEL_CONFIG_DIR", config.path())
        .env("HOME", config.path())
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env("STEEL_FORCE_TTY", "1")
        .env_remove("STEEL_API_KEY")
        .arg("--no-update-check")
        .arg("agent")
        .args(args);
    (command, config)
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

const CODEX_WITH_LOG: &str = "echo 'CODEX-PROGRESS-LOG' >&2\necho 'FINAL-ANSWER'";

#[test]
fn codex_shows_only_the_final_answer_by_default() {
    let bin = tempfile::tempdir().unwrap();
    write_fake_agent_with(bin.path(), "codex", CODEX_WITH_LOG);

    let output = run_agent(bin.path(), &["--no-session", "task"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.contains("FINAL-ANSWER"), "{stdout}");
    assert!(!stderr.contains("CODEX-PROGRESS-LOG"), "{stderr}");
    assert!(stderr.contains("[--verbose for the full log]"), "{stderr}");
}

#[test]
fn codex_verbose_shows_the_progress_log() {
    let bin = tempfile::tempdir().unwrap();
    write_fake_agent_with(bin.path(), "codex", CODEX_WITH_LOG);

    let output = run_agent(bin.path(), &["--no-session", "--verbose", "task"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stderr.contains("CODEX-PROGRESS-LOG"), "{stderr}");
}

#[test]
fn codex_failure_shows_the_end_of_the_log() {
    let bin = tempfile::tempdir().unwrap();
    write_fake_agent_with(bin.path(), "codex", "echo 'CODEX-BROKE' >&2\nexit 2");

    let output = run_agent(bin.path(), &["--no-session", "task"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("CODEX-BROKE"), "{stderr}");
    assert!(stderr.contains("Full log:"), "{stderr}");
}

#[test]
fn agent_does_not_wait_for_input() {
    let bin = tempfile::tempdir().unwrap();
    // `read` waits forever if the agent gets an open stdin.
    write_fake_agent_with(bin.path(), "claude", "read -r line\nexit 0");

    let (mut command, _config) = steel_agent_command(bin.path(), &["--no-session", "task"]);
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let _stdin = child.stdin.take();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if std::time::Instant::now() > deadline {
            child.kill().unwrap();
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    assert!(status.is_some(), "steel agent waited for stdin");
}
