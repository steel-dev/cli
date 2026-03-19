//! Browser action handler: routes Clap subcommands to a persistent daemon
//! that holds the BrowserEngine (CDP connection + RefMap) alive between calls.

use std::collections::HashMap;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::protocol::DaemonCommand;
use crate::util::output;

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

    // --- Cookies ---
    /// Manage browser cookies
    Cookies {
        #[command(subcommand)]
        command: Option<CookiesCommand>,
    },

    // --- Storage ---
    /// Manage browser storage (localStorage/sessionStorage)
    Storage {
        #[command(subcommand)]
        command: StorageTypeCommand,
    },

    // --- Drag & drop ---
    /// Drag and drop from one element to another
    Drag(DragArgs),

    // --- File upload ---
    /// Upload files to a file input element
    Upload(UploadArgs),

    // --- Visual ---
    /// Visually highlight an element
    Highlight(SelectorArg),

    // --- Browser settings ---
    /// Configure browser settings (viewport, geolocation, etc.)
    Set {
        #[command(subcommand)]
        command: SetCommand,
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

// ── Cookies subcommands ──────────────────────────────────────────────

#[derive(Subcommand)]
pub enum CookiesCommand {
    /// Set a cookie
    Set(CookiesSetArgs),
    /// Clear all cookies
    Clear,
}

// ── Storage subcommands ─────────────────────────────────────────────

#[derive(Subcommand)]
pub enum StorageTypeCommand {
    /// Manage localStorage
    Local(StorageSubArgs),
    /// Manage sessionStorage
    Session(StorageSubArgs),
}

#[derive(Parser)]
#[command(args_conflicts_with_subcommands = true)]
pub struct StorageSubArgs {
    /// Key to get (returns all if omitted)
    pub key: Option<String>,

    #[command(subcommand)]
    pub command: Option<StorageActionCommand>,
}

#[derive(Subcommand)]
pub enum StorageActionCommand {
    /// Set a storage value
    Set(StorageSetArgs),
    /// Clear all values
    Clear,
}

// ── Set subcommands ─────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum SetCommand {
    /// Set viewport size
    Viewport(SetViewportArgs),
    /// Set geolocation
    #[command(aliases = ["geolocation"])]
    Geo(SetGeoArgs),
    /// Toggle offline mode
    Offline(SetOfflineArgs),
    /// Set extra HTTP headers (JSON string)
    Headers(SetHeadersArgs),
    /// Set browser user agent string
    #[command(name = "useragent", aliases = ["ua"])]
    UserAgent(SetUserAgentArgs),
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
    pub script: String,
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

// ── New command arg structs ──────────────────────────────────────────

#[derive(Parser)]
pub struct CookiesSetArgs {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Cookie domain
    #[arg(long)]
    pub domain: Option<String>,
    /// Cookie path
    #[arg(long)]
    pub path: Option<String>,
    /// Secure flag
    #[arg(long)]
    pub secure: bool,
    /// HttpOnly flag
    #[arg(long)]
    pub http_only: bool,
}

#[derive(Parser)]
pub struct StorageSetArgs {
    /// Key to set
    pub key: String,
    /// Value to set
    pub value: String,
}

#[derive(Parser)]
pub struct DragArgs {
    /// Source element selector or ref
    pub source: String,
    /// Target element selector or ref
    pub target: String,
}

#[derive(Parser)]
pub struct UploadArgs {
    /// File input element selector or ref
    pub selector: String,
    /// File paths to upload
    #[arg(num_args = 1..)]
    pub files: Vec<String>,
}

#[derive(Parser)]
pub struct SetViewportArgs {
    /// Viewport width in pixels
    pub width: u32,
    /// Viewport height in pixels
    pub height: u32,
    /// Device scale factor
    #[arg(long)]
    pub scale: Option<f64>,
    /// Emulate mobile device
    #[arg(long)]
    pub mobile: bool,
}

#[derive(Parser)]
pub struct SetGeoArgs {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Accuracy in meters
    #[arg(long)]
    pub accuracy: Option<f64>,
}

#[derive(Parser)]
pub struct SetOfflineArgs {
    /// "on" to enable offline mode, "off" to disable
    pub state: String,
}

#[derive(Parser)]
pub struct SetHeadersArgs {
    /// JSON string of headers (e.g. '{"X-Key":"value"}')
    pub json: String,
}

#[derive(Parser)]
pub struct SetUserAgentArgs {
    /// User agent string
    #[arg(trailing_var_arg = true, num_args = 1..)]
    pub user_agent: Vec<String>,
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
            let data = client
                .send(DaemonCommand::Navigate {
                    url: args.url,
                    wait_until: args.wait_until,
                    headers: parse_header_args(&args.headers),
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Back => {
            let data = client.send(DaemonCommand::Back).await?;
            output::success(data, "");
        }
        ActionCommand::Forward => {
            let data = client.send(DaemonCommand::Forward).await?;
            output::success(data, "");
        }
        ActionCommand::Reload => {
            let data = client.send(DaemonCommand::Reload).await?;
            output::success(data, "");
        }

        // --- Interactions ---
        ActionCommand::Click(args) => {
            let data = client
                .send(DaemonCommand::Click {
                    selector: args.selector,
                    button: args.button,
                    click_count: args.count,
                    new_tab: args.new_tab,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::DblClick(args) => {
            let data = client
                .send(DaemonCommand::DblClick {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Fill(args) => {
            let value = args.value.join(" ");
            let data = client
                .send(DaemonCommand::Fill {
                    selector: args.selector,
                    value,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Type(args) => {
            let text = args.text.join(" ");
            let data = client
                .send(DaemonCommand::TypeText {
                    selector: args.selector,
                    text,
                    clear: args.clear,
                    delay_ms: args.delay,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Press(args) => {
            let data = client.send(DaemonCommand::Press { key: args.key }).await?;
            output::success(data, "");
        }
        ActionCommand::Hover(args) => {
            let data = client
                .send(DaemonCommand::Hover {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Focus(args) => {
            let data = client
                .send(DaemonCommand::Focus {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Check(args) => {
            let data = client
                .send(DaemonCommand::Check {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Uncheck(args) => {
            let data = client
                .send(DaemonCommand::Uncheck {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Select(args) => {
            let data = client
                .send(DaemonCommand::Select {
                    selector: args.selector,
                    values: args.values,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Clear(args) => {
            let data = client
                .send(DaemonCommand::Clear {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::SelectAll(args) => {
            let data = client
                .send(DaemonCommand::SelectAll {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::Scroll(args) => {
            let (dx, dy) = scroll_deltas(args.direction.as_deref(), args.amount);
            let data = client
                .send(DaemonCommand::Scroll {
                    selector: args.selector,
                    delta_x: dx,
                    delta_y: dy,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::ScrollIntoView(args) => {
            let data = client
                .send(DaemonCommand::ScrollIntoView {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }
        ActionCommand::SetValue(args) => {
            let value = args.value.join(" ");
            let data = client
                .send(DaemonCommand::SetValue {
                    selector: args.selector,
                    value,
                })
                .await?;
            output::success(data, "");
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
            let data = client
                .send(DaemonCommand::Eval {
                    script: args.script,
                })
                .await?;
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
                let data = client
                    .send(DaemonCommand::TabClose { index: args.index })
                    .await?;
                output::success(data, "");
            }
        },

        // --- Cookies ---
        ActionCommand::Cookies { command } => match command {
            None => {
                let data = client
                    .send(DaemonCommand::CookiesGet { urls: None })
                    .await?;
                output::success_data(data);
            }
            Some(CookiesCommand::Set(args)) => {
                let data = client
                    .send(DaemonCommand::CookiesSet {
                        name: args.name,
                        value: args.value,
                        domain: args.domain,
                        path: args.path,
                        secure: args.secure,
                        http_only: args.http_only,
                    })
                    .await?;
                output::success(data, "");
            }
            Some(CookiesCommand::Clear) => {
                let data = client.send(DaemonCommand::CookiesClear).await?;
                output::success(data, "");
            }
        },

        // --- Storage ---
        ActionCommand::Storage { command } => {
            let (storage_type, sub) = match command {
                StorageTypeCommand::Local(sub) => ("local", sub),
                StorageTypeCommand::Session(sub) => ("session", sub),
            };
            match sub.command {
                None => {
                    // `storage local` or `storage local <key>`
                    let data = client
                        .send(DaemonCommand::StorageGet {
                            storage_type: storage_type.to_string(),
                            key: sub.key,
                        })
                        .await?;
                    output::success_data(data);
                }
                Some(StorageActionCommand::Set(args)) => {
                    let data = client
                        .send(DaemonCommand::StorageSet {
                            storage_type: storage_type.to_string(),
                            key: args.key,
                            value: args.value,
                        })
                        .await?;
                    output::success(data, "");
                }
                Some(StorageActionCommand::Clear) => {
                    let data = client
                        .send(DaemonCommand::StorageClear {
                            storage_type: storage_type.to_string(),
                        })
                        .await?;
                    output::success(data, "");
                }
            }
        }

        // --- Drag & drop ---
        ActionCommand::Drag(args) => {
            let data = client
                .send(DaemonCommand::Drag {
                    source: args.source,
                    target: args.target,
                })
                .await?;
            output::success(data, "");
        }

        // --- Upload ---
        ActionCommand::Upload(args) => {
            let data = client
                .send(DaemonCommand::Upload {
                    selector: args.selector,
                    files: args.files,
                })
                .await?;
            output::success(data, "");
        }

        // --- Highlight ---
        ActionCommand::Highlight(args) => {
            let data = client
                .send(DaemonCommand::Highlight {
                    selector: args.selector,
                })
                .await?;
            output::success(data, "");
        }

        // --- Browser settings ---
        ActionCommand::Set { command } => match command {
            SetCommand::Viewport(args) => {
                let data = client
                    .send(DaemonCommand::SetViewport {
                        width: args.width,
                        height: args.height,
                        device_scale_factor: args.scale,
                        mobile: if args.mobile { Some(true) } else { None },
                    })
                    .await?;
                output::success(data, "");
            }
            SetCommand::Geo(args) => {
                let data = client
                    .send(DaemonCommand::SetGeolocation {
                        latitude: args.latitude,
                        longitude: args.longitude,
                        accuracy: args.accuracy,
                    })
                    .await?;
                output::success(data, "");
            }
            SetCommand::Offline(args) => {
                let offline = matches!(args.state.as_str(), "on" | "true" | "1");
                let data = client.send(DaemonCommand::SetOffline { offline }).await?;
                output::success(data, "");
            }
            SetCommand::Headers(args) => {
                let headers: HashMap<String, String> = serde_json::from_str(&args.json)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON for headers: {e}"))?;
                let data = client.send(DaemonCommand::SetHeaders { headers }).await?;
                output::success(data, "");
            }
            SetCommand::UserAgent(args) => {
                let user_agent = args.user_agent.join(" ");
                let data = client
                    .send(DaemonCommand::SetUserAgent { user_agent })
                    .await?;
                output::success(data, "");
            }
        },

        // --- Window ---
        ActionCommand::BringToFront => {
            let data = client.send(DaemonCommand::BringToFront).await?;
            output::success(data, "");
        }

        // --- Session ---
        ActionCommand::Close => {
            let data = client.send(DaemonCommand::Close).await?;
            output::success(data, "");
        }
    }

    Ok(())
}

async fn ensure_daemon(session_name: Option<&str>) -> Result<DaemonClient> {
    let name = session_name.unwrap_or("default");
    // connect() already cleans up stale sockets via cleanup_if_dead()
    DaemonClient::connect(name).await.map_err(|_| {
        if name == "default" {
            anyhow::anyhow!(
                "No active browser session. Start one with: steel browser start"
            )
        } else {
            anyhow::anyhow!(
                "No running session \"{name}\". Start one with: steel browser start --session {name}"
            )
        }
    })
}

/// On action failure, check if daemon is still reachable. If not, suggest restarting.
async fn check_session_health(
    session_name: Option<&str>,
    _original_err: &anyhow::Error,
) -> Option<anyhow::Error> {
    let name = session_name.unwrap_or("default");
    if let Ok(mut client) = DaemonClient::connect(name).await
        && client.send(DaemonCommand::Ping).await.is_ok()
    {
        return None; // Daemon is fine, original error stands
    }
    Some(anyhow::anyhow!(
        "Session \"{name}\" is no longer reachable. Run `steel browser start` to create a new one."
    ))
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
}
