//! Browser action handler: routes Clap subcommands to a persistent daemon
//! that holds the BrowserEngine (CDP connection + RefMap) alive between calls.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::api::client::SteelClient;
use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::process;
use crate::browser::daemon::protocol::DaemonCommand;
use crate::browser::lifecycle::to_session_summary;
use crate::config::session_state::{SessionStatePaths, read_state};
use crate::util::{api, output};

// ── Shared arg types ────────────────────────────────────────────────

#[derive(Parser)]
pub struct SelectorArg {
    /// Element selector or ref (e.g. @e1)
    pub selector: String,
}

// ── Top-level action enum ───────────────────────────────────────────

#[derive(Subcommand)]
pub enum ActionCommand {
    // --- Navigation ---
    /// Navigate to a URL
    #[command(aliases = ["open", "goto"])]
    Navigate(NavigateArgs),
    /// Navigate back in history
    Back,
    /// Navigate forward in history
    Forward,
    /// Reload the current page
    Reload,

    // --- Interactions ---
    /// Click an element
    Click(ClickArgs),
    /// Double-click an element
    #[command(name = "dblclick")]
    DblClick(SelectorArg),
    /// Fill an input field (clears existing value first)
    Fill(FillArgs),
    /// Type text into an element (appends to existing value)
    #[command(name = "type")]
    Type(TypeArgs),
    /// Press a keyboard key
    #[command(aliases = ["key"])]
    Press(PressArgs),
    /// Hover over an element
    Hover(SelectorArg),
    /// Focus an element
    Focus(SelectorArg),
    /// Check a checkbox or radio button
    Check(SelectorArg),
    /// Uncheck a checkbox
    Uncheck(SelectorArg),
    /// Select option(s) from a dropdown
    Select(SelectArgs),
    /// Clear an input field
    Clear(SelectorArg),
    /// Select all text in an input
    #[command(name = "selectall")]
    SelectAll(SelectorArg),
    /// Scroll the page or an element
    Scroll(ScrollArgs),
    /// Scroll an element into view
    #[command(name = "scrollintoview")]
    ScrollIntoView(SelectorArg),
    /// Set the value of an input element (without triggering events)
    #[command(name = "setvalue")]
    SetValue(SetValueArgs),

    // --- Observation ---
    /// Take an accessibility tree snapshot
    Snapshot(SnapshotArgs),
    /// Take a screenshot
    Screenshot(ScreenshotArgs),
    /// Evaluate JavaScript in the page
    Eval(EvalArgs),
    /// Find all elements matching a selector
    Find(SelectorArg),
    /// Get the page HTML content
    Content,

    // --- Get info ---
    /// Get information about elements or page
    Get {
        #[command(subcommand)]
        command: GetCommand,
    },

    // --- Check state ---
    /// Check element state
    Is {
        #[command(subcommand)]
        command: IsCommand,
    },

    // --- Waiting ---
    /// Wait for a condition (text, selector, URL, function, or timeout)
    Wait(WaitArgs),

    // --- Tabs ---
    /// Manage tabs
    Tab {
        #[command(subcommand)]
        command: TabCommand,
    },

    // --- Window ---
    /// Bring the browser window to the foreground
    #[command(name = "bringtofront")]
    BringToFront,

    // --- Session ---
    /// Close the browser session
    #[command(aliases = ["quit", "exit"])]
    Close,
}

// ── Get subcommands ─────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum GetCommand {
    /// Get element text content
    Text(SelectorArg),
    /// Get element inner HTML
    Html(SelectorArg),
    /// Get input/textarea value
    Value(SelectorArg),
    /// Get element attribute value
    Attr(GetAttrArgs),
    /// Get current page URL
    Url,
    /// Get current page title
    Title,
    /// Count matching elements
    Count(SelectorArg),
    /// Get element bounding box
    #[command(name = "box")]
    Box(SelectorArg),
    /// Get element CSS styles
    Styles(StylesArgs),
}

