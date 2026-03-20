//! Tests CLI flag parity between agent-browser and Steel browser actions.
//!
//! For each covered browser action that has CLI flags, this test:
//! 1. Auto-extracts flag strings from agent-browser's commands.rs
//! 2. Verifies every flag is in our covered or skipped list (catches additions)
//! 3. Verifies our Clap definitions have equivalent flags with aliases/shorts
//!    for zero-friction migration (i.e. agent-browser flag names just work).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use clap::{Command, CommandFactory};
use steel_cli::commands;

// ── Flag coverage declarations ──────────────────────────────────────

#[allow(dead_code)]
struct FlagParity {
    /// Command name in agent-browser's parse_command match arm
    command: &'static str,
    /// Path from "browser" to Steel subcommand (e.g., &["snapshot"])
    steel_path: &'static [&'static str],
    /// Inline flags (in parse_command match arm):
    /// (agent-browser flag string, Steel long flag name)
    inline_covered: &'static [(&'static str, &'static str)],
    /// Inline flags we intentionally skip
    inline_skipped: &'static [&'static str],
    /// Global flags (parsed in Flags struct, not in match arm):
    /// (agent-browser flag string, Steel long flag name)
    global_covered: &'static [(&'static str, &'static str)],
    /// Global flags we intentionally skip (documented for reference,
    /// not auto-verified since they're not in parse_command match arms)
    global_skipped: &'static [&'static str],
}

const FLAG_PARITY: &[FlagParity] = &[
    FlagParity {
        command: "snapshot",
        steel_path: &["snapshot"],
        inline_covered: &[
            ("-i", "interactive"),
            ("--interactive", "interactive"),
            ("-c", "compact"),
            ("--compact", "compact"),
            ("-C", "cursor"),
            ("--cursor", "cursor"),
            ("-d", "max-depth"),
            ("--depth", "max-depth"),
            ("-s", "selector"),
            ("--selector", "selector"),
        ],
        inline_skipped: &[],
        global_covered: &[],
        global_skipped: &[],
    },
    FlagParity {
        command: "click",
        steel_path: &["click"],
        inline_covered: &[("--new-tab", "new-tab")],
        inline_skipped: &[],
        global_covered: &[],
        global_skipped: &[],
    },
    FlagParity {
        command: "scroll",
        steel_path: &["scroll"],
        inline_covered: &[("-s", "selector"), ("--selector", "selector")],
        inline_skipped: &[],
        global_covered: &[],
        global_skipped: &[],
    },
    FlagParity {
        command: "wait",
        steel_path: &["wait"],
        inline_covered: &[
            ("--timeout", "timeout"),
            ("--text", "text"),
            ("-t", "text"),
            ("--url", "url"),
            ("-u", "url"),
            ("--fn", "function"),
            ("-f", "function"),
            ("--load", "load-state"),
            ("-l", "load-state"),
        ],
        inline_skipped: &["--download", "-d"],
        global_covered: &[],
        global_skipped: &[],
    },
    FlagParity {
        command: "screenshot",
        steel_path: &["screenshot"],
        // Screenshot flags are global in agent-browser (Flags struct),
        // so they won't appear in the screenshot match arm.
        inline_covered: &[],
        inline_skipped: &[],
        global_covered: &[
            ("--full", "full-page"),
            ("--annotate", "annotate"),
            ("--screenshot-format", "format"),
            ("--screenshot-quality", "quality"),
        ],
        // -f is the global short for --full in agent-browser; in Steel's
        // subcommand architecture we use --full as alias instead.
        // --screenshot-dir: not exposed as CLI flag (hardcoded to None).
        global_skipped: &["-f", "--screenshot-dir"],
    },
    FlagParity {
        command: "navigate",
        steel_path: &["navigate"],
        inline_covered: &[],
        inline_skipped: &[],
        // --headers is a global flag in agent-browser
        global_covered: &[("--headers", "header")],
        global_skipped: &[],
    },
    FlagParity {
        command: "eval",
        steel_path: &["eval"],
        inline_covered: &[],
        inline_skipped: &["-b", "--base64", "--stdin"],
        global_covered: &[],
        global_skipped: &[],
    },
];

// ── Tests ───────────────────────────────────────────────────────────

