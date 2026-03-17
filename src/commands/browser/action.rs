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
    #[arg(long)]
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
    #[arg(long)]
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

// ── Command dispatch ────────────────────────────────────────────────

pub async fn run(action: ActionCommand, session: Option<&str>) -> Result<()> {
    let mut client = ensure_daemon(session).await?;

    match action {
        // --- Navigation ---
        ActionCommand::Navigate(args) => {
            let headers = if args.headers.is_empty() {
                None
            } else {
                let mut map = HashMap::new();
                for h in &args.headers {
                    if let Some((k, v)) = h.split_once(':') {
                        map.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                Some(map)
            };
            client
                .send(DaemonCommand::Navigate {
                    url: args.url,
                    wait_until: args.wait_until,
                    headers,
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
            client
                .send(DaemonCommand::Press { key: args.key })
                .await?;
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
            let dir = args.direction.as_deref().unwrap_or("down");
            let amt = args.amount.unwrap_or(300.0);
            let (dx, dy) = match dir {
                "up" => (0.0, -amt),
                "down" => (0.0, amt),
                "left" => (-amt, 0.0),
                "right" => (amt, 0.0),
                _ => (0.0, amt),
            };
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
            let path = std::path::Path::new(&args.output);
            let abs_path = if path.is_absolute() {
                args.output.clone()
            } else {
                std::env::current_dir()?
                    .join(path)
                    .to_string_lossy()
                    .to_string()
            };
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
                let properties = if args.property.is_empty() {
                    None
                } else {
                    Some(args.property)
                };
                let data = client
                    .send(DaemonCommand::Styles {
                        selector: args.selector,
                        properties,
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
                let data = client
                    .send(DaemonCommand::TabNew { url: args.url })
                    .await?;
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
        state
            .active_session_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("No active browser session. Run `steel browser start` first."))?
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
