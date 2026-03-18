//! Tracks coverage of agent-browser native module public API.
//!
//! When agent-browser adds new public functions, this test fails,
//! forcing us to either wrap them or explicitly skip them.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Each module we track: the source file and its known functions.
struct ModuleCoverage {
    file: &'static str,
    /// Functions wrapped in BrowserEngine / exposed via DaemonCommand.
    covered: &'static [&'static str],
    /// Functions intentionally not wrapped (internal, infra, or out-of-scope).
    skipped: &'static [&'static str],
}

// ── interaction.rs ──────────────────────────────────────────────────

const INTERACTION: ModuleCoverage = ModuleCoverage {
    file: "interaction.rs",
    covered: &[
        "click",
        "hover",
        "fill",
        "type_text",
        "press_key",
        "scroll",
        "select_option",
        "check",
        "uncheck",
        "dblclick",
        "focus",
        "clear",
        "select_all",
        "scroll_into_view",
    ],
    skipped: &[
        "press_key_with_modifiers", // TODO: modifier key combos
        "dispatch_event",
        "highlight",
        "tap_touch",
    ],
};

// ── element.rs ──────────────────────────────────────────────────────

const ELEMENT: ModuleCoverage = ModuleCoverage {
    file: "element.rs",
    covered: &[
        // RefMap used internally by BrowserEngine
        "new",
        "add",
        "add_selector",
        "get",
        "entries_sorted",
        "clear",
        "next_ref_num",
        "set_next_ref_num",
        "parse_ref",
        "resolve_element_object_id",
        // Element queries
        "get_element_text",
        "get_element_attribute",
        "is_element_visible",
        "is_element_enabled",
        "is_element_checked",
        "get_element_inner_text",
        "get_element_inner_html",
        "get_element_input_value",
        "set_element_value",
        "get_element_count",
        "get_element_bounding_box",
        "get_element_styles",
    ],
    skipped: &["resolve_element_center"],
};

// ── cookies.rs ──────────────────────────────────────────────────────

const COOKIES: ModuleCoverage = ModuleCoverage {
    file: "cookies.rs",
    covered: &[],
    skipped: &["get_cookies", "set_cookies", "clear_cookies"],
};

// ── storage.rs ──────────────────────────────────────────────────────

const STORAGE: ModuleCoverage = ModuleCoverage {
    file: "storage.rs",
    covered: &[],
    skipped: &["storage_get", "storage_set", "storage_clear"],
};

// ── screenshot.rs ───────────────────────────────────────────────────

const SCREENSHOT: ModuleCoverage = ModuleCoverage {
    file: "screenshot.rs",
    covered: &["take_screenshot"],
    skipped: &[],
};

// ── snapshot.rs ─────────────────────────────────────────────────────

const SNAPSHOT: ModuleCoverage = ModuleCoverage {
    file: "snapshot.rs",
    covered: &["take_snapshot"],
    skipped: &[],
};

// ── network.rs ──────────────────────────────────────────────────────

const NETWORK: ModuleCoverage = ModuleCoverage {
    file: "network.rs",
    covered: &["set_extra_headers"],
    skipped: &[
        // EventTracker methods
        "new",
        "add_console",
        "add_error",
        "get_console_json",
        "get_errors_json",
        // DomainFilter methods
        "is_allowed",
        "check_url",
        // Free functions
        "set_offline",
        "set_content",
        "sanitize_existing_pages",
        "install_domain_filter",
        "install_domain_filter_script",
        "install_domain_filter_fetch",
    ],
};

// ── state.rs ────────────────────────────────────────────────────────

const STATE: ModuleCoverage = ModuleCoverage {
    file: "state.rs",
    covered: &[],
    skipped: &[
        "save_state",
        "load_state",
        "state_list",
        "state_show",
        "state_clear",
        "state_clean",
        "state_rename",
        "find_auto_state_file",
        "get_sessions_dir",
    ],
};

// ── diff.rs ─────────────────────────────────────────────────────────

const DIFF: ModuleCoverage = ModuleCoverage {
    file: "diff.rs",
    covered: &[],
    skipped: &[
        "diff_screenshot",
        "diff_snapshots",
        "diff_text",
        "diff_unified",
    ],
};

// ── browser.rs (BrowserManager + helpers) ───────────────────────────