/// Verifies that every flag in agent-browser's commands.rs for covered commands
/// is in our inline_covered or inline_skipped list. Catches new flag additions.
#[test]
fn inline_flag_coverage() {
    let Some(commands_path) = find_commands_rs() else {
        eprintln!(
            "Skipping inline_flag_coverage: agent-browser commands.rs not found. \
             Ensure ../agent-browser exists or cargo built the git dep."
        );
        return;
    };

    let content = std::fs::read_to_string(&commands_path).unwrap();
    let extracted = extract_command_flags(&content);

    let mut failures = Vec::new();

    for parity in FLAG_PARITY {
        let Some(actual_flags) = extracted.get(parity.command) else {
            // Command might not have any inline flags (e.g., screenshot uses globals).
            if !parity.inline_covered.is_empty() || !parity.inline_skipped.is_empty() {
                failures.push(format!(
                    "{}: has declared inline flags but command not found in \
                     parse_command match block",
                    parity.command,
                ));
            }
            continue;
        };

        let known: BTreeSet<&str> = parity
            .inline_covered
            .iter()
            .map(|(ab, _)| *ab)
            .chain(parity.inline_skipped.iter().copied())
            .collect();

        // New flags in agent-browser not in our list
        for flag in actual_flags {
            if !known.contains(flag.as_str()) {
                failures.push(format!(
                    "{}: new inline flag `{}` not in coverage. \
                     Add to inline_covered or inline_skipped.",
                    parity.command, flag,
                ));
            }
        }

        // Stale entries in our list
        for &flag in &known {
            if !actual_flags.contains(flag) {
                failures.push(format!(
                    "{}: inline flag `{}` declared but not found in commands.rs. \
                     Remove from list.",
                    parity.command, flag,
                ));
            }
        }
    }

    if !failures.is_empty() {
        let report = failures.join("\n  ");
        panic!("Inline flag coverage check failed:\n  {report}");
    }

    eprintln!(
        "Inline flag coverage: {} commands checked",
        FLAG_PARITY.len(),
    );
}

/// Verifies that our Clap definitions have the right flags, aliases, and shorts
/// so that agent-browser flag names work as-is in Steel CLI.
#[test]
fn steel_flag_parity() {
    let root = commands::Cli::command();
    let browser = get_subcommand(&root, &["browser"]);

    let mut failures = Vec::new();

    for parity in FLAG_PARITY {
        let steel_cmd = get_subcommand(&browser, parity.steel_path);

        let all_covered: Vec<_> = parity
            .inline_covered
            .iter()
            .chain(parity.global_covered.iter())
            .collect();

        for &(ab_flag, steel_long) in &all_covered {
            // Find the Steel arg by long name
            let steel_arg = steel_cmd
                .get_arguments()
                .find(|a| a.get_long() == Some(steel_long));

            let Some(arg) = steel_arg else {
                failures.push(format!(
                    "[{}] Steel --{} not found (needed for agent-browser `{}`)",
                    parity.command, steel_long, ab_flag,
                ));
                continue;
            };

            if let Some(ab_long) = ab_flag.strip_prefix("--") {
                if ab_long != *steel_long {
                    // Different long name → must be an alias
                    let has_alias = arg
                        .get_all_aliases()
                        .into_iter()
                        .flatten()
                        .any(|a| a == ab_long);
                    if !has_alias {
                        failures.push(format!(
                            "[{}] Steel --{} needs alias `--{}` for agent-browser compat",
                            parity.command, steel_long, ab_long,
                        ));
                    }
                }
            } else if ab_flag.starts_with('-') && ab_flag.len() == 2 {
                let ab_short = ab_flag.chars().nth(1).unwrap();
                if arg.get_short() != Some(ab_short) {
                    failures.push(format!(
                        "[{}] Steel --{} needs short `{}` for agent-browser compat \
                         (currently {:?})",
                        parity.command,
                        steel_long,
                        ab_flag,
                        arg.get_short().map(|c| format!("-{c}")),
                    ));
                }
            }
        }
    }

    if !failures.is_empty() {
        let report = failures.join("\n  ");
        panic!("Steel flag parity check failed:\n  {report}");
    }

    eprintln!(
        "Steel flag parity: {} commands, {} flags verified",
        FLAG_PARITY.len(),
        FLAG_PARITY
            .iter()
            .map(|p| p.inline_covered.len() + p.global_covered.len())
            .sum::<usize>(),
    );
}

// ── Source extraction ───────────────────────────────────────────────