// ── Is subcommands ──────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum IsCommand {
    /// Check if element is visible
    Visible(SelectorArg),
    /// Check if element is enabled
    Enabled(SelectorArg),
    /// Check if element is checked
    Checked(SelectorArg),
}

// ── Tab subcommands ─────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum TabCommand {
    /// List open tabs
    List,
    /// Open a new tab
    New(TabNewArgs),
    /// Switch to a tab by index
    Switch(TabSwitchArgs),
    /// Close a tab (active tab if no index given)
    Close(TabCloseArgs),
}

// ── Arg structs ─────────────────────────────────────────────────────

#[derive(Parser)]
pub struct NavigateArgs {
    /// URL to navigate to
    pub url: String,
    /// Wait condition: load, domcontentloaded, networkidle
    #[arg(long)]
    pub wait_until: Option<String>,
    /// Set request header (repeatable, format: "Key: Value")
    #[arg(long = "header", alias = "headers")]
    pub headers: Vec<String>,
}

#[derive(Parser)]
pub struct ClickArgs {
    /// Element selector or ref (e.g. @e1)
    pub selector: String,
    /// Mouse button: left, right, middle
    #[arg(long)]
    pub button: Option<String>,
    /// Number of clicks (2 for double-click)
    #[arg(long)]
    pub count: Option<i32>,
    /// Open the link in a new tab instead of clicking
    #[arg(long)]
    pub new_tab: bool,
}

#[derive(Parser)]
pub struct FillArgs {
    /// Element selector or ref
    pub selector: String,
    /// Value to fill
    #[arg(trailing_var_arg = true, num_args = 1..)]
    pub value: Vec<String>,
}

#[derive(Parser)]
pub struct TypeArgs {
    /// Element selector or ref
    pub selector: String,
    /// Text to type
    #[arg(trailing_var_arg = true, num_args = 1..)]
    pub text: Vec<String>,
    /// Clear the field before typing
    #[arg(long)]
    pub clear: bool,
    /// Delay between keystrokes in milliseconds
    #[arg(long)]
    pub delay: Option<u64>,
}

#[derive(Parser)]
pub struct PressArgs {
    /// Key to press (e.g. Enter, Escape, Tab, Control+a)
    pub key: String,
}

#[derive(Parser)]
pub struct SelectArgs {
    /// Select element selector or ref
    pub selector: String,
    /// Option value(s) to select
    #[arg(num_args = 1..)]
    pub values: Vec<String>,
}

#[derive(Parser)]
pub struct ScrollArgs {
    /// Direction: up, down, left, right
    pub direction: Option<String>,
    /// Scroll amount in pixels (default: 300)
    pub amount: Option<f64>,
    /// Element selector to scroll
    #[arg(long, short)]
    pub selector: Option<String>,
}

#[derive(Parser)]
pub struct SetValueArgs {
    /// Element selector or ref
    pub selector: String,
    /// Value to set
    #[arg(trailing_var_arg = true, num_args = 1..)]
    pub value: Vec<String>,
}

#[derive(Parser)]
pub struct SnapshotArgs {
    /// Show only interactive elements
    #[arg(short, long)]
    pub interactive: bool,
    /// Restrict snapshot to a subtree
    #[arg(short, long)]
    pub selector: Option<String>,
    /// Use compact output format
    #[arg(short, long)]
    pub compact: bool,
    /// Maximum nesting depth
    #[arg(short = 'd', long, alias = "depth")]
    pub max_depth: Option<usize>,
    /// Include cursor position
    #[arg(short = 'C', long)]
    pub cursor: bool,
}

