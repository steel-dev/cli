//! Verifies that Steel CLI's dispatch_inner echo-back JSON keys match
//! agent-browser's handler return keys for mutation commands.
//!
//! When agent-browser adds or changes return keys in a handler, this test
//! fails, forcing us to update Steel's dispatch_inner to stay in sync.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Declarative mapping: for each agent-browser handler, the JSON keys
/// Steel's dispatch_inner is expected to return.
struct OutputParity {
    /// The handler function name (e.g. "handle_click")
    handler: &'static str,
    /// JSON keys Steel returns for this command
    expected_keys: &'static [&'static str],
}

/// Commands where Steel intentionally diverges from agent-browser output
/// (e.g. agent-browser returns extra data we don't have at the daemon layer).
const SKIPPED_HANDLERS: &[&str] = &[
    // Steel doesn't wrap these yet
    "handle_launch",
    "handle_cdp_url",
    "handle_inspect",
    // Steel has its own session/state management
    "handle_credentials_set",
    "handle_credentials_get",
    "handle_credentials_delete",
    "handle_credentials_list",
    "handle_auth_save",
    "handle_auth_login",
    "handle_auth_list",
    "handle_auth_delete",
    "handle_auth_show",
    "handle_state_save",
    "handle_state_load",
    "handle_state_list",
    "handle_state_show",
    "handle_state_clear",
    "handle_state_clean",
    "handle_state_rename",
    // Steel returns subset of agent-browser's keys for these
    "handle_snapshot",
    "handle_screenshot",
    // Observation commands — Steel already matches via success_data/success_field
    "handle_evaluate",
    "handle_gettext",
    "handle_getattribute",
    "handle_isvisible",
    "handle_isenabled",
    "handle_ischecked",
    "handle_innertext",
    "handle_innerhtml",
    "handle_inputvalue",
    "handle_count",
    "handle_boundingbox",
    "handle_styles",
    "handle_content",
    "handle_find",
    // Not wrapped
    "handle_setcontent",
    "handle_dispatch",
    "handle_tap",
    "handle_swipe",
    "handle_multiselect",
    "handle_getbyrole",
    "handle_getbytext",
    "handle_getbylabel",
    "handle_getbyplaceholder",
    "handle_getbyalttext",
    "handle_getbytitle",
    "handle_getbytestid",
    "handle_nth",
    "handle_console",
    "handle_errors",
    "handle_route",
    "handle_unroute",
    "handle_requests",
    "handle_responsebody",
    "handle_credentials",
    "handle_trace_start",
    "handle_trace_stop",
    "handle_profiler_start",
    "handle_profiler_stop",
    "handle_screencast_start",
    "handle_screencast_stop",
    "handle_video_start",
    "handle_video_stop",
    "handle_har_start",
    "handle_har_stop",
    "handle_pdf",
    "handle_window_new",
    "handle_set_media",
    "handle_emulatemedia",
    "handle_timezone",
    "handle_locale",
    "handle_permissions",
    "handle_download",
    "handle_diff_snapshot",
    "handle_diff_url",
    "handle_diff_screenshot",
    "handle_mouse",
    "handle_keyboard",
    "handle_input_mouse",
    "handle_input_keyboard",
    "handle_input_touch",
    "handle_keydown",
    "handle_keyup",
    "handle_inserttext",
    "handle_mousemove",
    "handle_mousedown",
    "handle_mouseup",
    "handle_dialog",
    "handle_addscript",
    "handle_addinitscript",
    "handle_addstyle",
    "handle_clipboard",
    "handle_wheel",
    "handle_device",
    "handle_frame",
    "handle_mainframe",
    "handle_evalhandle",
    "handle_waitforurl",
    "handle_waitforloadstate",
    "handle_waitforfunction",
    "handle_waitfordownload",
    "handle_confirm",
    "handle_deny",
    "handle_expose",
    "handle_pause",
    "handle_device_list",
    "handle_recording_start",
    "handle_recording_stop",
    "handle_recording_restart",
    // Cookies get has its own output handling
    "handle_cookies_get",
    // HTTP credentials not wrapped
    "handle_http_credentials",
    // Streaming not wrapped
    "handle_stream_disable",
    "handle_stream_enable",
    "handle_stream_status",
    "handle_request_detail",
];

