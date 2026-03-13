//! Contract tests: verify that Rust CLI flags are 100% compatible with the TypeScript CLI.
//!
//! Each test defines the expected flags (extracted from TS source) and asserts
//! that the Rust clap parser exposes exactly those flags with correct names,
//! short aliases, and required/optional status.

use clap::{ArgAction, Command, CommandFactory};

// Re-use the clap-derived structs from the binary.
// We need CommandFactory to introspect without running.
use steel_cli::commands;

/// Describes a single CLI flag as defined in the TypeScript source.
#[derive(Debug)]
struct ExpectedFlag {
    /// Long name on the CLI (e.g., "use-proxy")
    long: &'static str,
    /// Short alias character, if any (e.g., 'u')
    short: Option<char>,
    /// Whether the flag is required (clap will reject the command without it)
    required: bool,
    /// Whether the flag takes a value (true) or is a boolean switch (false)
    takes_value: bool,
}

fn flag(long: &'static str) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: None,
        required: false,
        takes_value: false,
    }
}

fn flag_val(long: &'static str) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: None,
        required: false,
        takes_value: true,
    }
}

fn flag_short(long: &'static str, short: char) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: Some(short),
        required: false,
        takes_value: false,
    }
}

fn flag_val_short(long: &'static str, short: char) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: Some(short),
        required: false,
        takes_value: true,
    }
}

/// Assert that a clap Command has exactly the expected flags.
fn assert_flags(cmd: &Command, expected: &[ExpectedFlag], cmd_name: &str) {
    // Collect actual flags (skip "help" and "version" — clap adds those automatically)
    let skip = ["help", "version", "no-update-check"];

    let actual_args: Vec<_> = cmd
        .get_arguments()
        .filter(|a| !skip.contains(&a.get_id().as_str()))
        .filter(|a| a.get_long().is_some()) // skip positional args
        .collect();

    // Check each expected flag exists
    for ef in expected {
        let actual = actual_args
            .iter()
            .find(|a| a.get_long().map(|l| l == ef.long).unwrap_or(false));

        let actual = match actual {
            Some(a) => a,
            None => {
                panic!("[{cmd_name}] Missing expected flag --{}", ef.long);
            }
        };

        // Check short alias
        let actual_short = actual.get_short();
        assert_eq!(
            actual_short, ef.short,
            "[{cmd_name}] --{}: expected short={:?}, got short={:?}",
            ef.long, ef.short, actual_short
        );

        // Check required
        let actual_required = actual.is_required_set();
        assert_eq!(
            actual_required, ef.required,
            "[{cmd_name}] --{}: expected required={}, got required={}",
            ef.long, ef.required, actual_required
        );

        // Check takes_value via action type (Set/Append take values; SetTrue/SetFalse/Count don't)
        let actual_takes_value = matches!(actual.get_action(), ArgAction::Set | ArgAction::Append);
        assert_eq!(
            actual_takes_value, ef.takes_value,
            "[{cmd_name}] --{}: expected takes_value={}, got takes_value={}",
            ef.long, ef.takes_value, actual_takes_value
        );
    }

    // Check no unexpected flags
    for actual in &actual_args {
        let long = actual.get_long().unwrap_or("");
        let found = expected.iter().any(|ef| ef.long == long);
        if !found {
            panic!("[{cmd_name}] Unexpected flag --{long} not in TypeScript CLI");
        }
    }
}

/// Get a subcommand from the top-level CLI command.
fn get_subcommand(root: &Command, path: &[&str]) -> Command {
    let mut cmd = root.clone();
    for name in path {
        let next = cmd
            .get_subcommands()
            .find(|s| s.get_name() == *name)
            .unwrap_or_else(|| panic!("subcommand '{}' not found", name))
            .clone();
        cmd = next;
    }
    cmd
}

// ─── Tests ───────────────────────────────────────────────────────────────────

fn root_cmd() -> Command {
    commands::Cli::command()
}

