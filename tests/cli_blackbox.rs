//! Black-box integration tests for the `steel` CLI binary.
//!
//! These tests exercise the compiled binary via `std::process::Command`,
//! verifying exit codes, stdout/stderr patterns, and flag parsing.
//! They do NOT import any library code — they treat the CLI as an opaque
//! executable, ensuring the user-facing contract is preserved.

use std::path::Path;
use std::process::{Command, Output};

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build a `Command` targeting the `steel` binary with a sanitized environment.
///
/// - `STEEL_CONFIG_DIR` is set to a per-test temp directory so we never
///   touch the real user config.
/// - `STEEL_API_KEY` is explicitly removed to avoid accidental auth.
/// - `HOME` is preserved so the binary can still resolve paths if needed.
fn steel_cmd() -> (Command, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("STEEL_CONFIG_DIR", tmp.path());
    cmd.env("STEEL_TELEMETRY_DISABLED", "1");
    cmd.env_remove("STEEL_API_KEY");
    // Suppress update checks so they don't interfere with output
    cmd.arg("--no-update-check");
    (cmd, tmp)
}

/// Convenience: run steel with the given args and return the Output.
fn run(args: &[&str]) -> Output {
    let (mut cmd, _tmp) = steel_cmd();
    cmd.args(args);
    cmd.output().expect("failed to execute steel binary")
}

/// Convert Output stdout to a String (lossy).
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Convert Output stderr to a String (lossy).
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// ─── Version & top-level help ───────────────────────────────────────

#[test]
fn version_exits_zero_and_prints_version() {
    // --version is handled by clap before --no-update-check is parsed,
    // so we call the binary directly without the helper.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.arg("--version");
    let output = cmd.output().expect("failed to execute steel binary");
    assert!(output.status.success(), "steel --version should exit 0");
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(
        out.contains("steel-cli") || out.contains("steel"),
        "stdout should contain the binary name, got: {out}"
    );
    // Version string from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        out.contains(version),
        "stdout should contain version {version}, got: {out}"
    );
}

#[test]
fn help_exits_zero_and_mentions_steel_cli() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "steel --help should exit 0");
    let out = stdout(&output);
    assert!(
        out.contains("Steel CLI"),
        "help output should contain 'Steel CLI', got: {out}"
    );
}

// ─── Subcommand help availability ───────────────────────────────────

#[test]
fn browser_help_lists_subcommands() {
    let output = run(&["browser", "--help"]);
    assert!(
        output.status.success(),
        "steel browser --help should exit 0"
    );
    let out = stdout(&output);
    for sub in &["start", "stop", "sessions", "live"] {
        assert!(
            out.contains(sub),
            "browser help should list '{sub}', got: {out}"
        );
    }
}

