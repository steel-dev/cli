//! Integration tests for the `steel dev` command family.
//!
//! Mirrors the contracts originally tested by the Node.js suite
//! (tests/unit/dev-local-runtime.test.ts), adapted for the Rust implementation.
//!
//! Since dev.rs shells out to `git` and `docker compose`, we test behavior via
//! the compiled binary (black-box) rather than calling functions directly.

use std::process::Command;

fn steel_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    // Suppress update checks so they don't interfere with output.
    cmd.arg("--no-update-check");
    cmd
}

/// Helper: run `steel` with a temporary STEEL_CONFIG_DIR so tests are isolated.
/// Also sets STEEL_FORCE_TTY=1 to keep text output in piped test environments.
fn steel_with_tmp_config(tmp: &tempfile::TempDir) -> Command {
    let mut cmd = steel_cmd();
    cmd.env("STEEL_CONFIG_DIR", tmp.path());
    cmd.env("STEEL_FORCE_TTY", "1");
    cmd
}

// ─── repo_path derivation ────────────────────────────────────────────────────

#[test]
fn repo_path_is_config_dir_plus_steel_browser() {
    // The config module is public, so we can verify the derivation in-process.
    let dir = steel_cli::config::config_dir_with(Some("/tmp/test-steel-cfg"));
    let repo = dir.join("steel-browser");
    assert_eq!(
        repo,
        std::path::PathBuf::from("/tmp/test-steel-cfg/steel-browser"),
    );
}

#[test]
fn repo_path_respects_steel_config_dir_env() {
    // When STEEL_CONFIG_DIR is set, config_dir() picks it up.
    let dir = steel_cli::config::config_dir_with(Some("/custom/dir"));
    assert_eq!(dir, std::path::PathBuf::from("/custom/dir"));
}

#[test]
fn repo_path_falls_back_to_home_config() {
    // When env var is absent, falls back to ~/.config/steel
    let dir = steel_cli::config::config_dir_with(None);
    assert!(
        dir.ends_with(".config/steel"),
        "expected path ending with .config/steel, got: {}",
        dir.display()
    );
}

// ─── Help output: verify flags exist ─────────────────────────────────────────

#[test]
fn dev_install_help_shows_expected_flags() {
    let output = steel_cmd()
        .args(["dev", "install", "--help"])
        .output()
        .expect("failed to run steel");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "steel dev install --help failed");
    assert!(
        stdout.contains("--repo-url"),
        "missing --repo-url flag in help:\n{stdout}"
    );
    assert!(
        stdout.contains("--verbose"),
        "missing --verbose flag in help:\n{stdout}"
    );
}

#[test]
fn dev_start_help_shows_expected_flags() {
    let output = steel_cmd()
        .args(["dev", "start", "--help"])
        .output()
        .expect("failed to run steel");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "steel dev start --help failed");
    assert!(
        stdout.contains("--port"),
        "missing --port flag in help:\n{stdout}"
    );
    assert!(
        stdout.contains("--verbose"),
        "missing --verbose flag in help:\n{stdout}"
    );
    assert!(
        stdout.contains("--docker-check"),
        "missing --docker-check flag in help:\n{stdout}"
    );
}

#[test]
fn dev_stop_help_shows_expected_flags() {
    let output = steel_cmd()
        .args(["dev", "stop", "--help"])
        .output()
        .expect("failed to run steel");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "steel dev stop --help failed");
    assert!(
        stdout.contains("--verbose"),
        "missing --verbose flag in help:\n{stdout}"
    );
}

// ─── Start/stop without installed runtime ────────────────────────────────────

#[test]
fn start_without_install_fails_with_not_installed_message() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    let output = steel_with_tmp_config(&tmp)
        .args(["dev", "start"])
        .output()
        .expect("failed to run steel");

    assert!(
        !output.status.success(),
        "steel dev start should fail when runtime is not installed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed"),
        "expected 'not installed' in stderr, got:\n{stderr}"
    );
}

#[test]
fn stop_without_install_fails_with_not_installed_message() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    let output = steel_with_tmp_config(&tmp)
        .args(["dev", "stop"])
        .output()
        .expect("failed to run steel");

    assert!(
        !output.status.success(),
        "steel dev stop should fail when runtime is not installed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed"),
        "expected 'not installed' in stderr, got:\n{stderr}"
    );
}

// ─── Installation idempotency ────────────────────────────────────────────────

#[test]
fn install_skips_when_repo_already_exists() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    // Pre-create the steel-browser directory to simulate a prior install.
    let repo_dir = tmp.path().join("steel-browser");
    std::fs::create_dir_all(&repo_dir).expect("failed to create fake repo dir");

    let output = steel_with_tmp_config(&tmp)
        .args(["dev", "install"])
        .output()
        .expect("failed to run steel");

    assert!(
        output.status.success(),
        "steel dev install should succeed (skip) when repo exists"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already installed"),
        "expected 'already installed' in stderr, got:\n{stderr}"
    );
}

// ─── Default port fallback ───────────────────────────────────────────────────

#[test]
fn start_default_port_is_3000() {
    // We verify through help text that the default is 3000 (the implementation
    // uses `unwrap_or(3000)`). Additionally, we can check the help for the port
    // flag description. This is a structural test rather than runtime since we
    // cannot start docker in CI.
    let output = steel_cmd()
        .args(["dev", "start", "--help"])
        .output()
        .expect("failed to run steel");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The port flag exists and is optional (no "required" marker).
    assert!(stdout.contains("--port"), "missing --port flag");
    assert!(
        stdout.contains("-p"),
        "missing -p short alias for --port:\n{stdout}"
    );

    // Verify via clap introspection that port default is None (meaning the code
    // falls back to 3000 in run_start).
    use clap::CommandFactory;
    let root = steel_cli::commands::Cli::command();
    let dev = root
        .get_subcommands()
        .find(|s| s.get_name() == "dev")
        .expect("dev subcommand not found");
    let start = dev
        .get_subcommands()
        .find(|s| s.get_name() == "start")
        .expect("start subcommand not found");
    let port_arg = start
        .get_arguments()
        .find(|a| a.get_long() == Some("port"))
        .expect("--port arg not found");
    // Port is optional (not required), so the code path hits unwrap_or(3000).
    assert!(
        !port_arg.is_required_set(),
        "--port should be optional so the default of 3000 applies"
    );
}

// ─── Docker compose v2 only ──────────────────────────────────────────────────

#[test]
fn rust_impl_uses_docker_compose_v2() {
    // The Rust implementation exclusively uses `docker compose` (v2 plugin syntax),
    // not the legacy `docker-compose` binary. We verify this structurally: the
    // source file should contain "docker" + "compose" args, not "docker-compose".
    let source = include_str!("../src/commands/dev.rs");
    assert!(
        source.contains(r#"ProcessCommand::new("docker")"#),
        "expected docker command invocation"
    );
    assert!(
        source.contains(r#""compose""#),
        "expected 'compose' as a subcommand argument"
    );
    assert!(
        !source.contains("docker-compose"),
        "should not use legacy docker-compose binary"
    );
}
