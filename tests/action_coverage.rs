//! Tracks coverage of agent-browser action names and parameters.
//!
//! When agent-browser adds new actions or new parameters to existing actions,
//! this test fails, forcing us to either implement them or explicitly skip them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Actions we have proper Clap subcommands for.
const COVERED: &[&str] = &[
    "navigate",
    "click",
    "fill",
    "type",
    "press",
    "hover",
    "scroll",
    "select",
    "check",
    "uncheck",
    "snapshot",
    "screenshot",
    "evaluate",
    "gettext",
    "getattribute",
    "isvisible",
    "isenabled",
    "ischecked",
    "wait",
    "url",
    "title",
    "back",
    "forward",
    "reload",
    "close",
    // Interactions
    "dblclick",
    "focus",
    "clear",
    "selectall",
    "scrollintoview",
    // Element queries
    "innertext",
    "innerhtml",
    "inputvalue",
    "setvalue",
    "count",
    "boundingbox",
    "styles",
    // Page
    "content",
    "find",
    "bringtofront",
    // Tab management
    "tab_list",
    "tab_new",
    "tab_switch",
    "tab_close",
    // Cookies & storage
    "cookies_get",
    "cookies_set",
    "cookies_clear",
    "storage_get",
    "storage_set",
    "storage_clear",
    // Drag & upload
    "drag",
    "upload",
    // Visual
    "highlight",
    // Browser settings
    "viewport",
    "geolocation",
    "useragent",
    "user_agent",
    "headers",
    "offline",
];

/// Actions in agent-browser that we intentionally skip (not yet wrapped).
const SKIPPED: &[&str] = &[
    // Page content
    "setcontent",
    // Interactions not yet wrapped
    "dispatch",
    "tap",
    "swipe",
    "multiselect",
    // Locators
    "getbyrole",
    "getbytext",
    "getbylabel",
    "getbyplaceholder",
    "getbyalttext",
    "getbytitle",
    "getbytestid",
    "nth",
    // Network
    "console",
    "errors",
    "route",
    "unroute",
    "requests",
    "responsebody",
    "credentials",
    // Profiling / tracing
    "trace_start",
    "trace_stop",
    "profiler_start",
    "profiler_stop",
    "screencast_start",
    "screencast_stop",
    "video_start",
    "video_stop",
    "har_start",
    "har_stop",
    // Export / capture
    "pdf",
    // Tabs
    "window_new",
    // Browser configuration
    "set_media",
    "emulatemedia",
    "timezone",
    "locale",
    "permissions",
    "download",
    // Diff
    "diff_snapshot",
    "diff_url",
    "diff_screenshot",
    // Input
    "mouse",
    "keyboard",
    "input_mouse",
    "input_keyboard",
    "input_touch",
    "keydown",
    "keyup",
    "inserttext",
    "mousemove",
    "mousedown",
    "mouseup",
    // Dialog / scripts
    "dialog",
    "addscript",
    "addinitscript",
    "addstyle",
    "clipboard",
    "wheel",
    "device",
    // Frame / eval
    "frame",
    "mainframe",
    "evalhandle",
    // Wait variants
    "waitforurl",
    "waitforloadstate",
    "waitforfunction",
    "waitfordownload",
    // Policy
    "confirm",
    "deny",
];

/// Actions that don't apply to Steel's remote session architecture.
const NOT_APPLICABLE: &[&str] = &[
    // Steel manages browser lifecycle via its API, not local launch
    "launch",
    "cdp_url",
    "inspect",
    // agent-browser's own credential/auth storage — Steel has its own
    "credentials_set",
    "credentials_get",
    "credentials_delete",
    "credentials_list",
    "auth_save",
    "auth_login",
    "auth_list",
    "auth_delete",
    "auth_show",
    // agent-browser's session state management — Steel manages sessions via API
    "state_save",
    "state_load",
    "state_list",
    "state_show",
    "state_clear",
    "state_clean",
    "state_rename",
    // agent-browser internals
    "expose",
    "pause",
    "device_list",
    // agent-browser's recording (codegen) — not applicable to remote sessions
    "recording_start",
    "recording_stop",
    "recording_restart",
];

// ── Parameter coverage per covered action ───────────────────────────

struct ParamCoverage {
    /// The handle function name in actions.rs (e.g. "handle_navigate")
    handler: &'static str,
    /// Parameters we expose as Clap flags/args
    covered: &'static [&'static str],
    /// Parameters in agent-browser we intentionally don't expose yet
    skipped: &'static [&'static str],
}