const OUTPUT_PARITY: &[OutputParity] = &[
    OutputParity {
        handler: "handle_navigate",
        expected_keys: &["url", "title"],
    },
    OutputParity {
        handler: "handle_click",
        expected_keys: &["clicked", "newTab", "url"],
    },
    OutputParity {
        handler: "handle_dblclick",
        expected_keys: &["clicked"],
    },
    OutputParity {
        handler: "handle_fill",
        expected_keys: &["filled"],
    },
    OutputParity {
        handler: "handle_type",
        expected_keys: &["typed"],
    },
    OutputParity {
        handler: "handle_press",
        expected_keys: &["pressed"],
    },
    OutputParity {
        handler: "handle_hover",
        expected_keys: &["hovered"],
    },
    OutputParity {
        handler: "handle_scroll",
        expected_keys: &["scrolled"],
    },
    OutputParity {
        handler: "handle_select",
        expected_keys: &["selected"],
    },
    OutputParity {
        handler: "handle_check",
        expected_keys: &["checked"],
    },
    OutputParity {
        handler: "handle_uncheck",
        expected_keys: &["unchecked"],
    },
    OutputParity {
        handler: "handle_focus",
        expected_keys: &["focused"],
    },
    OutputParity {
        handler: "handle_clear",
        expected_keys: &["cleared"],
    },
    OutputParity {
        handler: "handle_selectall",
        expected_keys: &["selected"],
    },
    OutputParity {
        handler: "handle_scrollintoview",
        expected_keys: &["scrolled"],
    },
    OutputParity {
        handler: "handle_setvalue",
        expected_keys: &["set", "value"],
    },
    OutputParity {
        handler: "handle_back",
        expected_keys: &["url"],
    },
    OutputParity {
        handler: "handle_forward",
        expected_keys: &["url"],
    },
    OutputParity {
        handler: "handle_reload",
        expected_keys: &["url"],
    },
    OutputParity {
        handler: "handle_close",
        expected_keys: &["closed"],
    },
    OutputParity {
        handler: "handle_bringtofront",
        expected_keys: &["broughtToFront"],
    },
    OutputParity {
        handler: "handle_drag",
        expected_keys: &["dragged", "source", "target"],
    },
    OutputParity {
        handler: "handle_upload",
        expected_keys: &["uploaded", "selector"],
    },
    OutputParity {
        handler: "handle_highlight",
        expected_keys: &["highlighted"],
    },
    OutputParity {
        handler: "handle_viewport",
        expected_keys: &["width", "height", "deviceScaleFactor", "mobile"],
    },
    OutputParity {
        handler: "handle_geolocation",
        expected_keys: &["latitude", "longitude"],
    },
    OutputParity {
        handler: "handle_useragent",
        expected_keys: &["userAgent"],
    },
    OutputParity {
        handler: "handle_user_agent",
        expected_keys: &["userAgent"],
    },
    OutputParity {
        handler: "handle_headers",
        expected_keys: &["set"],
    },
    OutputParity {
        handler: "handle_offline",
        expected_keys: &["offline"],
    },
    OutputParity {
        handler: "handle_cookies_set",
        expected_keys: &["set"],
    },
    OutputParity {
        handler: "handle_cookies_clear",
        expected_keys: &["cleared"],
    },
    OutputParity {
        handler: "handle_storage_set",
        expected_keys: &["set"],
    },
    OutputParity {
        handler: "handle_storage_clear",
        expected_keys: &["cleared"],
    },
    // Wait command
    OutputParity {
        handler: "handle_wait",
        expected_keys: &["waited", "ms", "selector", "state", "text", "url"],
    },
    // Tab commands
    OutputParity {
        handler: "handle_tab_list",
        expected_keys: &["tabs"],
    },
    OutputParity {
        handler: "handle_tab_new",
        expected_keys: &["url"],
    },
    OutputParity {
        handler: "handle_tab_switch",
        expected_keys: &["url"],
    },
    OutputParity {
        handler: "handle_tab_close",
        expected_keys: &["closed"],
    },
    // URL / Title
    OutputParity {
        handler: "handle_url",
        expected_keys: &["url"],
    },
    OutputParity {
        handler: "handle_title",
        expected_keys: &["title"],
    },
];