#[derive(Parser)]
pub struct ScreenshotArgs {
    /// Capture the full scrollable page
    #[arg(long, alias = "full")]
    pub full_page: bool,
    /// Output file path
    #[arg(short, long, default_value = "screenshot.png")]
    pub output: String,
    /// Restrict screenshot to an element
    #[arg(long)]
    pub selector: Option<String>,
    /// Image format: png, jpeg, webp
    #[arg(long, alias = "type", alias = "screenshot-format")]
    pub format: Option<String>,
    /// JPEG/WebP quality (0-100)
    #[arg(long, alias = "screenshot-quality")]
    pub quality: Option<i32>,
    /// Annotate interactive elements on the screenshot
    #[arg(long)]
    pub annotate: bool,
}

#[derive(Parser)]
pub struct EvalArgs {
    /// JavaScript expression to evaluate
    #[arg(trailing_var_arg = true, num_args = 1..)]
    pub script: Vec<String>,
}

#[derive(Parser)]
pub struct GetAttrArgs {
    /// Element selector or ref
    pub selector: String,
    /// Attribute name
    pub attribute: String,
}

#[derive(Parser)]
pub struct StylesArgs {
    /// Element selector or ref
    pub selector: String,
    /// CSS property names to query (returns all computed styles if omitted)
    #[arg(long)]
    pub property: Vec<String>,
}

#[derive(Parser)]
pub struct WaitArgs {
    /// Timeout in milliseconds
    #[arg(long, default_value_t = 30000)]
    pub timeout: u64,
    /// Wait for text to appear on page
    #[arg(short = 't', long)]
    pub text: Option<String>,
    /// Wait for a CSS selector
    #[arg(long)]
    pub selector: Option<String>,
    /// Selector state: visible, hidden, attached, detached
    #[arg(long)]
    pub state: Option<String>,
    /// Wait for URL to contain this string
    #[arg(short = 'u', long)]
    pub url: Option<String>,
    /// Wait for a JS function to return truthy
    #[arg(short = 'f', long, alias = "fn")]
    pub function: Option<String>,
    /// Wait for load state: load, domcontentloaded, networkidle
    #[arg(short = 'l', long, alias = "load")]
    pub load_state: Option<String>,
}

#[derive(Parser)]
pub struct TabNewArgs {
    /// URL to open (defaults to about:blank)
    pub url: Option<String>,
}

#[derive(Parser)]
pub struct TabSwitchArgs {
    /// Tab index to switch to
    pub index: usize,
}

#[derive(Parser)]
pub struct TabCloseArgs {
    /// Tab index to close (closes active tab if omitted)
    pub index: Option<usize>,
}

// ── Conversion helpers (pure functions, tested below) ───────────────