const PARAM_COVERAGE: &[ParamCoverage] = &[
    ParamCoverage {
        handler: "handle_navigate",
        covered: &["url", "waitUntil", "headers"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_click",
        covered: &["selector", "button", "clickCount", "newTab"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_fill",
        covered: &["selector", "value"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_type",
        covered: &["selector", "text", "clear", "delay"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_press",
        covered: &["key"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_hover",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_scroll",
        covered: &["selector", "x", "y", "direction", "amount"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_select",
        covered: &["selector", "values", "value"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_check",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_uncheck",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_snapshot",
        covered: &["interactive", "selector", "compact", "maxDepth", "cursor"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_screenshot",
        covered: &[
            "fullPage",
            "annotate",
            "selector",
            "format",
            "quality",
            "path",
            "screenshotDir",
            "type",
        ],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_evaluate",
        covered: &["script"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_gettext",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_getattribute",
        covered: &["selector", "attribute"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_isvisible",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_isenabled",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_ischecked",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_wait",
        covered: &[
            "timeout",
            "text",
            "selector",
            "state",
            "url",
            "function",
            "loadState",
        ],
        skipped: &[],
    },
    // url, title, back, forward, reload, close: no cmd params
    ParamCoverage {
        handler: "handle_dblclick",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_focus",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_clear",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_selectall",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_scrollintoview",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_innertext",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_innerhtml",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_inputvalue",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_setvalue",
        covered: &["selector", "value"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_count",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_boundingbox",
        covered: &["selector"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_styles",
        covered: &["selector", "properties"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_find",
        covered: &["selector"],
        skipped: &[],
    },
    // content, bringtofront: no cmd params
    // tab_list: no cmd params
    ParamCoverage {
        handler: "handle_tab_new",
        covered: &["url"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_tab_switch",
        covered: &["index"],
        skipped: &[],
    },
    ParamCoverage {
        handler: "handle_tab_close",
        covered: &["index"],
        skipped: &[],
    },
];

// ── Tests ───────────────────────────────────────────────────────────

#[test]
fn action_coverage() {
    let Some(actions_path) = find_actions_rs() else {
        eprintln!(
            "Skipping action_coverage: agent-browser actions.rs not found. \
             Set AGENT_BROWSER_SRC or ensure ../agent-browser exists."
        );
        return;
    };

    let actual_actions = extract_action_names(&actions_path);
    let known: BTreeSet<&str> = COVERED
        .iter()
        .chain(SKIPPED.iter())
        .chain(NOT_APPLICABLE.iter())
        .copied()
        .collect();

    let mut failures = Vec::new();

    for action in &actual_actions {
        if !known.contains(action.as_str()) {
            failures.push(format!(
                "New action `{action}` not in coverage list. \
                 Add to COVERED or SKIPPED."
            ));
        }
    }

    for &action in known.iter() {
        if !actual_actions.contains(action) {
            failures.push(format!(
                "Action `{action}` in coverage list but not found \
                 in execute_command(). Remove from list."
            ));
        }
    }

    if !failures.is_empty() {
        let report = failures.join("\n  ");
        panic!(
            "Action coverage check failed:\n  {report}\n\n\
             Coverage: {} covered, {} skipped, {} n/a",
            COVERED.len(),
            SKIPPED.len(),
            NOT_APPLICABLE.len(),
        );
    }

    eprintln!(
        "Action coverage: {} covered, {} skipped, {} n/a, total {}",
        COVERED.len(),
        SKIPPED.len(),
        NOT_APPLICABLE.len(),
        COVERED.len() + SKIPPED.len() + NOT_APPLICABLE.len(),
    );
}

#[test]
fn param_coverage() {
    let Some(actions_path) = find_actions_rs() else {
        eprintln!(
            "Skipping param_coverage: agent-browser actions.rs not found. \
             Set AGENT_BROWSER_SRC or ensure ../agent-browser exists."
        );
        return;
    };

    let content = std::fs::read_to_string(&actions_path).unwrap();
    let actual_params = extract_handler_params(&content);

    let mut failures = Vec::new();
    let mut total_covered = 0usize;
    let mut total_skipped = 0usize;

    for pc in PARAM_COVERAGE {
        let Some(actual) = actual_params.get(pc.handler) else {
            failures.push(format!(
                "{}: handler function not found in actions.rs",
                pc.handler,
            ));
            continue;
        };

        let known: BTreeSet<&str> = pc
            .covered
            .iter()
            .chain(pc.skipped.iter())
            .copied()
            .collect();

        // New params not in our lists
        for param in actual {
            if !known.contains(param.as_str()) {
                failures.push(format!(
                    "{}: new param `{param}` not in coverage list. \
                     Add to covered or skipped.",
                    pc.handler,
                ));
            }
        }

        // Stale entries
        for &param in &known {
            if !actual.contains(param) {
                failures.push(format!(
                    "{}: param `{param}` in coverage list but not found \
                     in handler. Remove from list.",
                    pc.handler,
                ));
            }
        }

        total_covered += pc.covered.len();
        total_skipped += pc.skipped.len();
    }

    if !failures.is_empty() {
        let report = failures.join("\n  ");
        panic!(
            "Param coverage check failed:\n  {report}\n\n\
             Coverage: {total_covered} covered, {total_skipped} skipped",
        );
    }

    eprintln!(
        "Param coverage: {total_covered} covered, {total_skipped} skipped, \
         total {} across {} handlers",
        total_covered + total_skipped,
        PARAM_COVERAGE.len(),
    );
}

// ── Helpers ─────────────────────────────────────────────────────────

fn find_actions_rs() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("AGENT_BROWSER_SRC") {
        let p = PathBuf::from(&path).join("actions.rs");
        if p.exists() {
            return Some(p);
        }
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sibling = manifest_dir
        .parent()?
        .join("agent-browser/cli/src/native/actions.rs");
    if sibling.exists() {
        return Some(sibling);
    }

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
                    let p = ref_entry.path().join("cli/src/native/actions.rs");
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Extract action names from the `match action { ... }` block in execute_command().
fn extract_action_names(path: &Path) -> BTreeSet<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let mut actions = BTreeSet::new();
    let mut in_match = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("let result = match action {") {
            in_match = true;
            continue;
        }

        if !in_match {
            continue;
        }

        if trimmed.starts_with("};") {
            break;
        }

        if let Some(arrow_pos) = trimmed.find("=>") {
            let pattern = trimmed[..arrow_pos].trim();
            for alt in pattern.split('|') {
                let alt = alt.trim().trim_matches('"');
                if !alt.is_empty() && alt != "_" {
                    actions.insert(alt.to_string());
                }
            }
        }
    }

    actions
}

/// Extract `cmd.get("param")` calls from each `handle_*` function.
/// Returns handler_name → set of param names.
///
/// Handles multi-line chains like:
/// ```ignore
///     cmd
///         .get("url")
/// ```
fn extract_handler_params(content: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut result = BTreeMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(fn_name) = parse_handler_fn_name(trimmed) {
            // Only track handlers that take `cmd` parameter
            if !trimmed.contains("cmd:") && !trimmed.contains("cmd :") {
                i += 1;
                continue;
            }

            // Collect the entire function body as one string
            let mut body = String::new();
            let mut brace_depth = 0i32;
            let mut started = false;

            #[allow(clippy::needless_range_loop)] // j updates outer index i
            for j in i..lines.len() {
                for ch in lines[j].chars() {
                    if ch == '{' {
                        brace_depth += 1;
                        started = true;
                    } else if ch == '}' {
                        brace_depth -= 1;
                    }
                }

                body.push_str(lines[j]);
                body.push(' ');

                if started && brace_depth == 0 {
                    i = j + 1;
                    break;
                }
            }

            // Normalize `cmd\s+.` → `cmd.` so multi-line chains are matched
            let normalized = normalize_cmd_chains(&body);
            let mut params = BTreeSet::new();
            extract_cmd_get_params(&normalized, &mut params);

            params.remove("action");
            params.remove("id");

            result.insert(fn_name, params);
            continue;
        }

        i += 1;
    }

    result
}

/// Parse a handler function name from a line like `async fn handle_navigate(cmd: ...`
fn parse_handler_fn_name(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("async fn ")
        .or_else(|| line.strip_prefix("fn "))?;
    if !rest.starts_with("handle_") {
        return None;
    }
    let name = rest.split('(').next()?;
    Some(name.to_string())
}

/// Collapse `cmd` + whitespace + `.` into `cmd.` so multi-line chains
/// like `cmd\n        .get("url")` become `cmd.get("url")`.
fn normalize_cmd_chains(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let bytes = body.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 3 <= bytes.len() && &bytes[i..i + 3] == b"cmd" {
            result.push_str("cmd");
            i += 3;
            // Skip whitespace between `cmd` and `.`
            let ws_start = i;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\t') {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'.' {
                // Found `cmd<ws>.` → collapse to `cmd.`
            } else {
                // Not followed by `.` — keep the whitespace
                result.push_str(&body[ws_start..i]);
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Extract all `cmd.get("key")` and `cmd["key"]` occurrences from text.
/// Expects pre-normalized text (see `normalize_cmd_chains`).
fn extract_cmd_get_params(text: &str, params: &mut BTreeSet<String>) {
    // Pattern: cmd.get("key")
    let mut rest = text;
    while let Some(pos) = rest.find("cmd.get(\"") {
        let start = pos + "cmd.get(\"".len();
        rest = &rest[start..];
        if let Some(end) = rest.find('"') {
            let key = &rest[..end];
            if !key.is_empty() {
                params.insert(key.to_string());
            }
            rest = &rest[end..];
        }
    }

    // Pattern: cmd["key"]
    rest = text;
    while let Some(pos) = rest.find("cmd[\"") {
        let start = pos + "cmd[\"".len();
        rest = &rest[start..];
        if let Some(end) = rest.find('"') {
            let key = &rest[..end];
            if !key.is_empty() {
                params.insert(key.to_string());
            }
            rest = &rest[end..];
        }
    }
}