#[test]
fn output_parity() {
    let Some(actions_path) = find_actions_rs() else {
        eprintln!(
            "Skipping output_parity: agent-browser actions.rs not found. \
             Set AGENT_BROWSER_SRC or ensure ../agent-browser exists."
        );
        return;
    };

    let content = std::fs::read_to_string(&actions_path).unwrap();
    let actual_keys = extract_handler_output_keys(&content);

    let known_handlers: BTreeSet<&str> = OUTPUT_PARITY
        .iter()
        .map(|p| p.handler)
        .chain(SKIPPED_HANDLERS.iter().copied())
        .collect();

    let mut failures = Vec::new();

    // Check each parity entry against agent-browser source
    for parity in OUTPUT_PARITY {
        let Some(actual) = actual_keys.get(parity.handler) else {
            // Handler not found — might have been renamed or removed
            continue;
        };

        let expected: BTreeSet<&str> = parity.expected_keys.iter().copied().collect();

        for key in actual {
            if !expected.contains(key.as_str()) {
                failures.push(format!(
                    "{}: agent-browser returns key `{key}` not in expected_keys. \
                     Add to OUTPUT_PARITY or SKIPPED_HANDLERS.",
                    parity.handler,
                ));
            }
        }
    }

    // Check for new handlers in agent-browser not in our lists
    for handler in actual_keys.keys() {
        if !known_handlers.contains(handler.as_str()) {
            failures.push(format!(
                "New handler `{handler}` in agent-browser not tracked. \
                 Add to OUTPUT_PARITY or SKIPPED_HANDLERS."
            ));
        }
    }

    if !failures.is_empty() {
        let report = failures.join("\n  ");
        panic!(
            "Output parity check failed:\n  {report}\n\n\
             Coverage: {} tracked, {} skipped",
            OUTPUT_PARITY.len(),
            SKIPPED_HANDLERS.len(),
        );
    }

    eprintln!(
        "Output parity: {} tracked, {} skipped",
        OUTPUT_PARITY.len(),
        SKIPPED_HANDLERS.len(),
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

/// Extract JSON keys from `Ok(json!({ ... }))` return expressions in each `handle_*` function.
/// Returns handler_name -> set of top-level JSON keys.
fn extract_handler_output_keys(content: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut result = BTreeMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(fn_name) = parse_handler_fn_name(trimmed) {
            // Collect the entire function body
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

            // Extract keys from Ok(json!({ "key": ... })) patterns
            let keys = extract_json_keys_from_body(&body);
            if !keys.is_empty() {
                result.insert(fn_name, keys);
            }
            continue;
        }

        i += 1;
    }

    result
}

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

/// Scan function body for `Ok(json!({` patterns and extract the top-level keys.
/// Collects keys from ALL Ok(json!({...})) in the function (multiple return paths).
fn extract_json_keys_from_body(body: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut rest = body;

    while let Some(pos) = rest.find("Ok(json!({") {
        let start = pos + "Ok(json!({".len();
        rest = &rest[start..];

        // Extract keys from this json!({ ... }) — just top-level "key": patterns
        // We only need to scan until the matching })
        let mut depth = 1i32;
        let scan = rest;
        let mut block = String::new();

        for (idx, ch) in scan.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        block = scan[..idx].to_string();
                        rest = &scan[idx..];
                        break;
                    }
                }
                _ => {}
            }
        }

        // Extract "key": patterns from the block (top-level only, depth 0)
        let mut block_rest = block.as_str();
        while let Some(q_pos) = block_rest.find('"') {
            block_rest = &block_rest[q_pos + 1..];
            if let Some(end_q) = block_rest.find('"') {
                let key = &block_rest[..end_q];
                block_rest = &block_rest[end_q + 1..];
                // Check if followed by `:` (possibly with whitespace)
                let after = block_rest.trim_start();
                if after.starts_with(':') {
                    keys.insert(key.to_string());
                }
            } else {
                break;
            }
        }
    }

    keys
}