/// Verify the complete subcommand tree matches the TypeScript CLI.
#[test]
fn subcommand_tree_matches_ts() {
    let root = root_cmd();
    let top_level: Vec<&str> = root
        .get_subcommands()
        .map(|s| s.get_name())
        .filter(|n| *n != "help")
        .collect();

    let expected_top = vec![
        "scrape",
        "screenshot",
        "pdf",
        "browser",
        "login",
        "logout",
        "credentials",
        "dev",
        "forge",
        "run",
        "config",
        "update",
        "cache",
        "docs",
        "star",
        "support",
        "settings",
    ];

    for name in &expected_top {
        assert!(
            top_level.contains(name),
            "Missing top-level command: {name}"
        );
    }
    for name in &top_level {
        assert!(
            expected_top.contains(name),
            "Unexpected top-level command: {name}"
        );
    }

    // browser subcommands
    let browser = get_subcommand(&root, &["browser"]);
    let browser_subs: Vec<&str> = browser
        .get_subcommands()
        .map(|s| s.get_name())
        .filter(|n| *n != "help")
        .collect();
    let expected_browser = vec!["start", "stop", "sessions", "live", "captcha"];
    for name in &expected_browser {
        assert!(
            browser_subs.contains(name),
            "Missing browser subcommand: {name}"
        );
    }
    for name in &browser_subs {
        assert!(
            expected_browser.contains(name),
            "Unexpected browser subcommand: {name}"
        );
    }

    // browser captcha subcommands
    let captcha = get_subcommand(&root, &["browser", "captcha"]);
    let captcha_subs: Vec<&str> = captcha
        .get_subcommands()
        .map(|s| s.get_name())
        .filter(|n| *n != "help")
        .collect();
    let expected_captcha = vec!["solve", "status"];
    for name in &expected_captcha {
        assert!(
            captcha_subs.contains(name),
            "Missing captcha subcommand: {name}"
        );
    }
    for name in &captcha_subs {
        assert!(
            expected_captcha.contains(name),
            "Unexpected captcha subcommand: {name}"
        );
    }

    // credentials subcommands
    let creds = get_subcommand(&root, &["credentials"]);
    let creds_subs: Vec<&str> = creds
        .get_subcommands()
        .map(|s| s.get_name())
        .filter(|n| *n != "help")
        .collect();
    let expected_creds = vec!["list", "create", "update", "delete"];
    for name in &expected_creds {
        assert!(
            creds_subs.contains(name),
            "Missing credentials subcommand: {name}"
        );
    }
    for name in &creds_subs {
        assert!(
            expected_creds.contains(name),
            "Unexpected credentials subcommand: {name}"
        );
    }

    // dev subcommands
    let dev = get_subcommand(&root, &["dev"]);
    let dev_subs: Vec<&str> = dev
        .get_subcommands()
        .map(|s| s.get_name())
        .filter(|n| *n != "help")
        .collect();
    let expected_dev = vec!["install", "start", "stop"];
    for name in &expected_dev {
        assert!(
            dev_subs.contains(name),
            "Missing dev subcommand: {name}"
        );
    }
    for name in &dev_subs {
        assert!(
            expected_dev.contains(name),
            "Unexpected dev subcommand: {name}"
        );
    }
}

#[test]
fn scrape_flags() {
    let cmd = get_subcommand(&root_cmd(), &["scrape"]);
    assert_flags(
        &cmd,
        &[
            // positional `url` is not a flag
            flag_val_short("url", 'u'),
            flag_val("format"),
            flag("raw"),
            flag_val_short("delay", 'd'),
            flag("pdf"),
            flag("screenshot"),
            flag("use-proxy"),
            flag_val_short("region", 'r'),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "scrape",
    );
}

#[test]
fn screenshot_flags() {
    let cmd = get_subcommand(&root_cmd(), &["screenshot"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("url", 'u'),
            flag_val_short("delay", 'd'),
            flag_short("full-page", 'f'),
            flag("use-proxy"),
            flag_val_short("region", 'r'),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "screenshot",
    );
}

#[test]
fn pdf_flags() {
    let cmd = get_subcommand(&root_cmd(), &["pdf"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("url", 'u'),
            flag_val_short("delay", 'd'),
            flag("use-proxy"),
            flag_val_short("region", 'r'),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "pdf",
    );
}

#[test]
fn browser_start_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "start"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("session", 's'),
            flag_short("local", 'l'),
            flag_val("api-url"),
            flag("stealth"),
            flag_val_short("proxy", 'p'),
            flag_val("session-timeout"),
            flag_val("session-headless"),
            flag_val("session-region"),
            flag("session-solve-captcha"),
            flag_val("namespace"),
            flag("credentials"),
        ],
        "browser start",
    );
}

#[test]
fn browser_stop_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "stop"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("session", 's'),
            flag_short("all", 'a'),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "browser stop",
    );
}

#[test]
fn browser_sessions_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "sessions"]);
    assert_flags(
        &cmd,
        &[flag_short("local", 'l'), flag_val("api-url"), flag("raw")],
        "browser sessions",
    );
}

#[test]
fn browser_live_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "live"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("session", 's'),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "browser live",
    );
}

#[test]
fn browser_captcha_solve_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "captcha", "solve"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("session", 's'),
            flag_val("session-id"),
            flag_val("page-id"),
            flag_val("url"),
            flag_val("task-id"),
            flag("raw"),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "browser captcha solve",
    );
}