const BROWSER: ModuleCoverage = ModuleCoverage {
    file: "browser.rs",
    covered: &[
        "connect_cdp",
        "active_session_id",
        "navigate",
        "get_url",
        "get_title",
        "evaluate",
        "close",
        "is_connection_alive",
        "tab_new",
        "tab_list",
        "tab_switch",
        "tab_close",
        "wait_for_lifecycle_external",
        "get_content",
        "bring_to_front",
    ],
    skipped: &[
        // Launch / connect
        "launch",
        "connect_auto",
        "validate_launch_options",
        "to_ai_friendly_error",
        // Page internals
        "enable_domains_pub",
        "ensure_page",
        "update_active_page_if_needed",
        "add_page",
        "remove_page_by_target_id",
        "has_target",
        "has_pages",
        "page_count",
        "pages_list",
        // Getters
        "get_cdp_url",
        "chrome_host_port",
        "active_target_id",
        "is_cdp_connection",
        // Browser configuration
        "set_viewport",
        "set_user_agent",
        "set_emulated_media",
        "set_timezone",
        "set_locale",
        "set_geolocation",
        "set_download_behavior",
        // Dialog / file / permissions
        "handle_dialog",
        "upload_files",
        "grant_permissions",
        "add_script_to_evaluate",
        // BrowserState methods
        "from_str",
        "kill",
        "wait_or_kill",
    ],
};

// ── Test ────────────────────────────────────────────────────────────

const ALL_MODULES: &[&ModuleCoverage] = &[
    &INTERACTION,
    &ELEMENT,
    &COOKIES,
    &STORAGE,
    &SCREENSHOT,
    &SNAPSHOT,
    &NETWORK,
    &STATE,
    &DIFF,
    &BROWSER,
];

#[test]
fn native_api_coverage() {
    let Some(native_src) = find_native_src() else {
        eprintln!(
            "Skipping native_api_coverage: agent-browser source not found. \
             Set AGENT_BROWSER_SRC or ensure ../agent-browser exists."
        );
        return;
    };

    let mut total_covered = 0usize;
    let mut total_skipped = 0usize;
    let mut failures = Vec::new();

    for module in ALL_MODULES {
        let path = native_src.join(module.file);
        if !path.exists() {
            failures.push(format!(
                "{}: file not found at {}",
                module.file,
                path.display()
            ));
            continue;
        }

        let actual_fns = extract_pub_fns(&path);
        let known: BTreeSet<&str> = module
            .covered
            .iter()
            .chain(module.skipped.iter())
            .copied()
            .collect();

        // Check for new functions not in our lists
        for func in &actual_fns {
            if !known.contains(func.as_str()) {
                failures.push(format!(
                    "{}: new function `{func}` not in coverage list. \
                     Add to `covered` or `skipped`.",
                    module.file
                ));
            }
        }

        // Check for stale entries (function removed upstream)
        for func in &known {
            if !actual_fns.contains(*func) {
                failures.push(format!(
                    "{}: function `{func}` in coverage list but not found in source. \
                     Remove from list.",
                    module.file
                ));
            }
        }

        total_covered += module.covered.len();
        total_skipped += module.skipped.len();
    }

    if !failures.is_empty() {
        let report = failures.join("\n  ");
        panic!(
            "Native API coverage check failed:\n  {report}\n\n\
             Coverage: {total_covered} covered, {total_skipped} skipped"
        );
    }

    eprintln!(
        "Native API coverage: {total_covered} covered, {total_skipped} skipped, \
         total {total}",
        total = total_covered + total_skipped
    );
}

// ── Helpers ─────────────────────────────────────────────────────────

fn find_native_src() -> Option<PathBuf> {
    // Env var override
    if let Ok(path) = std::env::var("AGENT_BROWSER_SRC") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Sibling directory (workspace layout)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sibling = manifest_dir.parent()?.join("agent-browser/cli/src/native");
    if sibling.exists() {
        return Some(sibling);
    }

    // Cargo git checkouts
    let home = dirs::home_dir()?;
    let checkouts = home.join(".cargo/git/checkouts");
    if let Ok(entries) = std::fs::read_dir(&checkouts) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("agent-browser-")
            {
                if let Ok(refs) = std::fs::read_dir(entry.path()) {
                    for ref_entry in refs.flatten() {
                        let native = ref_entry.path().join("cli/src/native");
                        if native.exists() {
                            return Some(native);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract public function names from a Rust source file.
/// Catches both free functions and methods in impl blocks.
fn extract_pub_fns(path: &Path) -> BTreeSet<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let mut fns = BTreeSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip lines inside #[cfg(test)] modules
        // (simple heuristic: won't catch all cases but good enough)
        if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("#[test]") {
            continue;
        }

        // Match: pub fn name( or pub async fn name(
        let rest = if let Some(r) = trimmed.strip_prefix("pub async fn ") {
            Some(r)
        } else if let Some(r) = trimmed.strip_prefix("pub fn ") {
            Some(r)
        } else {
            None
        };

        if let Some(rest) = rest {
            if let Some(name) = rest.split(['(', '<', ' ']).next() {
                let name = name.trim();
                if !name.is_empty() {
                    fns.insert(name.to_string());
                }
            }
        }
    }

    fns
}
