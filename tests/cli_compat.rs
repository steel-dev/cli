//! Contract tests: verify that Rust CLI flags are 100% compatible with the TypeScript CLI.
//!
//! Each test defines the expected flags (extracted from TS source) and asserts
//! that the Rust clap parser exposes exactly those flags with correct names,
//! short aliases, and required/optional status.

use std::collections::BTreeSet;
use std::path::Path;

use clap::{ArgAction, Command, CommandFactory};

// Re-use the clap-derived structs from the binary.
// We need CommandFactory to introspect without running.
use steel_cli::commands;

/// Scan a Pastel commands directory and return the set of command names.
/// Convention: .tsx files (excluding index.tsx) are commands, directories are parent commands.
fn discover_pastel_commands(dir: &Path) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return commands,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_file() && name_str.ends_with(".tsx") && name_str != "index.tsx" {
            commands.insert(name_str.trim_end_matches(".tsx").to_string());
        } else if path.is_dir() {
            commands.insert(name_str.to_string());
        }
    }
    commands
}

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

fn flag_val_required(long: &'static str) -> ExpectedFlag {
    ExpectedFlag {
        long,
        short: None,
        required: true,
        takes_value: true,
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
/// Commands are discovered dynamically from the Pastel source/commands/ directory.
#[test]
fn subcommand_tree_matches_ts() {
    let ts_commands_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("source/commands");

    // Discover top-level TS commands from filesystem
    let ts_top = discover_pastel_commands(&ts_commands_dir);
    assert!(
        !ts_top.is_empty(),
        "Failed to discover TS commands from {}",
        ts_commands_dir.display()
    );

    // Get Rust top-level commands
    let root = root_cmd();
    let rust_top: BTreeSet<String> = root
        .get_subcommands()
        .filter(|s| !s.is_hide_set())
        .map(|s| s.get_name().to_string())
        .filter(|n| n != "help")
        .collect();

    // Check both directions
    for name in &ts_top {
        assert!(
            rust_top.contains(name),
            "TS has command '{name}' but Rust does not"
        );
    }
    for name in &rust_top {
        assert!(
            ts_top.contains(name),
            "Rust has command '{name}' but TS does not"
        );
    }

    // Recurse into directories (subcommands)
    for name in &ts_top {
        let sub_dir = ts_commands_dir.join(name);
        if !sub_dir.is_dir() {
            continue;
        }

        let ts_subs = discover_pastel_commands(&sub_dir);
        let rust_parent = get_subcommand(&root, &[name.as_str()]);
        let rust_subs: BTreeSet<String> = rust_parent
            .get_subcommands()
            .map(|s| s.get_name().to_string())
            .filter(|n| n != "help")
            .collect();

        for sub in &ts_subs {
            assert!(
                rust_subs.contains(sub),
                "TS has '{name} {sub}' but Rust does not"
            );
        }
        for sub in &rust_subs {
            // Browser action subcommands (navigate, click, …) are Rust-native
            // and have no TS equivalent — skip the reverse check for them.
            if name == "browser" && !ts_subs.contains(sub) {
                continue;
            }
            assert!(
                ts_subs.contains(sub),
                "Rust has '{name} {sub}' but TS does not"
            );
        }

        // One more level (e.g., browser/captcha/)
        for sub in &ts_subs {
            let nested_dir = sub_dir.join(sub);
            if !nested_dir.is_dir() {
                continue;
            }

            let ts_nested = discover_pastel_commands(&nested_dir);
            let rust_nested_parent =
                get_subcommand(&root, &[name.as_str(), sub.as_str()]);
            let rust_nested: BTreeSet<String> = rust_nested_parent
                .get_subcommands()
                .map(|s| s.get_name().to_string())
                .filter(|n| n != "help")
                .collect();

            for n in &ts_nested {
                assert!(
                    rust_nested.contains(n),
                    "TS has '{name} {sub} {n}' but Rust does not"
                );
            }
            for n in &rust_nested {
                assert!(
                    ts_nested.contains(n),
                    "Rust has '{name} {sub} {n}' but TS does not"
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

#[test]
fn profile_list_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "list"]);
    assert_flags(&cmd, &[flag("json")], "profile list");
}

#[test]
fn profile_import_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "import"]);
    assert_flags(
        &cmd,
        &[flag_val_required("name"), flag_val("from")],
        "profile import",
    );
}

#[test]
fn profile_sync_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "sync"]);
    assert_flags(
        &cmd,
        &[flag_val_required("name"), flag_val("from")],
        "profile sync",
    );
}

#[test]
fn profile_delete_flags() {
    let cmd = get_subcommand(&root_cmd(), &["profile", "delete"]);
    assert_flags(&cmd, &[flag_val_required("name")], "profile delete");
}