#[test]
fn browser_captcha_status_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "captcha", "status"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("session", 's'),
            flag_val("session-id"),
            flag_val("page-id"),
            flag_short("wait", 'w'),
            flag_val("timeout"),
            flag_val("interval"),
            flag("raw"),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "browser captcha status",
    );
}

#[test]
fn credentials_list_flags() {
    let cmd = get_subcommand(&root_cmd(), &["credentials", "list"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("namespace", 'n'),
            flag_val("origin"),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "credentials list",
    );
}

#[test]
fn credentials_create_flags() {
    let cmd = get_subcommand(&root_cmd(), &["credentials", "create"]);
    assert_flags(
        &cmd,
        &[
            flag_val("origin"),
            flag_val_short("username", 'u'),
            flag_val_short("password", 'p'),
            flag_val("totp-secret"),
            flag_val_short("namespace", 'n'),
            flag_val("label"),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "credentials create",
    );
}

#[test]
fn credentials_update_flags() {
    let cmd = get_subcommand(&root_cmd(), &["credentials", "update"]);
    assert_flags(
        &cmd,
        &[
            flag_val("origin"),
            flag_val_short("username", 'u'),
            flag_val_short("password", 'p'),
            flag_val("totp-secret"),
            flag_val_short("namespace", 'n'),
            flag_val("label"),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "credentials update",
    );
}

#[test]
fn credentials_delete_flags() {
    let cmd = get_subcommand(&root_cmd(), &["credentials", "delete"]);
    assert_flags(
        &cmd,
        &[
            flag_val("origin"),
            flag_val_short("namespace", 'n'),
            flag_short("local", 'l'),
            flag_val("api-url"),
        ],
        "credentials delete",
    );
}

#[test]
fn dev_install_flags() {
    let cmd = get_subcommand(&root_cmd(), &["dev", "install"]);
    assert_flags(
        &cmd,
        &[flag_val("repo-url"), flag_short("verbose", 'V')],
        "dev install",
    );
}

#[test]
fn dev_start_flags() {
    let cmd = get_subcommand(&root_cmd(), &["dev", "start"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("port", 'p'),
            flag_short("verbose", 'V'),
            flag_short("docker-check", 'd'),
        ],
        "dev start",
    );
}

#[test]
fn dev_stop_flags() {
    let cmd = get_subcommand(&root_cmd(), &["dev", "stop"]);
    assert_flags(&cmd, &[flag_short("verbose", 'V')], "dev stop");
}

#[test]
fn forge_flags() {
    let cmd = get_subcommand(&root_cmd(), &["forge"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("name", 'n'),
            flag_val_short("api-url", 'a'),
            flag_val("api-key"),
            flag_val("openai-key"),
            flag_val("anthropic-key"),
            flag("skip-auth"),
        ],
        "forge",
    );
}

#[test]
fn run_flags() {
    let cmd = get_subcommand(&root_cmd(), &["run"]);
    assert_flags(
        &cmd,
        &[
            flag_val_short("task", 't'),
            flag_short("view", 'o'),
            flag_val_short("api-url", 'a'),
            flag_val("api-key"),
            flag_val("openai-key"),
            flag_val("anthropic-key"),
            flag_val("gemini-key"),
            flag("skip-auth"),
        ],
        "run",
    );
}

#[test]
fn cache_flags() {
    let cmd = get_subcommand(&root_cmd(), &["cache"]);
    assert_flags(&cmd, &[flag_short("clean", 'c')], "cache");
}

#[test]
fn docs_flags() {
    let cmd = get_subcommand(&root_cmd(), &["docs"]);
    assert_flags(&cmd, &[], "docs");
}

#[test]
fn star_flags() {
    let cmd = get_subcommand(&root_cmd(), &["star"]);
    assert_flags(&cmd, &[], "star");
}

#[test]
fn support_flags() {
    let cmd = get_subcommand(&root_cmd(), &["support"]);
    assert_flags(&cmd, &[], "support");
}

#[test]
fn settings_flags() {
    let cmd = get_subcommand(&root_cmd(), &["settings"]);
    assert_flags(&cmd, &[], "settings");
}

#[test]
fn login_has_auth_alias() {
    let root = root_cmd();
    let login = root
        .get_subcommands()
        .find(|s| s.get_name() == "login")
        .expect("login command not found");
    let aliases: Vec<&str> = login.get_all_aliases().collect();
    assert!(
        aliases.contains(&"auth"),
        "login command should have 'auth' alias, got: {:?}",
        aliases
    );
}