fn parse_header_args(headers: &[String]) -> Option<HashMap<String, String>> {
    if headers.is_empty() {
        return None;
    }
    let mut map = HashMap::new();
    for h in headers {
        if let Some((k, v)) = h.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Some(map)
}

fn scroll_deltas(direction: Option<&str>, amount: Option<f64>) -> (f64, f64) {
    let dir = direction.unwrap_or("down");
    let amt = amount.unwrap_or(300.0);
    match dir {
        "up" => (0.0, -amt),
        "down" => (0.0, amt),
        "left" => (-amt, 0.0),
        "right" => (amt, 0.0),
        _ => (0.0, amt),
    }
}

fn resolve_output_path(output: &str) -> Result<String> {
    let path = std::path::Path::new(output);
    if path.is_absolute() {
        Ok(output.to_string())
    } else {
        Ok(std::env::current_dir()?
            .join(path)
            .to_string_lossy()
            .to_string())
    }
}

fn optional_vec(v: Vec<String>) -> Option<Vec<String>> {
    if v.is_empty() { None } else { Some(v) }
}

// ── Command dispatch ────────────────────────────────────────────────

pub async fn run(action: ActionCommand, session: Option<&str>) -> Result<()> {
    let mut client = ensure_daemon(session).await?;

    let result = dispatch_action(&mut client, action).await;
    if let Err(ref err) = result {
        // On failure, check if the remote session is still alive.
        // If not, clear stale state and give a helpful error.
        if let Some(enriched) = check_session_health(session, err).await {
            return Err(enriched);
        }
    }
    result
}

async fn dispatch_action(client: &mut DaemonClient, action: ActionCommand) -> Result<()> {
    match action {
        // --- Navigation ---
        ActionCommand::Navigate(args) => {
            client
                .send(DaemonCommand::Navigate {
                    url: args.url,
                    wait_until: args.wait_until,
                    headers: parse_header_args(&args.headers),
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Back => {
            client.send(DaemonCommand::Back).await?;
            output::success_silent();
        }
        ActionCommand::Forward => {
            client.send(DaemonCommand::Forward).await?;
            output::success_silent();
        }
        ActionCommand::Reload => {
            client.send(DaemonCommand::Reload).await?;
            output::success_silent();
        }

        // --- Interactions ---
        ActionCommand::Click(args) => {
            client
                .send(DaemonCommand::Click {
                    selector: args.selector,
                    button: args.button,
                    click_count: args.count,
                    new_tab: args.new_tab,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::DblClick(args) => {
            client
                .send(DaemonCommand::DblClick {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Fill(args) => {
            let value = args.value.join(" ");
            client
                .send(DaemonCommand::Fill {
                    selector: args.selector,
                    value,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Type(args) => {
            let text = args.text.join(" ");
            client
                .send(DaemonCommand::TypeText {
                    selector: args.selector,
                    text,
                    clear: args.clear,
                    delay_ms: args.delay,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Press(args) => {
            client.send(DaemonCommand::Press { key: args.key }).await?;
            output::success_silent();
        }
        ActionCommand::Hover(args) => {
            client
                .send(DaemonCommand::Hover {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Focus(args) => {
            client
                .send(DaemonCommand::Focus {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Check(args) => {
            client
                .send(DaemonCommand::Check {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Uncheck(args) => {
            client
                .send(DaemonCommand::Uncheck {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Select(args) => {
            client
                .send(DaemonCommand::Select {
                    selector: args.selector,
                    values: args.values,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Clear(args) => {
            client
                .send(DaemonCommand::Clear {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::SelectAll(args) => {
            client
                .send(DaemonCommand::SelectAll {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::Scroll(args) => {
            let (dx, dy) = scroll_deltas(args.direction.as_deref(), args.amount);
            client
                .send(DaemonCommand::Scroll {
                    selector: args.selector,
                    delta_x: dx,
                    delta_y: dy,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::ScrollIntoView(args) => {
            client
                .send(DaemonCommand::ScrollIntoView {
                    selector: args.selector,
                })
                .await?;
            output::success_silent();
        }
        ActionCommand::SetValue(args) => {
            let value = args.value.join(" ");
            client
                .send(DaemonCommand::SetValue {
                    selector: args.selector,
                    value,
                })
                .await?;
            output::success_silent();
        }

        // --- Observation ---
        ActionCommand::Snapshot(args) => {
            let data = client
                .send(DaemonCommand::Snapshot {
                    interactive_only: args.interactive,
                    selector: args.selector,
                    compact: args.compact,
                    max_depth: args.max_depth,
                    cursor: args.cursor,
                })
                .await?;
            output::success_text(data);
        }
        ActionCommand::Screenshot(args) => {
            let abs_path = resolve_output_path(&args.output)?;
            let data = client
                .send(DaemonCommand::Screenshot {
                    full_page: args.full_page,
                    selector: args.selector,
                    format: args.format,
                    quality: args.quality,
                    annotate: args.annotate,
                    path: Some(abs_path),
                    screenshot_dir: None,
                })
                .await?;
            if output::is_json() {
                output::success_data(data);
            } else {
                let saved_path = data["path"].as_str().unwrap_or(&args.output);
                println!("{saved_path}");
            }
        }
        ActionCommand::Eval(args) => {
            let script = args.script.join(" ");
            let data = client.send(DaemonCommand::Eval { script }).await?;
            output::success_data(data);
        }
        ActionCommand::Find(args) => {
            let data = client
                .send(DaemonCommand::Find {
                    selector: args.selector,
                })
                .await?;
            if output::is_json() {
                output::success_data(data);
            } else if let Some(elements) = data["elements"].as_array() {
                for el in elements {
                    let idx = el["index"].as_u64().unwrap_or(0);
                    let tag = el["tagName"].as_str().unwrap_or("");
                    let text = el["text"].as_str().unwrap_or("");
                    let visible = el["visible"].as_bool().unwrap_or(false);
                    let vis = if visible { "" } else { " (hidden)" };
                    println!("[{idx}] <{tag}>{vis} {text}");
                }
            }
        }
        ActionCommand::Content => {
            let data = client.send(DaemonCommand::Content).await?;
            output::success_text(data);
        }

        // --- Get info ---
        ActionCommand::Get { command } => match command {
            GetCommand::Text(args) => {
                let data = client
                    .send(DaemonCommand::GetText {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "text");
            }
            GetCommand::Html(args) => {
                let data = client
                    .send(DaemonCommand::InnerHtml {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "html");
            }
            GetCommand::Value(args) => {
                let data = client
                    .send(DaemonCommand::InputValue {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "value");
            }
            GetCommand::Attr(args) => {
                let data = client
                    .send(DaemonCommand::GetAttribute {
                        selector: args.selector,
                        attribute: args.attribute,
                    })
                    .await?;
                output::success_field(data, "value");
            }
            GetCommand::Url => {
                let data = client.send(DaemonCommand::Url).await?;
                output::success_text(data);
            }
            GetCommand::Title => {
                let data = client.send(DaemonCommand::Title).await?;
                output::success_text(data);
            }
            GetCommand::Count(args) => {
                let data = client
                    .send(DaemonCommand::Count {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "count");
            }
            GetCommand::Box(args) => {
                let data = client
                    .send(DaemonCommand::BoundingBox {
                        selector: args.selector,
                    })
                    .await?;
                output::success_data(data);
            }
            GetCommand::Styles(args) => {
                let data = client
                    .send(DaemonCommand::Styles {
                        selector: args.selector,
                        properties: optional_vec(args.property),
                    })
                    .await?;
                output::success_data(data);
            }
        },

        // --- Check state ---
        ActionCommand::Is { command } => match command {
            IsCommand::Visible(args) => {
                let data = client
                    .send(DaemonCommand::IsVisible {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "visible");
            }
            IsCommand::Enabled(args) => {
                let data = client
                    .send(DaemonCommand::IsEnabled {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "enabled");
            }
            IsCommand::Checked(args) => {
                let data = client
                    .send(DaemonCommand::IsChecked {
                        selector: args.selector,
                    })
                    .await?;
                output::success_field(data, "checked");
            }
        },

        // --- Wait ---
        ActionCommand::Wait(args) => {
            let data = client
                .send(DaemonCommand::Wait {
                    timeout: Some(args.timeout),
                    text: args.text,
                    selector: args.selector,
                    state: args.state,
                    url: args.url,
                    function: args.function,
                    load_state: args.load_state,
                })
                .await?;
            output::success_field(data, "waited");
        }

        // --- Tabs ---
        ActionCommand::Tab { command } => match command {
            TabCommand::List => {
                let data = client.send(DaemonCommand::TabList).await?;
                if output::is_json() {
                    output::success_data(data);
                } else if let Some(tabs) = data["tabs"].as_array() {
                    for tab in tabs {
                        let idx = tab["index"].as_u64().unwrap_or(0);
                        let title = tab["title"].as_str().unwrap_or("");
                        let url = tab["url"].as_str().unwrap_or("");
                        let active = tab["active"].as_bool().unwrap_or(false);
                        let marker = if active { " *" } else { "" };
                        println!("[{idx}]{marker} {title} — {url}");
                    }
                }
            }
            TabCommand::New(args) => {
                let data = client.send(DaemonCommand::TabNew { url: args.url }).await?;
                output::success_field(data, "url");
            }
            TabCommand::Switch(args) => {
                let data = client
                    .send(DaemonCommand::TabSwitch { index: args.index })
                    .await?;
                output::success_field(data, "url");
            }
            TabCommand::Close(args) => {
                client
                    .send(DaemonCommand::TabClose { index: args.index })
                    .await?;
                output::success_silent();
            }
        },

        // --- Window ---
        ActionCommand::BringToFront => {
            client.send(DaemonCommand::BringToFront).await?;
            output::success_silent();
        }

        // --- Session ---
        ActionCommand::Close => {
            client.send(DaemonCommand::Close).await?;
            output::success_silent();
        }
    }

    Ok(())
}

/// Ensure a daemon is running for the target session and return a connected client.
/// If no daemon exists, resolves the CDP URL via the API and spawns one.
///
/// When `session_name` is Some, resolves the named session from state.
/// Otherwise falls back to the active session.
async fn ensure_daemon(session_name: Option<&str>) -> Result<DaemonClient> {
    let paths = SessionStatePaths::default_paths();
    let state = read_state(&paths.state_path);

    let session_id = if let Some(name) = session_name {
        state
            .resolve_candidate(api::mode(), Some(name))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No session found for \"{name}\". Start one with `steel browser start --session {name}`."
                )
            })?
    } else {
        state.active_session_id.as_deref().ok_or_else(|| {
            anyhow::anyhow!("No active browser session. Run `steel browser start` first.")
        })?
    };

    // Fast path: daemon already running
    if let Ok(client) = DaemonClient::connect(session_id).await {
        return Ok(client);
    }

    // Slow path: resolve CDP URL and spawn daemon
    let (mode, base_url, auth_info) = api::resolve_with_auth();

    let api_client = SteelClient::new()?;
    let session = api_client
        .get_session(&base_url, mode, session_id, &auth_info)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let summary = to_session_summary(&session, mode, None, &auth_info)?;
    let cdp_url = summary
        .connect_url
        .ok_or_else(|| anyhow::anyhow!("Session {} has no CDP connect URL.", session_id))?;

    process::spawn_daemon(session_id, &cdp_url)?;
    process::wait_for_daemon(session_id, Duration::from_secs(10)).await?;

    DaemonClient::connect(session_id).await
}

/// On action failure, check whether the remote session is still alive.
/// If the session is dead/expired, clear stale state and return a user-friendly error.
/// If it's alive but close to expiry, warn on stderr.
/// Returns None if the session is fine (original error should be used).
async fn check_session_health(
    session_name: Option<&str>,
    _original_err: &anyhow::Error,
) -> Option<anyhow::Error> {
    let paths = SessionStatePaths::default_paths();
    let state = read_state(&paths.state_path);
    let mode = api::mode();

    let session_id = if let Some(name) = session_name {
        state.resolve_candidate(mode, Some(name))?
    } else {
        state.active_session_id.as_deref()?
    };

    let (_, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new().ok()?;
    let session = match client.get_session(&base_url, mode, session_id, &auth).await {
        Ok(s) => s,
        Err(e) if e.is_not_found() => {
            // Session doesn't exist anymore — clean up state
            let _ = crate::config::session_state::with_lock(&paths, true, |state| {
                state.clear_active(mode, session_id);
            });
            let _ = process::kill_daemon(session_id);
            return Some(anyhow::anyhow!(
                "Session expired or not found. Run `steel browser start` to create a new one."
            ));
        }
        Err(_) => return None, // API unreachable, don't mask the original error
    };

    use crate::browser::lifecycle::is_session_live;
    if !is_session_live(&session) {
        // Session exists but is dead — clean up state
        let _ = crate::config::session_state::with_lock(&paths, true, |state| {
            state.clear_active(mode, session_id);
        });
        let _ = process::kill_daemon(session_id);
        let status = session
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Some(anyhow::anyhow!(
            "Session is no longer active (status: {status}). Run `steel browser start` to create a new one."
        ));
    }

    // Session is alive — check if close to expiry and warn
    if let Some(timeout) = session.get("timeout").and_then(|v| v.as_u64())
        && let Some(created) = session.get("createdAt").and_then(|v| v.as_str())
        && let Some(remaining_ms) = estimate_remaining_ms(created, timeout)
        && remaining_ms < 5 * 60 * 1000
    {
        let remaining_secs = remaining_ms / 1000;
        let mins = remaining_secs / 60;
        let secs = remaining_secs % 60;
        eprintln!(
            "Warning: Session expires in {mins}m{secs}s. \
                         Run `steel browser start` to create a new one."
        );
    }

    None
}

/// Estimate remaining session time from a createdAt timestamp (RFC3339) and timeout (ms).
/// Returns None if parsing fails or the session has already expired.
fn estimate_remaining_ms(created_at: &str, timeout_ms: u64) -> Option<u64> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let created = parse_rfc3339_to_epoch_ms(created_at)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    let expires_at = created.checked_add(timeout_ms)?;
    if now >= expires_at {
        return None; // Already expired
    }
    Some(expires_at - now)
}

/// Parse an RFC3339 timestamp to epoch milliseconds.
fn parse_rfc3339_to_epoch_ms(s: &str) -> Option<u64> {
    let ts: jiff::Timestamp = s.trim().parse().ok()?;
    let ms = ts.as_millisecond();
    if ms < 0 {
        return None;
    }
    Some(ms as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_header_args ──────────────────────────────────────────

    #[test]
    fn parse_headers_empty() {
        assert!(parse_header_args(&[]).is_none());
    }

    #[test]
    fn parse_headers_single() {
        let headers = vec!["Content-Type: application/json".to_string()];
        let map = parse_header_args(&headers).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map["Content-Type"], "application/json");
    }

    #[test]
    fn parse_headers_multiple() {
        let headers = vec![
            "X-Custom: value1".to_string(),
            "Authorization: Bearer token".to_string(),
        ];
        let map = parse_header_args(&headers).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["X-Custom"], "value1");
        assert_eq!(map["Authorization"], "Bearer token");
    }

    #[test]
    fn parse_headers_trims_whitespace() {
        let headers = vec!["  Key  :  Value  ".to_string()];
        let map = parse_header_args(&headers).unwrap();
        assert_eq!(map["Key"], "Value");
    }

    #[test]
    fn parse_headers_colon_in_value() {
        let headers = vec!["URL: https://example.com:8080/path".to_string()];
        let map = parse_header_args(&headers).unwrap();
        assert_eq!(map["URL"], "https://example.com:8080/path");
    }

    #[test]
    fn parse_headers_no_colon_skipped() {
        let headers = vec!["invalid-header".to_string(), "Valid: yes".to_string()];
        let map = parse_header_args(&headers).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map["Valid"], "yes");
    }

    #[test]
    fn parse_headers_all_invalid_returns_empty_map() {
        let headers = vec!["no-colon".to_string()];
        let map = parse_header_args(&headers).unwrap();
        assert!(map.is_empty());
    }

    // ── scroll_deltas ──────────────────────────────────────────────

    #[test]
    fn scroll_down_default() {
        let (dx, dy) = scroll_deltas(None, None);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 300.0);
    }

    #[test]
    fn scroll_up() {
        let (dx, dy) = scroll_deltas(Some("up"), Some(500.0));
        assert_eq!(dx, 0.0);
        assert_eq!(dy, -500.0);
    }

    #[test]
    fn scroll_down_explicit() {
        let (dx, dy) = scroll_deltas(Some("down"), Some(200.0));
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 200.0);
    }

    #[test]
    fn scroll_left() {
        let (dx, dy) = scroll_deltas(Some("left"), Some(100.0));
        assert_eq!(dx, -100.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn scroll_right() {
        let (dx, dy) = scroll_deltas(Some("right"), Some(100.0));
        assert_eq!(dx, 100.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn scroll_unknown_direction_defaults_down() {
        let (dx, dy) = scroll_deltas(Some("diagonal"), Some(100.0));
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 100.0);
    }

    #[test]
    fn scroll_default_amount() {
        let (_, dy) = scroll_deltas(Some("down"), None);
        assert_eq!(dy, 300.0);
    }

    // ── resolve_output_path ────────────────────────────────────────

    #[test]
    fn resolve_absolute_path_unchanged() {
        let result = resolve_output_path("/tmp/screenshot.png").unwrap();
        assert_eq!(result, "/tmp/screenshot.png");
    }

    #[test]
    fn resolve_relative_path_prepends_cwd() {
        let result = resolve_output_path("screenshot.png").unwrap();
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(result, cwd.join("screenshot.png").to_string_lossy());
    }

    #[test]
    fn resolve_relative_nested_path() {
        let result = resolve_output_path("output/shot.png").unwrap();
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(result, cwd.join("output/shot.png").to_string_lossy());
    }

    // ── optional_vec ───────────────────────────────────────────────

    #[test]
    fn optional_vec_empty_is_none() {
        assert!(optional_vec(vec![]).is_none());
    }

    #[test]
    fn optional_vec_non_empty_is_some() {
        let v = optional_vec(vec!["color".to_string()]);
        assert_eq!(v.unwrap(), vec!["color"]);
    }

    // ── parse_rfc3339_to_epoch_ms ─────────────────────────────────

    #[test]
    fn rfc3339_utc_basic() {
        // 2025-01-01T00:00:00Z = 1735689600000 ms
        let ms = parse_rfc3339_to_epoch_ms("2025-01-01T00:00:00Z").unwrap();
        assert_eq!(ms, 1735689600000);
    }

    #[test]
    fn rfc3339_with_fractional_seconds() {
        let ms = parse_rfc3339_to_epoch_ms("2025-01-01T00:00:00.500Z").unwrap();
        assert_eq!(ms, 1735689600500);
    }

    #[test]
    fn rfc3339_with_positive_offset() {
        // 2025-01-01T09:00:00+09:00 = 2025-01-01T00:00:00Z
        let ms = parse_rfc3339_to_epoch_ms("2025-01-01T09:00:00+09:00").unwrap();
        assert_eq!(ms, 1735689600000);
    }

    #[test]
    fn rfc3339_with_negative_offset() {
        // 2024-12-31T19:00:00-05:00 = 2025-01-01T00:00:00Z
        let ms = parse_rfc3339_to_epoch_ms("2024-12-31T19:00:00-05:00").unwrap();
        assert_eq!(ms, 1735689600000);
    }

    #[test]
    fn rfc3339_invalid_returns_none() {
        assert!(parse_rfc3339_to_epoch_ms("not a date").is_none());
        assert!(parse_rfc3339_to_epoch_ms("").is_none());
    }

    // ── estimate_remaining_ms ─────────────────────────────────────

    #[test]
    fn estimate_remaining_far_future() {
        // Created in far future with 10 min timeout → should have remaining time
        let remaining = estimate_remaining_ms("2099-01-01T00:00:00Z", 600_000);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > 0);
    }

    #[test]
    fn estimate_remaining_already_expired() {
        // Created in the past with 1ms timeout → expired
        let remaining = estimate_remaining_ms("2020-01-01T00:00:00Z", 1);
        assert!(remaining.is_none());
    }

}
