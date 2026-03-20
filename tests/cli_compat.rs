//! Contract tests: verify that Rust CLI flags are compatible with the frozen CLI spec.
//!
//! Each test defines the expected flags and asserts that the Rust clap parser
//! exposes exactly those flags with correct names, short aliases, and required/optional status.

use std::collections::BTreeSet;
use std::path::Path;

use clap::{ArgAction, Command, CommandFactory};

use steel_cli::commands;

/// Describes a single CLI flag.
#[derive(Debug)]
struct ExpectedFlag {
    long: &'static str,
    short: Option<char>,
    required: bool,
    takes_value: bool,
}

const fn flag(long: &'static str) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: None,
        required: false,
        takes_value: false,
    }
}

const fn flag_val(long: &'static str) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: None,
        required: false,
        takes_value: true,
    }
}

const fn flag_short(long: &'static str, short: char) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: Some(short),
        required: false,
        takes_value: false,
    }
}

const fn flag_val_required(long: &'static str) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: None,
        required: true,
        takes_value: true,
    }
}

const fn flag_val_short(long: &'static str, short: char) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: Some(short),
        required: false,
        takes_value: true,
    }
}

/// Assert that a clap Command has exactly the expected visible flags.
fn assert_flags(cmd: &Command, expected: &[ExpectedFlag], cmd_name: &str) {
    let skip = [
        "help",
        "version",
        "json",
        "no-update-check",
        "local",
        "api-url",
    ];

    let actual_args: Vec<_> = cmd
        .get_arguments()
        .filter(|a| !skip.contains(&a.get_id().as_str()))
        .filter(|a| a.get_long().is_some()) // skip positional args
        .filter(|a| !a.is_hide_set()) // skip hidden args
        .collect();

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

        let actual_short = actual.get_short();
        assert_eq!(
            actual_short, ef.short,
            "[{cmd_name}] --{}: expected short={:?}, got short={:?}",
            ef.long, ef.short, actual_short
        );

        let actual_required = actual.is_required_set();
        assert_eq!(
            actual_required, ef.required,
            "[{cmd_name}] --{}: expected required={}, got required={}",
            ef.long, ef.required, actual_required
        );

        let actual_takes_value = matches!(actual.get_action(), ArgAction::Set | ArgAction::Append);
        assert_eq!(
            actual_takes_value, ef.takes_value,
            "[{cmd_name}] --{}: expected takes_value={}, got takes_value={}",
            ef.long, ef.takes_value, actual_takes_value
        );
    }

    for actual in &actual_args {
        let long = actual.get_long().unwrap_or("");
        let found = expected.iter().any(|ef| ef.long == long);
        if !found {
            panic!("[{cmd_name}] Unexpected flag --{long} not in spec");
        }
    }
}

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

fn root_cmd() -> Command {
    commands::Cli::command()
}

/// Verify the complete subcommand tree matches the spec.
#[test]
fn subcommand_tree_matches_spec() {
    let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cli-spec.json");
    let spec_text = std::fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("Failed to read cli-spec.json: {e}"));
    let spec: serde_json::Value = serde_json::from_str(&spec_text)
        .unwrap_or_else(|e| panic!("Failed to parse cli-spec.json: {e}"));

    let spec_cmds = spec
        .get("commands")
        .and_then(|v| v.as_array())
        .expect("spec should have a 'commands' array");

    let spec_top: BTreeSet<String> = spec_cmds
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .map(|s| s.to_string())
        .collect();
    assert!(!spec_top.is_empty(), "Spec has no commands");

    let root = root_cmd();
    let rust_top: BTreeSet<String> = root
        .get_subcommands()
        .filter(|s| !s.is_hide_set())
        .map(|s| s.get_name().to_string())
        .filter(|n| n != "help")
        .collect();

    for name in &spec_top {
        assert!(
            rust_top.contains(name),
            "Spec has command '{name}' but Rust does not"
        );
    }

    for cmd in spec_cmds {
        let name = match cmd.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => continue,
        };
        let subs = match cmd.get("subcommands").and_then(|v| v.as_array()) {
            Some(s) => s,
            None => continue,
        };

        let spec_subs: BTreeSet<String> = subs
            .iter()
            .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
            .map(|s| s.to_string())
            .collect();

        let rust_parent = get_subcommand(&root, &[name]);
        let rust_subs: BTreeSet<String> = rust_parent
            .get_subcommands()
            .map(|s| s.get_name().to_string())
            .filter(|n| n != "help")
            .collect();

        for sub in &spec_subs {
            assert!(
                rust_subs.contains(sub),
                "Spec has '{name} {sub}' but Rust does not"
            );
        }

        for sub_cmd in subs {
            let sub_name = match sub_cmd.get("name").and_then(|n| n.as_str()) {
                Some(n) => n,
                None => continue,
            };
            let nested = match sub_cmd.get("subcommands").and_then(|v| v.as_array()) {
                Some(n) => n,
                None => continue,
            };

            let spec_nested: BTreeSet<String> = nested
                .iter()
                .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                .map(|s| s.to_string())
                .collect();

            let rust_nested_parent = get_subcommand(&root, &[name, sub_name]);
            let rust_nested: BTreeSet<String> = rust_nested_parent
                .get_subcommands()
                .map(|s| s.get_name().to_string())
                .filter(|n| n != "help")
                .collect();

            for n in &spec_nested {
                assert!(
                    rust_nested.contains(n),
                    "Spec has '{name} {sub_name} {n}' but Rust does not"
                );
            }
        }
    }
}