/// Extract flag strings from each command's match arm in parse_command().
/// Returns command_name → set of flag strings (e.g., "--text", "-t").
fn extract_command_flags(content: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut result = BTreeMap::new();

    // Find the parse_command function
    let fn_start = match content.find("fn parse_command") {
        Some(pos) => pos,
        None => return result,
    };
    let rest = &content[fn_start..];

    // Find `match cmd` or `match cmd.as_str()`
    let match_pos = match rest.find("match cmd") {
        Some(pos) => pos,
        None => return result,
    };
    let rest = &rest[match_pos..];

    // Find the opening brace of the match
    let brace_pos = match rest.find('{') {
        Some(pos) => pos,
        None => return result,
    };
    let body = &rest[brace_pos + 1..];

    // Parse match arms with brace depth tracking.
    // depth 0 = at match body level (where arm patterns appear)
    let mut depth = 0i32;
    let mut current_cmds: Vec<String> = vec![];

    for line in body.lines() {
        let trimmed = line.trim();
        let depth_at_start = depth;

        // Track brace depth (simplified: balanced braces in strings/macros
        // cancel out, so this works in practice)
        for ch in trimmed.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }

        // Exited match block
        if depth < 0 {
            break;
        }

        // At match body level, look for arm patterns
        if depth_at_start == 0
            && let Some(cmds) = parse_arm_commands(trimmed)
        {
            current_cmds = cmds;
            for cmd in &current_cmds {
                result.entry(cmd.clone()).or_insert_with(BTreeSet::new);
            }
        }

        // Extract flag literals from this line and attribute to current commands
        if !current_cmds.is_empty() {
            for flag in extract_flag_literals(trimmed) {
                for cmd in &current_cmds {
                    if let Some(set) = result.get_mut(cmd) {
                        set.insert(flag.clone());
                    }
                }
            }
        }
    }

    result
}

/// Parse a match arm pattern like `"open" | "goto" | "navigate" => {`.
/// Returns command names if this looks like a top-level match arm.
fn parse_arm_commands(line: &str) -> Option<Vec<String>> {
    let arrow_pos = line.find("=>")?;
    let before = &line[..arrow_pos];

    let mut cmds = vec![];
    let mut rest = before;
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('"') {
            let val = &rest[..end];
            if !val.is_empty()
                && !val.starts_with('-')
                && val.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                cmds.push(val.to_string());
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }

    if cmds.is_empty() { None } else { Some(cmds) }
}

/// Extract flag-like string literals from a line.
/// Matches `"--flag-name"` and `"-x"` patterns.
fn extract_flag_literals(line: &str) -> Vec<String> {
    let mut flags = vec![];
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('"') {
            let val = &rest[..end];
            let is_long_flag = val.starts_with("--")
                && val.len() > 2
                && val[2..].chars().all(|c| c.is_alphanumeric() || c == '-');
            let is_short_flag = val.starts_with('-')
                && val.len() == 2
                && val.chars().nth(1).is_some_and(|c| c.is_alphabetic());
            if is_long_flag || is_short_flag {
                flags.push(val.to_string());
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    flags
}

// ── Shared helpers ──────────────────────────────────────────────────

fn get_subcommand(root: &Command, path: &[&str]) -> Command {
    let mut cmd = root.clone();
    for name in path {
        let next = cmd
            .get_subcommands()
            .find(|s| s.get_name() == *name || s.get_all_aliases().any(|a| a == *name))
            .unwrap_or_else(|| panic!("subcommand '{}' not found", name))
            .clone();
        cmd = next;
    }
    cmd
}

fn find_commands_rs() -> Option<PathBuf> {
    // Check explicit env var
    if let Ok(path) = std::env::var("AGENT_BROWSER_SRC") {
        let p = PathBuf::from(&path).join("commands.rs");
        if p.exists() {
            return Some(p);
        }
    }

    // Check sibling directory
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sibling = manifest_dir
        .parent()?
        .join("agent-browser/cli/src/commands.rs");
    if sibling.exists() {
        return Some(sibling);
    }

    // Check cargo git checkouts
    let home = dirs::home_dir()?;
    let checkouts = home.join(".cargo/git/checkouts");
    if let Ok(entries) = std::fs::read_dir(&checkouts) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("agent-browser-")
                && let Ok(refs) = std::fs::read_dir(entry.path())
            {
                for ref_entry in refs.flatten() {
                    let p = ref_entry.path().join("cli/src/commands.rs");
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}