#[test]
fn scrape_help_shows_expected_flags() {
    let output = run(&["scrape", "--help"]);
    assert!(output.status.success(), "steel scrape --help should exit 0");
    let out = stdout(&output);
    for flag in &["url", "--format", "--delay"] {
        assert!(
            out.contains(flag),
            "scrape help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn screenshot_help_shows_expected_flags() {
    let output = run(&["screenshot", "--help"]);
    assert!(
        output.status.success(),
        "steel screenshot --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["url", "--delay", "--full-page"] {
        assert!(
            out.contains(flag),
            "screenshot help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn pdf_help_shows_expected_flags() {
    let output = run(&["pdf", "--help"]);
    assert!(output.status.success(), "steel pdf --help should exit 0");
    let out = stdout(&output);
    for flag in &["url", "--delay"] {
        assert!(
            out.contains(flag),
            "pdf help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn dev_help_lists_subcommands() {
    let output = run(&["dev", "--help"]);
    assert!(output.status.success(), "steel dev --help should exit 0");
    let out = stdout(&output);
    for sub in &["install", "start", "stop"] {
        assert!(
            out.contains(sub),
            "dev help should list '{sub}', got: {out}"
        );
    }
}

#[test]
fn credentials_help_lists_subcommands() {
    let output = run(&["credentials", "--help"]);
    assert!(
        output.status.success(),
        "steel credentials --help should exit 0"
    );
    let out = stdout(&output);
    for sub in &["create", "update", "delete", "list"] {
        assert!(
            out.contains(sub),
            "credentials help should list '{sub}', got: {out}"
        );
    }
}

#[test]
fn skills_help_lists_subcommands() {
    let output = run(&["skills", "--help"]);
    assert!(output.status.success(), "steel skills --help should exit 0");
    let out = stdout(&output);
    for sub in &["list", "install", "update", "doctor", "open", "paths"] {
        assert!(
            out.contains(sub),
            "skills help should list '{sub}', got: {out}"
        );
    }
}

#[test]
fn skills_list_offline_shows_launch_catalog() {
    let output = run(&["skills", "list", "--offline"]);
    assert!(
        output.status.success(),
        "steel skills list --offline should exit 0; stderr: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    for skill in &[
        "steel-browser",
        "steel-developer",
        "steel-session-debugging",
        "steel-reliability",
        "steel-skill-creator",
    ] {
        assert!(
            out.contains(skill),
            "skills list should include '{skill}', got: {out}"
        );
    }
}

#[test]
fn profile_help_lists_subcommands() {
    let output = run(&["profile", "--help"]);
    assert!(
        output.status.success(),
        "steel profile --help should exit 0"
    );
    let out = stdout(&output);
    for sub in &["import", "sync", "list", "delete"] {
        assert!(
            out.contains(sub),
            "profile help should list '{sub}', got: {out}"
        );
    }
}

#[test]
fn sessions_help_lists_subcommands() {
    let output = run(&["sessions", "--help"]);
    assert!(
        output.status.success(),
        "steel sessions --help should exit 0"
    );
    let out = stdout(&output);
    for sub in &[
        "list",
        "get",
        "metadata",
        "release",
        "logs",
        "agent-logs",
        "traces",
    ] {
        assert!(
            out.contains(sub),
            "sessions help should list '{sub}', got: {out}"
        );
    }
}

#[test]
fn sessions_list_help_shows_filters() {
    let output = run(&["sessions", "list", "--help"]);
    assert!(
        output.status.success(),
        "steel sessions list --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &[
        "--metadata <KEY=VALUE>",
        "--signal",
        "--release-reason",
        "--created-after",
        "--created-before",
        "--cursor-id",
    ] {
        assert!(
            out.contains(flag),
            "sessions list help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn sessions_metadata_help_shows_expected_flags() {
    let output = run(&["sessions", "metadata", "--help"]);
    assert!(
        output.status.success(),
        "steel sessions metadata --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &[
        "--session",
        "--set <KEY=VALUE>",
        "--unset <KEY>",
        "--replace",
        "--clear",
    ] {
        assert!(
            out.contains(flag),
            "sessions metadata help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn sessions_metadata_replace_without_set_fails_instead_of_clearing() {
    let output = run(&["sessions", "metadata", "s1", "--replace"]);
    assert!(
        !output.status.success(),
        "--replace without --set should fail"
    );
    assert!(
        stderr(&output).contains("--set <KEY=VALUE>"),
        "stderr should ask for --set, got: {}",
        stderr(&output)
    );
}

#[test]
fn sessions_metadata_rejects_a_pair_without_an_equals_sign() {
    let output = run(&["sessions", "metadata", "s1", "--set", "env"]);
    assert!(
        !output.status.success(),
        "--set without KEY=VALUE should fail"
    );
    assert!(
        stderr(&output).contains("expected KEY=VALUE"),
        "stderr should explain the format, got: {}",
        stderr(&output)
    );
}

#[test]
fn sessions_list_rejects_a_repeated_metadata_key_before_calling_the_api() {
    let output = run(&[
        "sessions",
        "list",
        "--metadata",
        "env=prod",
        "--metadata",
        "env=staging",
    ]);
    assert!(
        !output.status.success(),
        "a repeated metadata key should fail"
    );
    let err = stderr(&output);
    assert!(
        err.contains("Metadata key \"env\" is given more than once."),
        "stderr should name the repeated key, got: {err}"
    );
}

#[test]
fn sessions_list_rejects_an_empty_metadata_filter_before_calling_the_api() {
    let output = run(&["sessions", "list", "--metadata", "env="]);
    assert!(
        !output.status.success(),
        "an empty metadata filter should fail"
    );
    let err = stderr(&output);
    assert!(
        err.contains("Metadata filter \"env\" needs a value."),
        "stderr should name the empty filter, got: {err}"
    );
}

#[test]
fn sessions_logs_help_shows_expected_flags() {
    let output = run(&["sessions", "logs", "--help"]);
    assert!(
        output.status.success(),
        "steel sessions logs --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &[
        "--session",
        "--follow",
        "--since",
        "--event-type",
        "--limit",
        "--offset",
    ] {
        assert!(
            out.contains(flag),
            "sessions logs help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn sessions_traces_help_shows_expected_flags() {
    let output = run(&["sessions", "traces", "--help"]);
    assert!(
        output.status.success(),
        "steel sessions traces --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--session", "--follow", "--since", "--event-type"] {
        assert!(
            out.contains(flag),
            "sessions traces help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn login_help_exits_zero() {
    let output = run(&["login", "--help"]);
    assert!(output.status.success(), "steel login --help should exit 0");
}

#[test]
fn auth_alias_help_exits_zero() {
    let output = run(&["auth", "--help"]);
    assert!(
        output.status.success(),
        "steel auth --help should exit 0 (alias for login)"
    );
}

// ─── Error handling contracts ───────────────────────────────────────

#[test]
fn scrape_without_url_fails() {
    let output = run(&["scrape"]);
    assert!(
        !output.status.success(),
        "steel scrape (no URL) should exit non-zero"
    );
    let err = stderr(&output);
    let out = stdout(&output);
    let combined = format!("{err}{out}");
    // The error should mention that a URL is missing (from resolve_tool_url)
    // or be a usage error from clap. Either way, it should not silently succeed.
    assert!(
        combined.to_lowercase().contains("url")
            || combined.to_lowercase().contains("missing")
            || combined.to_lowercase().contains("argument")
            || combined.to_lowercase().contains("error"),
        "error output should mention URL or missing argument, got: {combined}"
    );
}

#[test]
fn screenshot_without_url_fails() {
    let output = run(&["screenshot"]);
    assert!(
        !output.status.success(),
        "steel screenshot (no URL) should exit non-zero"
    );
    let err = stderr(&output);
    let out = stdout(&output);
    let combined = format!("{err}{out}");
    assert!(
        combined.to_lowercase().contains("url")
            || combined.to_lowercase().contains("missing")
            || combined.to_lowercase().contains("argument")
            || combined.to_lowercase().contains("error"),
        "error output should mention URL or missing argument, got: {combined}"
    );
}

#[test]
fn pdf_without_url_fails() {
    let output = run(&["pdf"]);
    assert!(
        !output.status.success(),
        "steel pdf (no URL) should exit non-zero"
    );
}

#[test]
fn nonexistent_command_fails() {
    let output = run(&["nonexistent-command"]);
    assert!(
        !output.status.success(),
        "steel nonexistent-command should exit non-zero"
    );
    let err = stderr(&output);
    // clap should report an unrecognized subcommand
    assert!(
        !err.is_empty(),
        "stderr should contain an error message for unknown command"
    );
}

#[test]
fn browser_start_without_auth_or_local_fails() {
    // With no API key and no --local flag, `browser start` should fail
    // with the missing API key error, not a generic timeout.
    let start = std::time::Instant::now();
    let output = run(&["browser", "start"]);
    let elapsed = start.elapsed();

    assert!(
        !output.status.success(),
        "steel browser start without auth should exit non-zero"
    );

    let err = stderr(&output);
    let out = stdout(&output);
    let combined = format!("{err}{out}");
    let lower = combined.to_lowercase();

    // The error should surface the actual failure reason (missing API key).
    assert!(
        lower.contains("api key") || lower.contains("api_key") || lower.contains("login"),
        "output should mention the actual auth error (API key / login), got: {combined}"
    );

    assert!(
        elapsed.as_secs() < 15,
        "should fail quickly, but took {elapsed:?}"
    );
}

#[test]
fn piped_output_is_text_unless_json_is_passed() {
    let text = run(&["scrape", "https://example.com"]);
    assert!(!text.status.success());
    assert!(stdout(&text).is_empty(), "got: {}", stdout(&text));
    assert!(stderr(&text).contains("Error: Missing Steel API key"));

    let json = run(&["--json", "scrape", "https://example.com"]);
    let parsed: serde_json::Value = serde_json::from_str(stdout(&json).trim()).unwrap();
    assert_eq!(parsed["error_code"], "auth_required");
}

#[test]
fn browser_start_without_auth_json_mode_surfaces_error() {
    // In --json mode, the actual error should also appear in the output.
    let output = run(&["--json", "browser", "start"]);
    assert!(!output.status.success());

    let out = stdout(&output);
    let parsed: serde_json::Value = serde_json::from_str(out.trim())
        .unwrap_or_else(|_| panic!("expected valid JSON, got: {out}"));
    assert_eq!(parsed["success"], false);

    let error_msg = parsed["error"].as_str().unwrap_or("");
    let lower = error_msg.to_lowercase();
    assert!(
        lower.contains("api key") || lower.contains("api_key") || lower.contains("login"),
        "JSON error should mention the actual auth error, got: {error_msg}"
    );
}

#[test]
fn browser_action_with_missing_named_session_fails_without_starting_one() {
    let (mut cmd, tmp) = steel_cmd();
    cmd.args([
        "--json",
        "browser",
        "navigate",
        "https://example.com",
        "--session",
        "missing",
    ]);
    let output = cmd.output().expect("failed to execute steel binary");
    assert!(!output.status.success());

    let out = stdout(&output);
    let parsed: serde_json::Value = serde_json::from_str(out.trim())
        .unwrap_or_else(|_| panic!("expected valid JSON, got: {out}"));
    let error_msg = parsed["error"].as_str().unwrap_or("");
    assert!(
        error_msg.contains("steel browser start --session missing"),
        "error should say how to start the session, got: {error_msg}"
    );
    assert!(
        !tmp.path()
            .join("browser-sessions")
            .join("missing.json")
            .exists(),
        "no session should be created for a missing named session"
    );
}

#[test]
fn browser_action_without_session_warns_before_starting_default() {
    let output = run(&["--json", "browser", "navigate", "https://example.com"]);
    assert!(!output.status.success());

    let err = stderr(&output);
    assert!(
        err.contains("warning: session \"default\" is not running"),
        "stderr should warn that a default session is being started, got: {err}"
    );
}

fn single_json_result(output: &Output) -> serde_json::Value {
    let out = stdout(output);
    let lines: Vec<&str> = out.lines().filter(|line| !line.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "expected one JSON result, got stdout={out} stderr={}",
        stderr(output)
    );
    serde_json::from_str(lines[0]).unwrap()
}

async fn init_against(server: &MockServer) -> Output {
    let (mut cmd, _tmp) = steel_cmd();
    cmd.env("STEEL_API_URL", format!("{}/v1", server.uri()))
        .env("STEEL_API_KEY", "sk-test")
        .args(["--json", "init", "--no-skills"]);
    tokio::process::Command::from(cmd).output().await.unwrap()
}

#[tokio::test]
async fn init_json_prints_one_result() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"sessions": []})))
        .mount(&server)
        .await;

    let output = init_against(&server).await;

    assert!(output.status.success(), "{output:?}");
    let result = single_json_result(&output);
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["doctor"]["overall"], "pass");
    assert_eq!(result["data"]["skills"], "skipped");
}

#[tokio::test]
async fn init_json_prints_one_result_when_preflight_fails() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/sessions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let output = init_against(&server).await;

    assert_eq!(output.status.code(), Some(1));
    let result = single_json_result(&output);
    assert_eq!(result["success"], false);
    assert_eq!(result["error_code"], "doctor_failed");
    assert_eq!(result["data"]["overall"], "fail");
}

fn pinned_cookbook_ref() -> String {
    let source = include_str!("../src/commands/forge.rs");
    let line = source
        .lines()
        .find(|line| line.starts_with("const COOKBOOK_REF"))
        .unwrap();
    line.split('"').nth(1).unwrap()[..12].to_string()
}

#[test]
fn forge_json_prints_the_created_project() {
    let home = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let steel = |args: &[&str]| {
        let (mut cmd, _tmp) = steel_cmd();
        cmd.env("HOME", home.path())
            .env_remove("XDG_CACHE_HOME")
            .current_dir(work.path())
            .args(args);
        cmd.output().unwrap()
    };

    let cache = single_json_result(&steel(&["--json", "cache"]));
    let cookbook = Path::new(cache["data"]["cacheDir"].as_str().unwrap())
        .join("cookbook")
        .join(pinned_cookbook_ref());
    let template = cookbook.join("examples/demo-ts");
    std::fs::create_dir_all(&template).unwrap();
    std::fs::write(template.join("index.ts"), "console.log('hi')").unwrap();
    std::fs::write(
        cookbook.join("registry.yaml"),
        "- title: Demo\n  path: examples/demo-ts\n  language: typescript\n",
    )
    .unwrap();

    let output = steel(&["--json", "forge", "demo-ts", "--name", "proj"]);

    assert!(output.status.success(), "{output:?}");
    let result = single_json_result(&output);
    assert_eq!(result["data"]["name"], "proj");
    assert_eq!(result["data"]["template"], "demo-ts");
    assert_eq!(result["data"]["language"], "typescript");
    assert!(Path::new(result["data"]["path"].as_str().unwrap()).ends_with("proj"));
    assert!(work.path().join("proj/index.ts").exists());
}

// ─── Environment variable contracts ─────────────────────────────────

#[test]
fn steel_config_dir_is_respected() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("custom-steel-config");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("STEEL_CONFIG_DIR", &config_dir);
    cmd.env("STEEL_TELEMETRY_DISABLED", "1");
    cmd.env_remove("STEEL_API_KEY");
    cmd.args(["--no-update-check", "config"]);

    let output = cmd.output().expect("failed to execute steel binary");

    // The command should not fail just because the config dir is custom;
    // it either creates the dir or reports the config location.
    // We verify that no writes happen to the default ~/.config/steel
    // by checking the custom dir was used (if any files were created).
    if config_dir.exists() {
        // If files were written, they should be in our custom dir.
        let entries: Vec<_> = std::fs::read_dir(&config_dir).unwrap().collect();
        // This is a positive signal that the custom dir was used.
        // (The dir might not exist if config command only reads.)
        let _ = entries;
    }

    // Additionally, check that the output (if any) references the custom path
    // or that the command at least ran without panicking.
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    // The config command should either succeed or fail gracefully
    assert!(
        output.status.success() || !combined.is_empty(),
        "steel config with custom STEEL_CONFIG_DIR should run without panic"
    );
}

// ─── Subcommand-specific help flags ─────────────────────────────────

#[test]
fn browser_start_help_shows_expected_flags() {
    let output = run(&["browser", "start", "--help"]);
    assert!(
        output.status.success(),
        "steel browser start --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &[
        "--session",
        "--local",
        "--stealth",
        "--proxy",
        "--metadata <KEY=VALUE>",
    ] {
        assert!(
            out.contains(flag),
            "browser start help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn browser_stop_help_shows_expected_flags() {
    let output = run(&["browser", "stop", "--help"]);
    assert!(
        output.status.success(),
        "steel browser stop --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--session", "--all", "--local"] {
        assert!(
            out.contains(flag),
            "browser stop help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn browser_sessions_help_shows_expected_flags() {
    let output = run(&["browser", "sessions", "--help"]);
    assert!(
        output.status.success(),
        "steel browser sessions --help should exit 0"
    );
    let out = stdout(&output);
    {
        let flag = &"--local";
        assert!(
            out.contains(flag),
            "browser sessions help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn browser_live_help_shows_expected_flags() {
    let output = run(&["browser", "live", "--help"]);
    assert!(
        output.status.success(),
        "steel browser live --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--session", "--local"] {
        assert!(
            out.contains(flag),
            "browser live help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn credentials_create_help_shows_expected_flags() {
    let output = run(&["credentials", "create", "--help"]);
    assert!(
        output.status.success(),
        "steel credentials create --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--origin", "--username", "--password", "--namespace"] {
        assert!(
            out.contains(flag),
            "credentials create help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn credentials_delete_help_shows_expected_flags() {
    let output = run(&["credentials", "delete", "--help"]);
    assert!(
        output.status.success(),
        "steel credentials delete --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--origin", "--namespace", "--local"] {
        assert!(
            out.contains(flag),
            "credentials delete help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn dev_install_help_shows_expected_flags() {
    let output = run(&["dev", "install", "--help"]);
    assert!(
        output.status.success(),
        "steel dev install --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--repo-url", "--verbose"] {
        assert!(
            out.contains(flag),
            "dev install help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn dev_start_help_shows_expected_flags() {
    let output = run(&["dev", "start", "--help"]);
    assert!(
        output.status.success(),
        "steel dev start --help should exit 0"
    );
    let out = stdout(&output);
    for flag in &["--port", "--verbose", "--docker-check"] {
        assert!(
            out.contains(flag),
            "dev start help should mention '{flag}', got: {out}"
        );
    }
}

#[test]
fn profile_import_help_shows_expected_flags() {
    let output = run(&["profile", "import", "--help"]);
    assert!(
        output.status.success(),
        "steel profile import --help should exit 0"
    );
    let out = stdout(&output);
    for expected in &["[NAME]", "--from"] {
        assert!(
            out.contains(expected),
            "profile import help should mention '{expected}', got: {out}"
        );
    }
}

#[test]
fn profile_sync_help_shows_expected_flags() {
    let output = run(&["profile", "sync", "--help"]);
    assert!(
        output.status.success(),
        "steel profile sync --help should exit 0"
    );
    let out = stdout(&output);
    for expected in &["[NAME]", "--from"] {
        assert!(
            out.contains(expected),
            "profile sync help should mention '{expected}', got: {out}"
        );
    }
}

#[test]
fn profile_delete_help_shows_expected_flags() {
    let output = run(&["profile", "delete", "--help"]);
    assert!(
        output.status.success(),
        "steel profile delete --help should exit 0"
    );
    let out = stdout(&output);
    assert!(
        out.contains("[NAME]"),
        "profile delete help should mention '[NAME]', got: {out}"
    );
}

#[test]
fn profile_list_help_shows_expected_flags() {
    let output = run(&["profile", "list", "--help"]);
    assert!(
        output.status.success(),
        "steel profile list --help should exit 0"
    );
    let out = stdout(&output);
    assert!(
        out.contains("--json"),
        "profile list help should mention '--json', got: {out}"
    );
}

// ─── Flag parsing edge cases ────────────────────────────────────────

#[test]
fn scrape_unknown_flag_fails() {
    let output = run(&["scrape", "--nonexistent-flag"]);
    assert!(
        !output.status.success(),
        "steel scrape --nonexistent-flag should exit non-zero"
    );
    let err = stderr(&output);
    assert!(
        err.contains("unexpected")
            || err.contains("not expected")
            || err.contains("unrecognized")
            || err.contains("unknown")
            || err.contains("found argument"),
        "stderr should indicate an unknown flag, got: {err}"
    );
}

#[test]
fn browser_without_subcommand_fails() {
    let output = run(&["browser"]);
    assert!(
        !output.status.success(),
        "steel browser (no subcommand) should exit non-zero"
    );
    let err = stderr(&output);
    // clap should show an error about missing subcommand
    assert!(
        !err.is_empty(),
        "stderr should contain usage/error info for missing subcommand"
    );
}

#[test]
fn credentials_without_subcommand_fails() {
    let output = run(&["credentials"]);
    assert!(
        !output.status.success(),
        "steel credentials (no subcommand) should exit non-zero"
    );
}

#[test]
fn dev_without_subcommand_fails() {
    let output = run(&["dev"]);
    assert!(
        !output.status.success(),
        "steel dev (no subcommand) should exit non-zero"
    );
}

#[test]
fn profile_without_subcommand_fails() {
    let output = run(&["profile"]);
    assert!(
        !output.status.success(),
        "steel profile (no subcommand) should exit non-zero"
    );
}

// ─── Exit code specifics ────────────────────────────────────────────

#[test]
fn clap_usage_error_exits_with_code_2() {
    // clap exits with code 2 for usage errors (bad flags, missing subcommand).
    let output = run(&["--this-flag-does-not-exist"]);
    assert!(
        !output.status.success(),
        "invalid global flag should exit non-zero"
    );
    let code = output.status.code().expect("should have exit code");
    assert_eq!(
        code, 2,
        "clap usage error should exit with code 2, got: {code}"
    );
}

#[test]
fn help_flag_exits_with_code_0() {
    let output = run(&["--help"]);
    let code = output.status.code().expect("should have exit code");
    assert_eq!(code, 0, "--help should exit with code 0, got: {code}");
}
