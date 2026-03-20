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

    // --- Diff ---
    /// Compare snapshots or screenshots
    Diff {
        #[command(subcommand)]
        command: DiffCommand,
    },

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

// ── Diff subcommands ────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DiffCommand {
    /// Compare current snapshot against a baseline
    Snapshot(DiffSnapshotArgs),
    /// Compare current screenshot against a baseline image
    Screenshot(DiffScreenshotArgs),
}

#[derive(Parser)]
pub struct DiffSnapshotArgs {
    /// Baseline snapshot text or file path
    #[arg(short, long)]
    pub baseline: Option<String>,
    /// Restrict snapshot to a subtree
    #[arg(short, long)]
    pub selector: Option<String>,
    /// Use compact output format
    #[arg(short, long)]
    pub compact: bool,
    /// Maximum nesting depth
    #[arg(short = 'd', long, alias = "depth")]
    pub max_depth: Option<usize>,
}

#[derive(Parser)]
pub struct DiffScreenshotArgs {
    /// Baseline image file path (required)
    #[arg(short, long)]
    pub baseline: String,
    /// Color difference threshold (0.0–1.0)
    #[arg(short, long)]
    pub threshold: Option<f64>,
    /// Save diff image to this path
    #[arg(short, long)]
    pub output: Option<String>,
    /// Restrict screenshot to an element
    #[arg(short, long)]
    pub selector: Option<String>,
    /// Capture the full scrollable page
    #[arg(long, alias = "full")]
    pub full_page: bool,
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

// ── Output strategy ─────────────────────────────────────────────────

enum OutputStrategy {
    /// `output::success(data, "")` — silent in text mode, JSON envelope in JSON mode
    Echo,
    /// `output::success_text(data)` — bare string in text mode
    Text,
    /// `output::success_data(data)` — pretty-printed JSON in text mode
    Data,
    /// `output::success_field(data, field)` — extract one field for text mode
    Field(&'static str),
    /// Custom handler for responses needing bespoke formatting
    Custom(Box<dyn FnOnce(serde_json::Value)>),
}

impl OutputStrategy {
    fn display(self, data: serde_json::Value) {
        match self {
            Self::Echo => output::success(data, ""),
            Self::Text => output::success_text(data),
            Self::Data => output::success_data(data),
            Self::Field(f) => output::success_field(data, f),
            Self::Custom(handler) => handler(data),
        }
    }
}

// ── Custom display handlers ─────────────────────────────────────────

fn display_find(data: serde_json::Value) {
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

fn display_tab_list(data: serde_json::Value) {
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

fn display_diff_snapshot(data: serde_json::Value) {
    if output::is_json() {
        output::success_data(data);
    } else {
        let diff_text = data["diff"].as_str().unwrap_or("");
        if diff_text.is_empty() {
            println!("No changes.");
        } else {
            print!("{diff_text}");
        }
    }
}

fn display_diff_screenshot(data: serde_json::Value) {
    if output::is_json() {
        output::success_data(data);
    } else {
        let matched = data["match"].as_bool().unwrap_or(false);
        let pct = data["mismatchPercentage"]
            .as_f64()
            .map(|p| format!("{p:.2}%"))
            .unwrap_or_default();
        if matched {
            println!("Screenshots match.");
        } else {
            println!("Mismatch: {pct}");
            if let Some(p) = data["diffPath"].as_str() {
                println!("Diff image: {p}");
            }
        }
    }
}

// ── ActionCommand → (DaemonCommand, OutputStrategy) ─────────────────

impl ActionCommand {
    fn into_wire(self) -> Result<(DaemonCommand, OutputStrategy)> {
        use DaemonCommand as D;
        use OutputStrategy as O;

        Ok(match self {
            // ── Navigation ──────────────────────────────────────────
            Self::Navigate(a) => (
                D::Navigate {
                    url: a.url,
                    wait_until: a.wait_until,
                    headers: parse_header_args(&a.headers),
                },
                O::Echo,
            ),
            Self::Back => (D::Back, O::Echo),
            Self::Forward => (D::Forward, O::Echo),
            Self::Reload => (D::Reload, O::Echo),

            // ── Interactions ────────────────────────────────────────
            Self::Click(a) => (
                D::Click {
                    selector: a.selector,
                    button: a.button,
                    click_count: a.count,
                    new_tab: a.new_tab,
                },
                O::Echo,
            ),
            Self::DblClick(a) => (
                D::DblClick {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::Fill(a) => (
                D::Fill {
                    selector: a.selector,
                    value: a.value.join(" "),
                },
                O::Echo,
            ),
            Self::Type(a) => (
                D::TypeText {
                    selector: a.selector,
                    text: a.text.join(" "),
                    clear: a.clear,
                    delay_ms: a.delay,
                },
                O::Echo,
            ),
            Self::Press(a) => (D::Press { key: a.key }, O::Echo),
            Self::Hover(a) => (
                D::Hover {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::Focus(a) => (
                D::Focus {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::Check(a) => (
                D::Check {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::Uncheck(a) => (
                D::Uncheck {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::Select(a) => (
                D::Select {
                    selector: a.selector,
                    values: a.values,
                },
                O::Echo,
            ),
            Self::Clear(a) => (
                D::Clear {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::SelectAll(a) => (
                D::SelectAll {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::Scroll(a) => {
                let (dx, dy) = scroll_deltas(a.direction.as_deref(), a.amount);
                (
                    D::Scroll {
                        selector: a.selector,
                        delta_x: dx,
                        delta_y: dy,
                    },
                    O::Echo,
                )
            }
            Self::ScrollIntoView(a) => (
                D::ScrollIntoView {
                    selector: a.selector,
                },
                O::Echo,
            ),
            Self::SetValue(a) => (
                D::SetValue {
                    selector: a.selector,
                    value: a.value.join(" "),
                },
                O::Echo,
            ),

            // ── Observation ─────────────────────────────────────────
            Self::Snapshot(a) => (
                D::Snapshot {
                    interactive_only: a.interactive,
                    selector: a.selector,
                    compact: a.compact,
                    max_depth: a.max_depth,
                    cursor: a.cursor,
                },
                O::Text,
            ),
            Self::Screenshot(a) => {
                let output_fallback = a.output.clone();
                let abs_path = resolve_output_path(&a.output)?;
                (
                    D::Screenshot {
                        full_page: a.full_page,
                        selector: a.selector,
                        format: a.format,
                        quality: a.quality,
                        annotate: a.annotate,
                        path: Some(abs_path),
                        screenshot_dir: None,
                    },
                    O::Custom(Box::new(move |data| {
                        if output::is_json() {
                            output::success_data(data);
                        } else {
                            let saved_path = data["path"].as_str().unwrap_or(&output_fallback);
                            println!("{saved_path}");
                        }
                    })),
                )
            }
            Self::Eval(a) => (D::Eval { script: a.script }, O::Data),
            Self::Find(a) => (
                D::Find {
                    selector: a.selector,
                },
                O::Custom(Box::new(display_find)),
            ),
            Self::Content => (D::Content, O::Text),

            // ── Get info ────────────────────────────────────────────
            Self::Get { command } => match command {
                GetCommand::Text(a) => (
                    D::GetText {
                        selector: a.selector,
                    },
                    O::Field("text"),
                ),
                GetCommand::Html(a) => (
                    D::InnerHtml {
                        selector: a.selector,
                    },
                    O::Field("html"),
                ),
                GetCommand::Value(a) => (
                    D::InputValue {
                        selector: a.selector,
                    },
                    O::Field("value"),
                ),
                GetCommand::Attr(a) => (
                    D::GetAttribute {
                        selector: a.selector,
                        attribute: a.attribute,
                    },
                    O::Field("value"),
                ),
                GetCommand::Url => (D::Url, O::Text),
                GetCommand::Title => (D::Title, O::Text),
                GetCommand::Count(a) => (
                    D::Count {
                        selector: a.selector,
                    },
                    O::Field("count"),
                ),
                GetCommand::Box(a) => (
                    D::BoundingBox {
                        selector: a.selector,
                    },
                    O::Data,
                ),
                GetCommand::Styles(a) => (
                    D::Styles {
                        selector: a.selector,
                        properties: optional_vec(a.property),
                    },
                    O::Data,
                ),
            },

            // ── Check state ─────────────────────────────────────────
            Self::Is { command } => match command {
                IsCommand::Visible(a) => (
                    D::IsVisible {
                        selector: a.selector,
                    },
                    O::Field("visible"),
                ),
                IsCommand::Enabled(a) => (
                    D::IsEnabled {
                        selector: a.selector,
                    },
                    O::Field("enabled"),
                ),
                IsCommand::Checked(a) => (
                    D::IsChecked {
                        selector: a.selector,
                    },
                    O::Field("checked"),
                ),
            },

            // ── Wait ────────────────────────────────────────────────
            Self::Wait(a) => (
                D::Wait {
                    timeout: Some(a.timeout),
                    text: a.text,
                    selector: a.selector,
                    state: a.state,
                    url: a.url,
                    function: a.function,
                    load_state: a.load_state,
                },
                O::Field("waited"),
            ),

            // ── Tabs ────────────────────────────────────────────────
            Self::Tab { command } => match command {
                TabCommand::List => (D::TabList, O::Custom(Box::new(display_tab_list))),
                TabCommand::New(a) => (D::TabNew { url: a.url }, O::Field("url")),
                TabCommand::Switch(a) => (D::TabSwitch { index: a.index }, O::Field("url")),
                TabCommand::Close(a) => (D::TabClose { index: a.index }, O::Echo),
            },

            // ── Cookies ─────────────────────────────────────────────
            Self::Cookies { command } => match command {
                None => (D::CookiesGet { urls: None }, O::Data),
                Some(CookiesCommand::Set(a)) => (
                    D::CookiesSet {
                        name: a.name,
                        value: a.value,
                        domain: a.domain,
                        path: a.path,
                        secure: a.secure,
                        http_only: a.http_only,
                    },
                    O::Echo,
                ),
                Some(CookiesCommand::Clear) => (D::CookiesClear, O::Echo),
            },

            // ── Storage ─────────────────────────────────────────────
            Self::Storage { command } => {
                let (st, sub) = match command {
                    StorageTypeCommand::Local(sub) => ("local", sub),
                    StorageTypeCommand::Session(sub) => ("session", sub),
                };
                match sub.command {
                    None => (
                        D::StorageGet {
                            storage_type: st.to_string(),
                            key: sub.key,
                        },
                        O::Data,
                    ),
                    Some(StorageActionCommand::Set(a)) => (
                        D::StorageSet {
                            storage_type: st.to_string(),
                            key: a.key,
                            value: a.value,
                        },
                        O::Echo,
                    ),
                    Some(StorageActionCommand::Clear) => (
                        D::StorageClear {
                            storage_type: st.to_string(),
                        },
                        O::Echo,
                    ),
                }
            }

            // ── Drag & drop ─────────────────────────────────────────
            Self::Drag(a) => (
                D::Drag {
                    source: a.source,
                    target: a.target,
                },
                O::Echo,
            ),

            // ── File upload ─────────────────────────────────────────
            Self::Upload(a) => (
                D::Upload {
                    selector: a.selector,
                    files: a.files,
                },
                O::Echo,
            ),

            // ── Visual ──────────────────────────────────────────────
            Self::Highlight(a) => (
                D::Highlight {
                    selector: a.selector,
                },
                O::Echo,
            ),

            // ── Browser settings ────────────────────────────────────
            Self::Set { command } => match command {
                SetCommand::Viewport(a) => (
                    D::SetViewport {
                        width: a.width,
                        height: a.height,
                        device_scale_factor: a.scale,
                        mobile: if a.mobile { Some(true) } else { None },
                    },
                    O::Echo,
                ),
                SetCommand::Geo(a) => (
                    D::SetGeolocation {
                        latitude: a.latitude,
                        longitude: a.longitude,
                        accuracy: a.accuracy,
                    },
                    O::Echo,
                ),
                SetCommand::Offline(a) => {
                    let offline = matches!(a.state.as_str(), "on" | "true" | "1");
                    (D::SetOffline { offline }, O::Echo)
                }
                SetCommand::Headers(a) => {
                    let headers: HashMap<String, String> = serde_json::from_str(&a.json)
                        .map_err(|e| anyhow::anyhow!("Invalid JSON for headers: {e}"))?;
                    (D::SetHeaders { headers }, O::Echo)
                }
                SetCommand::UserAgent(a) => (
                    D::SetUserAgent {
                        user_agent: a.user_agent.join(" "),
                    },
                    O::Echo,
                ),
            },

            // ── Window ──────────────────────────────────────────────
            Self::BringToFront => (D::BringToFront, O::Echo),

            // ── Diff ────────────────────────────────────────────────
            Self::Diff { command } => match command {
                DiffCommand::Snapshot(a) => (
                    D::DiffSnapshot {
                        baseline: a.baseline,
                        selector: a.selector,
                        compact: a.compact,
                        max_depth: a.max_depth,
                    },
                    O::Custom(Box::new(display_diff_snapshot)),
                ),
                DiffCommand::Screenshot(a) => {
                    let output_path = a
                        .output
                        .as_ref()
                        .map(|p| resolve_output_path(p))
                        .transpose()?;
                    (
                        D::DiffScreenshot {
                            baseline: resolve_output_path(&a.baseline)?,
                            threshold: a.threshold,
                            selector: a.selector,
                            full_page: a.full_page,
                            output: output_path,
                        },
                        O::Custom(Box::new(display_diff_screenshot)),
                    )
                }
            },

            // ── Session ─────────────────────────────────────────────
            Self::Close => (D::Close, O::Echo),
        })
    }
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
    let (cmd, strategy) = action.into_wire()?;
    let data = client.send(cmd).await?;
    strategy.display(data);
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