#[test]
fn scrape_flags() {
    let cmd = get_subcommand(&root_cmd(), &["scrape"]);
    assert_flags(
        &cmd,
        &[
            flag_val("format"),
            flag_val_short("delay", 'd'),
            flag("pdf"),
            flag("screenshot"),
            flag("use-proxy"),
            flag_val_short("region", 'r'),
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
            flag_val_short("delay", 'd'),
            flag_short("full-page", 'f'),
            flag("use-proxy"),
            flag_val_short("region", 'r'),
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
            flag_val_short("delay", 'd'),
            flag("use-proxy"),
            flag_val_short("region", 'r'),
        ],
        "pdf",
    );
}

#[test]
fn browser_session_flag() {
    // --session is a global flag on the browser command, propagated to all subcommands
    let cmd = get_subcommand(&root_cmd(), &["browser"]);
    assert_flags(&cmd, &[flag_val("session")], "browser");
}

#[test]
fn browser_start_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "start"]);
    assert_flags(
        &cmd,
        &[
            flag("stealth"),
            flag_val_short("proxy", 'p'),
            flag_val("session-timeout"),
            flag("session-solve-captcha"),
            flag_val("profile"),
            flag("update-profile"),
            flag_val("namespace"),
            flag("credentials"),
        ],
        "browser start",
    );
}

#[test]
fn browser_stop_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "stop"]);
    assert_flags(&cmd, &[flag_short("all", 'a')], "browser stop");
}

#[test]
fn browser_sessions_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "sessions"]);
    assert_flags(&cmd, &[], "browser sessions");
}

#[test]
fn browser_live_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "live"]);
    assert_flags(&cmd, &[], "browser live");
}

#[test]
fn browser_captcha_solve_flags() {
    let cmd = get_subcommand(&root_cmd(), &["browser", "captcha", "solve"]);
    assert_flags(
        &cmd,
        &[
            flag_val("session-id"),
            flag_val("page-id"),
            flag_val("url"),
            flag_val("task-id"),
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
            flag_val("session-id"),
            flag_val("page-id"),
            flag_short("wait", 'w'),
            flag_val("timeout"),
            flag_val("interval"),
        ],
        "browser captcha status",
    );
}

#[test]
fn credentials_list_flags() {
    let cmd = get_subcommand(&root_cmd(), &["credentials", "list"]);
    assert_flags(
        &cmd,
        &[flag_val_short("namespace", 'n'), flag_val("origin")],
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
        ],
        "credentials update",
    );
}

#[test]
fn credentials_delete_flags() {
    let cmd = get_subcommand(&root_cmd(), &["credentials", "delete"]);
    assert_flags(
        &cmd,
        &[flag_val("origin"), flag_val_short("namespace", 'n')],
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
    assert_flags(&cmd, &[flag_val_short("name", 'n')], "forge");
}

#[test]
fn update_flags() {
    let cmd = get_subcommand(&root_cmd(), &["update"]);
    assert_flags(
        &cmd,
        &[flag_short("force", 'f'), flag_short("check", 'c')],
        "update",
    );
}

#[test]
fn cache_flags() {
    let cmd = get_subcommand(&root_cmd(), &["cache"]);
    assert_flags(&cmd, &[flag_short("clean", 'c')], "cache");
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

#[test]
fn profile_list_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "list"]);
    assert_flags(&cmd, &[], "profile list");
}

#[test]
fn profile_import_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "import"]);
    assert_flags(
        &cmd,
        &[
            flag_val_required("name"),
            flag_val("from"),
            flag_val("browser"),
            flag("full"),
        ],
        "profile import",
    );
}

#[test]
fn profile_sync_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "sync"]);
    assert_flags(
        &cmd,
        &[
            flag_val_required("name"),
            flag_val("from"),
            flag_val("browser"),
            flag("full"),
        ],
        "profile sync",
    );
}

#[test]
fn profile_delete_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "delete"]);
    assert_flags(&cmd, &[flag_val_required("name")], "profile delete");
}
