use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use base64::Engine as _;
use clap::{Parser, Subcommand};
use serde_json::{Map, Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::api::commands::{Reply, Session};
use crate::api::session::Session as RemoteSession;
use crate::browser::state::{self, BrowserSession, SnapshotInfo};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::util::api;
use crate::util::output::{self, SilentExit};

use super::{DEFAULT_SESSION, start};

#[derive(Parser)]
pub struct SelectorArg {
    /// Element ref (e.g. e5) or CSS selector
    pub selector: String,
}

#[derive(Parser)]
pub struct OptionalSelectorArg {
    /// Element ref (e.g. e5) or CSS selector; the whole page if omitted
    pub selector: Option<String>,
}

#[derive(Parser)]
pub struct RemovedArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum ActionCommand {
    /// Navigate to a URL
    #[command(aliases = ["open", "goto"])]
    Navigate(NavigateArgs),
    /// Navigate back in history
    Back,
    /// Navigate forward in history
    Forward,
    /// Reload the current page
    Reload,

    /// Click an element
    Click(ClickArgs),
    /// Double-click an element
    #[command(name = "dblclick")]
    DblClick(SelectorArg),
    /// Fill an input field (replaces the current value)
    Fill(FillArgs),
    /// Type text into an element key by key (appends to the current value)
    #[command(name = "type")]
    Type(TypeArgs),
    /// Press a key or chord (Enter, Escape, Tab, Control+a)
    #[command(aliases = ["key"])]
    Press(PressArgs),
    /// Hold a key down until keyup
    #[command(name = "keydown")]
    KeyDown(KeyArgs),
    /// Release a key held with keydown
    #[command(name = "keyup")]
    KeyUp(KeyArgs),
    /// Hover over an element
    Hover(SelectorArg),
    /// Focus an element
    Focus(SelectorArg),
    /// Check a checkbox, radio button, or switch
    Check(SelectorArg),
    /// Uncheck a checkbox or switch
    Uncheck(SelectorArg),
    /// Select option(s) in a <select> element
    Select(SelectArgs),
    /// Clear an input field
    Clear(SelectorArg),
    /// Select all text in an input
    #[command(name = "selectall")]
    SelectAll(SelectorArg),
    /// Scroll the page or an element
    Scroll(ScrollArgs),
    /// Drag an element onto another
    Drag(DragArgs),
    /// Upload files to a file input
    Upload(UploadArgs),
    /// Mouse input at a point, for canvases and maps
    Mouse(MouseArgs),

    /// Take an accessibility tree snapshot with element refs
    Snapshot(SnapshotArgs),
    /// Take a screenshot
    Screenshot(ScreenshotArgs),
    /// Save the page as a PDF (headless sessions only)
    Pdf(PdfArgs),
    /// Run JavaScript in the page and print its value
    Eval(EvalArgs),
    /// Find elements and print their refs
    Find(FindArgs),
    /// Get the page HTML
    Content,
    /// Get information about elements or the page
    Get {
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Check element state
    Is {
        #[command(subcommand)]
        command: IsCommand,
    },
    /// Wait for an element, text, URL, load state, download, or a fixed time
    Wait(WaitArgs),
    /// List, open, switch, or close tabs
    Tab {
        #[command(subcommand)]
        command: TabCommand,
    },
    /// Accept or dismiss an open dialog
    Dialog(DialogArgs),
    /// Wait for a command that is still running
    Status(ExecutionArgs),
    /// Stop a command that is still running
    Cancel(ExecutionArgs),
    /// Close the browser session
    #[command(aliases = ["quit", "exit"])]
    Close,

    #[command(hide = true)]
    Cookies(RemovedArgs),
    #[command(hide = true)]
    Storage(RemovedArgs),
    #[command(hide = true)]
    Set(RemovedArgs),
    #[command(hide = true)]
    Highlight(RemovedArgs),
    #[command(name = "bringtofront", hide = true)]
    BringToFront(RemovedArgs),
    #[command(hide = true)]
    Diff(RemovedArgs),
    #[command(name = "scrollintoview", hide = true)]
    ScrollIntoView(RemovedArgs),
    #[command(name = "setvalue", hide = true)]
    SetValue(RemovedArgs),
}

impl ActionCommand {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::Navigate(_) => "navigate",
            Self::Back => "back",
            Self::Forward => "forward",
            Self::Reload => "reload",
            Self::Click(_) => "click",
            Self::DblClick(_) => "dblclick",
            Self::Fill(_) => "fill",
            Self::Type(_) => "type",
            Self::Press(_) => "press",
            Self::KeyDown(_) => "keydown",
            Self::KeyUp(_) => "keyup",
            Self::Hover(_) => "hover",
            Self::Focus(_) => "focus",
            Self::Check(_) => "check",
            Self::Uncheck(_) => "uncheck",
            Self::Select(_) => "select",
            Self::Clear(_) => "clear",
            Self::SelectAll(_) => "selectall",
            Self::Scroll(_) => "scroll",
            Self::Drag(_) => "drag",
            Self::Upload(_) => "upload",
            Self::Mouse(_) => "mouse",
            Self::Snapshot(_) => "snapshot",
            Self::Screenshot(_) => "screenshot",
            Self::Pdf(_) => "pdf",
            Self::Eval(_) => "eval",
            Self::Find(_) => "find",
            Self::Content => "content",
            Self::Get { .. } => "get",
            Self::Is { .. } => "is",
            Self::Wait(_) => "wait",
            Self::Tab { .. } => "tab",
            Self::Dialog(_) => "dialog",
            Self::Status(_) => "status",
            Self::Cancel(_) => "cancel",
            Self::Close => "close",
            Self::Cookies(_) => "cookies",
            Self::Storage(_) => "storage",
            Self::Set(_) => "set",
            Self::Highlight(_) => "highlight",
            Self::BringToFront(_) => "bringtofront",
            Self::Diff(_) => "diff",
            Self::ScrollIntoView(_) => "scrollintoview",
            Self::SetValue(_) => "setvalue",
        }
    }
}

#[derive(Subcommand)]
pub enum GetCommand {
    /// Get the visible text of an element
    Text(SelectorArg),
    /// Get the inner HTML of an element
    Html(SelectorArg),
    /// Get the main content of an element or the page as Markdown
    Markdown(OptionalSelectorArg),
    /// Get an input or textarea value
    Value(SelectorArg),
    /// Get an element attribute
    Attr(GetAttrArgs),
    /// Get the page URL
    Url,
    /// Get the page title
    Title,
    /// Count matching elements
    Count(SelectorArg),
    /// Get an element's bounding box
    #[command(name = "box")]
    Box(SelectorArg),
    #[command(hide = true)]
    Styles(RemovedArgs),
}

#[derive(Subcommand)]
pub enum IsCommand {
    /// Check if an element is visible
    Visible(SelectorArg),
    /// Check if an element is enabled
    Enabled(SelectorArg),
    /// Check if an element is checked
    Checked(SelectorArg),
}

#[derive(Subcommand)]
pub enum TabCommand {
    /// List open tabs
    List,
    /// Open a new tab and switch to it
    New(TabNewArgs),
    /// Switch to a tab
    Switch(TabSwitchArgs),
    /// Close a tab (the current tab if omitted)
    Close(TabCloseArgs),
}

#[derive(Parser)]
pub struct NavigateArgs {
    /// URL to navigate to
    pub url: String,
    /// Wait condition: load, domcontentloaded, networkidle
    #[arg(long, value_parser = ["load", "domcontentloaded", "networkidle"])]
    pub wait_until: Option<String>,
    #[arg(long = "header", alias = "headers", hide = true)]
    pub headers: Vec<String>,
}

#[derive(Parser)]
pub struct ClickArgs {
    /// Element ref (e.g. e5) or CSS selector
    pub selector: String,
    /// Number of clicks (2 for double-click)
    #[arg(long)]
    pub count: Option<u8>,
    #[arg(long, hide = true)]
    pub button: Option<String>,
    #[arg(long, hide = true)]
    pub new_tab: bool,
}

#[derive(Parser)]
pub struct FillArgs {
    /// Element ref or CSS selector
    pub selector: String,
    /// Value to fill
    #[arg(num_args = 1..)]
    pub value: Vec<String>,
}

#[derive(Parser)]
pub struct TypeArgs {
    /// Element ref or CSS selector
    pub selector: String,
    /// Text to type
    #[arg(num_args = 1..)]
    pub text: Vec<String>,
    /// Clear the field before typing
    #[arg(long)]
    pub clear: bool,
    /// Delay between keystrokes in milliseconds (at most 1000)
    #[arg(long)]
    pub delay: Option<u64>,
}

#[derive(Parser)]
pub struct PressArgs {
    /// Key to press (e.g. Enter, Escape, Tab, Control+a)
    pub key: String,
}

#[derive(Parser)]
pub struct KeyArgs {
    /// Key name (e.g. Shift, Control, ArrowDown)
    pub key: String,
}

#[derive(Parser)]
pub struct SelectArgs {
    /// Select element ref or CSS selector
    pub selector: String,
    /// Option value(s) or label(s) to select
    #[arg(num_args = 1..)]
    pub values: Vec<String>,
}

#[derive(Parser)]
pub struct ScrollArgs {
    /// Direction: up, down, left, right (default: down)
    #[arg(value_parser = ["up", "down", "left", "right"])]
    pub direction: Option<String>,
    /// Scroll amount in pixels (default: 300)
    pub amount: Option<f64>,
    /// Element to scroll instead of the page
    #[arg(long, short)]
    pub selector: Option<String>,
}

#[derive(Parser)]
pub struct DragArgs {
    /// Element to drag (ref or CSS selector)
    pub source: String,
    /// Element to drop on (ref or CSS selector)
    pub target: String,
}

#[derive(Parser)]
pub struct UploadArgs {
    /// File input ref or CSS selector
    pub selector: String,
    /// Local files to upload, or names of files already in the session
    #[arg(num_args = 1..)]
    pub files: Vec<String>,
}

#[derive(Parser)]
pub struct MouseArgs {
    /// move, down, up, click, or wheel
    #[arg(value_parser = ["move", "down", "up", "click", "wheel"])]
    pub action: String,
    /// X in CSS pixels of the visible page (needed for move, click, and wheel)
    pub x: Option<f64>,
    /// Y in CSS pixels of the visible page (needed for move, click, and wheel)
    pub y: Option<f64>,
    /// Mouse button: left, right, middle
    #[arg(long, value_parser = ["left", "right", "middle"])]
    pub button: Option<String>,
    /// Click count for click (1 to 3)
    #[arg(long)]
    pub clicks: Option<u8>,
    /// Horizontal wheel delta
    #[arg(long, allow_hyphen_values = true)]
    pub dx: Option<f64>,
    /// Vertical wheel delta
    #[arg(long, allow_hyphen_values = true)]
    pub dy: Option<f64>,
}

#[derive(Parser)]
pub struct SnapshotArgs {
    /// Show only interactive elements
    #[arg(short, long)]
    pub interactive: bool,
    /// Keep only lines with refs or values, and their ancestors
    #[arg(short, long)]
    pub compact: bool,
    /// Maximum nesting depth
    #[arg(short = 'd', long, alias = "depth")]
    pub max_depth: Option<u64>,
    #[arg(short, long, hide = true)]
    pub selector: Option<String>,
    #[arg(short = 'u', long, hide = true)]
    pub urls: bool,
}

#[derive(Parser)]
pub struct ScreenshotArgs {
    /// Capture the full scrollable page
    #[arg(short = 'f', long, alias = "full")]
    pub full_page: bool,
    /// Output file path
    #[arg(short, long, default_value = "screenshot.png")]
    pub output: String,
    /// Capture one element (ref or CSS selector)
    #[arg(long)]
    pub selector: Option<String>,
    /// Image format: png, jpeg, webp (default: from the output extension)
    #[arg(long, alias = "type", alias = "screenshot-format", value_parser = ["png", "jpeg", "webp"])]
    pub format: Option<String>,
    /// JPEG/WebP quality (0-100)
    #[arg(long, alias = "screenshot-quality", value_parser = clap::value_parser!(u8).range(0..=100))]
    pub quality: Option<u8>,
    #[arg(long, hide = true)]
    pub annotate: bool,
}

#[derive(Parser)]
pub struct PdfArgs {
    /// Output file path
    #[arg(short, long, default_value = "page.pdf")]
    pub output: String,
    /// Use landscape orientation
    #[arg(long)]
    pub landscape: bool,
    /// Print background graphics
    #[arg(long)]
    pub print_background: bool,
}

#[derive(Parser)]
pub struct EvalArgs {
    /// JavaScript expression or function body; its value is printed
    pub script: String,
}

#[derive(Parser)]
pub struct FindArgs {
    /// What to look for (a CSS selector unless --by says otherwise)
    pub value: String,
    /// Locator kind
    #[arg(long, default_value = "css", value_parser = ["css", "role", "text", "label", "placeholder", "alt", "title", "testid"])]
    pub by: String,
    /// Accessible name, with --by role
    #[arg(long)]
    pub name: Option<String>,
    /// Match the text exactly instead of as a case-insensitive substring
    #[arg(long)]
    pub exact: bool,
}

#[derive(Parser)]
pub struct GetAttrArgs {
    /// Element ref or CSS selector
    pub selector: String,
    /// Attribute name
    pub attribute: String,
}

#[derive(Parser)]
pub struct WaitArgs {
    /// Fixed time to wait in milliseconds (at most 30000)
    pub ms: Option<u64>,
    /// Timeout in milliseconds (at most 50000)
    #[arg(long, default_value_t = 30000)]
    pub timeout: u64,
    /// Wait for text to appear on the page
    #[arg(short = 't', long)]
    pub text: Option<String>,
    /// Wait for an element (ref or CSS selector)
    #[arg(long)]
    pub selector: Option<String>,
    /// Element state: visible, hidden, attached, detached
    #[arg(long, value_parser = ["visible", "hidden", "attached", "detached"])]
    pub state: Option<String>,
    /// Wait for the URL to contain this string (`*` matches anything)
    #[arg(short = 'u', long)]
    pub url: Option<String>,
    /// Wait for a load state: load, domcontentloaded, networkidle
    #[arg(short = 'l', long, alias = "load", value_parser = ["load", "domcontentloaded", "networkidle"])]
    pub load_state: Option<String>,
    /// Wait for a download to finish and save it
    #[arg(long)]
    pub download: bool,
    /// Where to save the download (default: its file name in the current directory)
    #[arg(short, long, requires = "download")]
    pub output: Option<String>,
    #[arg(short = 'f', long, alias = "fn", hide = true)]
    pub function: Option<String>,
}

#[derive(Parser)]
pub struct TabNewArgs {
    /// URL to open (defaults to about:blank)
    pub url: Option<String>,
}

#[derive(Parser)]
pub struct TabSwitchArgs {
    /// Tab id from `tab list`, such as t2
    #[arg(value_parser = parse_tab)]
    pub tab: String,
}

#[derive(Parser)]
pub struct TabCloseArgs {
    /// Tab id from `tab list`, such as t2
    #[arg(value_parser = parse_tab)]
    pub tab: Option<String>,
}

#[derive(Parser)]
pub struct DialogArgs {
    /// accept or dismiss
    #[arg(value_parser = ["accept", "dismiss"])]
    pub action: String,
    /// Dialog id such as d-2 (default: the last dialog the CLI saw)
    pub id: Option<String>,
    /// Answer for a prompt dialog
    #[arg(long)]
    pub text: Option<String>,
}

#[derive(Parser)]
pub struct ExecutionArgs {
    /// Execution id such as x-7
    pub execution: String,
}

fn parse_tab(value: &str) -> Result<String, String> {
    let digits = value.strip_prefix('t').unwrap_or("");
    if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
        Ok(value.to_string())
    } else {
        Err("tab ids look like t2. Run `steel browser tab list` to see them".to_string())
    }
}

pub enum Output {
    None,
    Snapshot,
    Value,
    Find,
    Tabs,
    TabOpened,
    Switch(String),
    Saved(PathBuf),
    Download(Option<PathBuf>),
    Execution,
    Dialog,
}

pub struct Step {
    pub commands: Vec<Value>,
    pub output: Output,
    pub uploads: Vec<(PathBuf, String)>,
}

impl Step {
    const fn new(commands: Vec<Value>, output: Output) -> Self {
        Self {
            commands,
            output,
            uploads: Vec::new(),
        }
    }

    pub fn batchable(&self) -> bool {
        !matches!(self.output, Output::Switch(_) | Output::TabOpened)
            && !self
                .commands
                .iter()
                .any(|c| c["command"] == "tab" && c["action"] == "close")
    }
}

pub enum Plan {
    Run(Step),
    Control(Value, Output),
    Close,
}

struct Cmd(Map<String, Value>);

fn cmd(name: &str) -> Cmd {
    let mut map = Map::new();
    map.insert("command".into(), json!(name));
    Cmd(map)
}

impl Cmd {
    fn set(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }

    fn opt(self, key: &str, value: Option<impl Into<Value>>) -> Self {
        match value {
            Some(value) => self.set(key, value),
            None => self,
        }
    }

    fn on(self, key: &str, enabled: bool) -> Self {
        if enabled { self.set(key, true) } else { self }
    }

    fn target(self, selector: String) -> Self {
        self.set("target", selector)
    }

    fn done(self) -> Value {
        Value::Object(self.0)
    }
}

fn single(command: Cmd, output: Output) -> Result<Plan> {
    Ok(Plan::Run(Step::new(vec![command.done()], output)))
}

fn removed(what: &str, instead: &str) -> anyhow::Error {
    anyhow!("`steel browser {what}` was removed in 0.5. {instead}")
}

fn absolute(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    })
}

fn image_format(output: &str) -> Option<&'static str> {
    let extension = Path::new(output)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some("jpeg"),
        "webp" => Some("webp"),
        "png" => Some("png"),
        _ => None,
    }
}

impl ActionCommand {
    pub fn plan(self) -> Result<Plan> {
        use Output as O;
        match self {
            Self::Navigate(a) => {
                if !a.headers.is_empty() {
                    return Err(removed(
                        "navigate --header",
                        "Per-navigation request headers are not supported.",
                    ));
                }
                let mut commands = vec![cmd("navigate").set("url", a.url).done()];
                if a.wait_until.as_deref() == Some("networkidle") {
                    commands.push(cmd("wait").set("load", "networkidle").done());
                }
                Ok(Plan::Run(Step::new(commands, O::None)))
            }
            Self::Back => single(cmd("back"), O::None),
            Self::Forward => single(cmd("forward"), O::None),
            Self::Reload => single(cmd("reload"), O::None),
            Self::Click(a) => {
                if a.button.is_some() {
                    return Err(removed(
                        "click --button",
                        "Use `steel browser get box <selector>`, then `steel browser mouse click <x> <y> --button right`.",
                    ));
                }
                if a.new_tab {
                    return Err(removed(
                        "click --new-tab",
                        "Use `steel browser get attr <selector> href`, then `steel browser tab new <url>`.",
                    ));
                }
                let name = match a.count {
                    None | Some(1) => "click",
                    Some(2) => "dblclick",
                    Some(n) => bail!(
                        "--count {n} is not supported. Use 1 or 2, or `steel browser mouse click <x> <y> --clicks 3`."
                    ),
                };
                single(cmd(name).target(a.selector), O::None)
            }
            Self::DblClick(a) => single(cmd("dblclick").target(a.selector), O::None),
            Self::Fill(a) => single(
                cmd("fill")
                    .target(a.selector)
                    .set("value", a.value.join(" ")),
                O::None,
            ),
            Self::Type(a) => single(
                cmd("type")
                    .target(a.selector)
                    .set("text", a.text.join(" "))
                    .on("clear", a.clear)
                    .opt("delay", a.delay),
                O::None,
            ),
            Self::Press(a) => single(cmd("press").set("key", a.key), O::None),
            Self::KeyDown(a) => single(cmd("keydown").set("key", a.key), O::None),
            Self::KeyUp(a) => single(cmd("keyup").set("key", a.key), O::None),
            Self::Hover(a) => single(cmd("hover").target(a.selector), O::None),
            Self::Focus(a) => single(cmd("focus").target(a.selector), O::None),
            Self::Check(a) => single(cmd("check").target(a.selector), O::None),
            Self::Uncheck(a) => single(cmd("uncheck").target(a.selector), O::None),
            Self::Select(a) => single(
                cmd("select").target(a.selector).set("values", a.values),
                O::None,
            ),
            Self::Clear(a) => single(cmd("fill").target(a.selector).set("value", ""), O::None),
            Self::SelectAll(a) => single(
                cmd("press").target(a.selector).set("key", "Control+a"),
                O::None,
            ),
            Self::Scroll(a) => single(
                cmd("scroll")
                    .opt("direction", a.direction)
                    .opt("amount", a.amount)
                    .opt("target", a.selector),
                O::None,
            ),
            Self::Drag(a) => single(
                cmd("drag").set("source", a.source).target(a.target),
                O::None,
            ),
            Self::Upload(a) => {
                let mut uploads = Vec::new();
                let mut files = Vec::new();
                for file in a.files {
                    let path = PathBuf::from(&file);
                    if !path.is_file() {
                        files.push(file);
                        continue;
                    }
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| anyhow!("Invalid file name: {file}"))?
                        .to_string();
                    files.push(name.clone());
                    uploads.push((path, name));
                }
                let mut step = Step::new(
                    vec![cmd("upload").target(a.selector).set("files", files).done()],
                    O::None,
                );
                step.uploads = uploads;
                Ok(Plan::Run(step))
            }
            Self::Mouse(a) => {
                if a.x.is_some() != a.y.is_some() {
                    bail!("Pass x and y together.");
                }
                if a.x.is_none() && !matches!(a.action.as_str(), "down" | "up") {
                    bail!("mouse {} needs x and y.", a.action);
                }
                single(
                    cmd("mouse")
                        .set("action", a.action)
                        .opt("x", a.x)
                        .opt("y", a.y)
                        .opt("button", a.button)
                        .opt("clicks", a.clicks)
                        .opt("deltaX", a.dx)
                        .opt("deltaY", a.dy),
                    O::None,
                )
            }
            Self::Snapshot(a) => {
                if a.selector.is_some() {
                    return Err(removed(
                        "snapshot --selector",
                        "Use `steel browser get text <selector>` or `steel browser get html <selector>` for one element.",
                    ));
                }
                if a.urls {
                    return Err(removed(
                        "snapshot --urls",
                        "Read a link with `steel browser get attr <ref> href`.",
                    ));
                }
                single(
                    cmd("snapshot")
                        .on("interactive", a.interactive)
                        .on("compact", a.compact)
                        .opt("depth", a.max_depth),
                    O::Snapshot,
                )
            }
            Self::Screenshot(a) => {
                if a.annotate {
                    return Err(removed(
                        "screenshot --annotate",
                        "Take a snapshot for refs with `steel browser snapshot -i`.",
                    ));
                }
                let format = a
                    .format
                    .or_else(|| image_format(&a.output).map(String::from));
                single(
                    cmd("screenshot")
                        .opt("target", a.selector)
                        .on("fullPage", a.full_page)
                        .opt("format", format)
                        .opt("quality", a.quality),
                    O::Saved(absolute(&a.output)?),
                )
            }
            Self::Pdf(a) => single(
                cmd("pdf")
                    .on("landscape", a.landscape)
                    .on("printBackground", a.print_background),
                O::Saved(absolute(&a.output)?),
            ),
            Self::Eval(a) => single(cmd("eval").set("script", a.script), O::Value),
            Self::Find(a) => single(
                cmd("find")
                    .set("by", a.by)
                    .set("value", a.value)
                    .opt("name", a.name)
                    .on("exact", a.exact),
                O::Find,
            ),
            Self::Content => single(
                cmd("get").set("what", "html").target("html".to_string()),
                O::Value,
            ),
            Self::Get { command } => {
                let get = |what: &str| cmd("get").set("what", what);
                match command {
                    GetCommand::Text(a) => single(get("text").target(a.selector), O::Value),
                    GetCommand::Html(a) => single(get("html").target(a.selector), O::Value),
                    GetCommand::Markdown(a) => {
                        single(get("markdown").opt("target", a.selector), O::Value)
                    }
                    GetCommand::Value(a) => single(get("value").target(a.selector), O::Value),
                    GetCommand::Attr(a) => single(
                        get("attr").target(a.selector).set("attribute", a.attribute),
                        O::Value,
                    ),
                    GetCommand::Url => single(get("url"), O::Value),
                    GetCommand::Title => single(get("title"), O::Value),
                    GetCommand::Count(a) => single(get("count").target(a.selector), O::Value),
                    GetCommand::Box(a) => single(get("box").target(a.selector), O::Value),
                    GetCommand::Styles(_) => Err(removed(
                        "get styles",
                        "Use `steel browser eval \"getComputedStyle(document.querySelector('<selector>')).color\"`.",
                    )),
                }
            }
            Self::Is { command } => {
                let (what, selector) = match command {
                    IsCommand::Visible(a) => ("visible", a.selector),
                    IsCommand::Enabled(a) => ("enabled", a.selector),
                    IsCommand::Checked(a) => ("checked", a.selector),
                };
                single(cmd("is").set("what", what).target(selector), O::Value)
            }
            Self::Wait(a) => plan_wait(a),
            Self::Tab { command } => match command {
                TabCommand::List => single(cmd("tab").set("action", "list"), O::Tabs),
                TabCommand::New(a) => single(
                    cmd("tab").set("action", "new").opt("url", a.url),
                    O::TabOpened,
                ),
                TabCommand::Switch(a) => single(cmd("tab").set("action", "list"), O::Switch(a.tab)),
                TabCommand::Close(a) => {
                    single(cmd("tab").set("action", "close").opt("tab", a.tab), O::None)
                }
            },
            Self::Dialog(a) => Ok(Plan::Control(
                cmd("dialog")
                    .opt("dialog", a.id)
                    .set("action", a.action)
                    .opt("text", a.text)
                    .done(),
                O::Dialog,
            )),
            Self::Status(a) => Ok(Plan::Control(
                cmd("status").set("execution", a.execution).done(),
                O::Execution,
            )),
            Self::Cancel(a) => Ok(Plan::Control(
                cmd("cancel").set("execution", a.execution).done(),
                O::Execution,
            )),
            Self::Close => Ok(Plan::Close),
            Self::Cookies(_) => Err(removed(
                "cookies",
                "Read cookies with `steel browser eval \"document.cookie\"` (HttpOnly cookies are not visible). Keep logins across sessions with `steel browser start --profile <name>`.",
            )),
            Self::Storage(_) => Err(removed(
                "storage",
                "Use `steel browser eval \"localStorage.getItem('key')\"` or `steel browser eval \"localStorage.setItem('key', 'value')\"`.",
            )),
            Self::Set(_) => Err(removed(
                "set",
                "The viewport, geolocation, offline mode, headers, and user agent cannot be changed on a running session.",
            )),
            Self::Highlight(_) => Err(removed(
                "highlight",
                "Capture the element with `steel browser screenshot --selector <selector>`.",
            )),
            Self::BringToFront(_) => Err(removed(
                "bringtofront",
                "Commands bring their tab to the front on their own.",
            )),
            Self::Diff(_) => Err(removed(
                "diff",
                "Add `--diff` to a command to see what it changed on the page.",
            )),
            Self::ScrollIntoView(_) => Err(removed(
                "scrollintoview",
                "Use `steel browser hover <selector>` (it moves the mouse), or `steel browser scroll --selector <container>`.",
            )),
            Self::SetValue(_) => Err(removed(
                "setvalue",
                "Use `steel browser fill <selector> <value>`.",
            )),
        }
    }
}

fn plan_wait(a: WaitArgs) -> Result<Plan> {
    if a.function.is_some() {
        return Err(removed(
            "wait --function",
            "Use `steel browser wait --text`, `--selector`, or `--url`, or check with `steel browser eval`.",
        ));
    }
    if a.state.is_some() && a.selector.is_none() {
        bail!("--state needs --selector.");
    }
    let conditions = [
        a.ms.is_some(),
        a.selector.is_some(),
        a.text.is_some(),
        a.url.is_some(),
        a.load_state.is_some(),
        a.download,
    ];
    if conditions.iter().filter(|given| **given).count() != 1 {
        bail!(
            "Pass exactly one of: a time in ms, --selector, --text, --url, --load, or --download."
        );
    }
    let url = a.url.map(|url| {
        if url.contains('*') {
            url
        } else {
            format!("*{url}*")
        }
    });
    let timeout = a.ms.is_none().then_some(a.timeout);
    let output = if a.download {
        Output::Download(a.output.as_deref().map(absolute).transpose()?)
    } else {
        Output::None
    };
    single(
        cmd("wait")
            .opt("ms", a.ms)
            .opt("target", a.selector)
            .opt("state", a.state)
            .opt("text", a.text)
            .opt("url", url)
            .opt("load", a.load_state)
            .on("download", a.download)
            .opt("timeout", timeout),
        output,
    )
}

fn is_ref(target: &str) -> bool {
    let target = target.trim();
    let target = target
        .strip_prefix('@')
        .or_else(|| target.strip_prefix("ref="))
        .unwrap_or(target);
    target
        .strip_prefix('e')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

fn scoped(mut command: Value, tab: Option<&str>) -> Value {
    let Some(tab) = tab else {
        return command;
    };
    let Some(map) = command.as_object_mut() else {
        return command;
    };
    let eligible = match map.get("command").and_then(Value::as_str) {
        Some("status" | "cancel" | "dialog") | None => false,
        Some("tab") => map.get("action").and_then(Value::as_str) == Some("close"),
        Some(_) => true,
    };
    let names_ref = ["target", "source"]
        .iter()
        .any(|key| map.get(*key).and_then(Value::as_str).is_some_and(is_ref));
    if eligible && !names_ref && !map.contains_key("tab") {
        map.insert("tab".into(), json!(tab));
    }
    command
}

fn snapshot_options(command: &Value, id: &str) -> SnapshotInfo {
    SnapshotInfo {
        id: id.to_string(),
        interactive: command["interactive"] == true,
        compact: command["compact"] == true,
        depth: command["depth"].as_u64(),
    }
}

fn show(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Bool(_) | Value::Number(_) => Some(value.to_string()),
        other => serde_json::to_string_pretty(other).ok(),
    }
}

fn session_flag(name: &str) -> String {
    if name == DEFAULT_SESSION {
        String::new()
    } else {
        format!(" --session {name}")
    }
}

pub fn fail(code: &str, message: &str, data: Option<&Value>) -> anyhow::Error {
    if output::is_json() {
        let mut obj = json!({"success": false, "error": message, "error_code": code});
        if let Some(data) = data {
            obj["data"] = data.clone();
        }
        println!("{obj}");
    } else {
        eprintln!("Error: {message}");
    }
    SilentExit(output::exit_code::GENERAL).into()
}

fn hint(code: &str, message: &str) -> &'static str {
    match code {
        "tab_not_found" | "browser_restarted" if message.contains("tab list") => "",
        "stale_ref" => " Run `steel browser snapshot -i`.",
        "tab_required" => " Run `steel browser tab switch <tab>`.",
        "tab_not_found" => " Run `steel browser tab list`.",
        "dialog_pending" => " Run `steel browser dialog accept` or `steel browser dialog dismiss`.",
        "browser_restarted" => " Run `steel browser tab list`, then `steel browser snapshot -i`.",
        "missing_capability" => " Run `steel browser start`.",
        "invalid_operation_token" => " If it did not run, run it again.",
        "queue_full" | "expired" | "records_full" => " The session is busy. Try again in a moment.",
        _ => "",
    }
}

fn failure(body: &Value) -> Option<(String, String)> {
    let execution = &body["execution"];
    let state = execution["state"].as_str()?;
    if state == "completed" {
        return None;
    }
    if state == "running" {
        let id = execution["id"].as_str().unwrap_or("");
        return Some((
            "running".to_string(),
            format!("The command is still running. Wait for it with `steel browser status {id}`."),
        ));
    }
    let error = body["results"]
        .as_array()
        .and_then(|results| results.iter().find(|r| r["ok"] == false))
        .map(|r| &r["error"])
        .filter(|e| e.is_object())
        .unwrap_or_else(|| &execution["error"]);
    let code = error["code"].as_str().unwrap_or(state).to_string();
    let message = error["message"].as_str().unwrap_or("The command failed.");
    let effect = match execution["effect"].as_str() {
        Some("applied") => " Input already reached the page (effect: applied).",
        Some("unknown") => {
            " It is unknown whether the action took effect. Check the page before repeating it."
        }
        _ => "",
    };
    Some((
        code.clone(),
        format!("{code}: {message}{effect}{}", hint(&code, message)),
    ))
}

fn notices(
    body: &Value,
    events: bool,
    current_tab: Option<&str>,
    downloading: bool,
) -> Vec<String> {
    let mut notes = Vec::new();
    if body["execution"]["replayed"] == true {
        notes.push("This is the stored result of an earlier run. Nothing ran again.".to_string());
    }
    if body["epochChanged"] == true {
        notes.push(
            "The page changed since your last command, by another client or a dialog answer. Take a new snapshot before using old refs."
                .to_string(),
        );
    }
    if body["bodyExpired"] == true {
        notes.push(
            "The stored result is older than 10 minutes, so its content is gone.".to_string(),
        );
    }
    let loading = body["results"]
        .as_array()
        .is_some_and(|results| results.iter().any(|r| r["loading"] == true));
    if loading {
        notes.push(
            "The page is still loading. Wait for it with `steel browser wait --load load`."
                .to_string(),
        );
    }
    if let Some(dialog) = body.get("dialog").filter(|d| d.is_object()) {
        notes.push(format!(
            "Dialog {} ({}) is open on {}: {}. Answer it with `steel browser dialog accept` or `steel browser dialog dismiss`.",
            dialog["id"].as_str().unwrap_or("?"),
            dialog["type"].as_str().unwrap_or("dialog"),
            dialog["tab"].as_str().unwrap_or("?"),
            dialog["message"],
        ));
    }
    if let Some(message) = body["observe"]["error"]["message"].as_str() {
        notes.push(format!("Could not take the diff: {message}"));
    }
    if !events {
        return notes;
    }
    for item in body["events"]["items"].as_array().into_iter().flatten() {
        let tab = item["tab"].as_str().unwrap_or("?");
        match item["type"].as_str() {
            Some("tab_opened") if Some(tab) != current_tab => {
                let from = item["opener"]
                    .as_str()
                    .map(|opener| format!(" from {opener}"))
                    .unwrap_or_default();
                notes.push(format!(
                    "Tab {tab} opened{from}. Switch to it with `steel browser tab switch {tab}`."
                ));
            }
            Some("tab_closed") => notes.push(format!("Tab {tab} closed.")),
            Some("download") if !downloading && item["state"] == "completed" => {
                notes.push(format!(
                    "Downloaded {}. Save it with `steel browser wait --download`.",
                    item["file"].as_str().unwrap_or("a file")
                ));
            }
            _ => {}
        }
    }
    notes
}

fn find_lines(result: &Value) -> String {
    let count = result["count"].as_u64().unwrap_or(0);
    let mut lines = vec![format!("{count} found")];
    for found in result["matches"].as_array().into_iter().flatten() {
        lines.push(format!(
            "{} {} {}",
            found["ref"].as_str().unwrap_or("?"),
            found["role"].as_str().unwrap_or(""),
            found["name"]
        ));
    }
    lines.join("\n")
}

fn tab_line(tab: &Value, current: Option<&str>) -> String {
    let id = tab["tab"].as_str().unwrap_or("?");
    let marker = if Some(id) == current { "*" } else { "" };
    format!(
        "{id}{marker} {} — {}",
        tab["url"].as_str().unwrap_or(""),
        tab["title"].as_str().unwrap_or("")
    )
}

fn absorb_page(state: &mut BrowserSession, body: &Value) {
    if let Some(cursor) = body["events"]["cursor"].as_str() {
        state.events = Some(cursor.to_string());
    }
    if let Some(dialog) = body["dialog"]["id"].as_str() {
        state.dialog = Some(dialog.to_string());
    }
}

fn restarted(body: &Value) -> bool {
    body["execution"]["error"]["code"] == "browser_restarted"
        || body["results"].as_array().is_some_and(|results| {
            results
                .iter()
                .any(|r| r["error"]["code"] == "browser_restarted")
        })
}

pub struct Browser {
    client: SteelClient,
    auth: Auth,
    pub state: BrowserSession,
}

async fn attach(
    client: &SteelClient,
    mode: ApiMode,
    base_url: &str,
    auth: &Auth,
    id: &str,
) -> Result<BrowserSession> {
    let remote = RemoteSession::parse(client.cli_get_session(base_url, mode, auth, id).await?)?;
    if !remote.is_live() {
        bail!("Session {id} has ended. Start a new one with `steel browser start`.");
    }
    let mut attached =
        BrowserSession::new(id.to_string(), id.to_string(), mode, base_url.to_string());
    attached.live_url = remote.viewer_url(mode);
    state::save(&attached)?;
    Ok(attached)
}

impl Browser {
    pub async fn open(name: Option<&str>, start_default: bool) -> Result<Self> {
        let (mode, base_url, auth) = api::resolve_with_auth();
        let client = SteelClient::new()?;
        let state = match name {
            Some(name) => match state::load(name)? {
                Some(found) => found,
                None if state::looks_like_session_id(name) => {
                    attach(&client, mode, &base_url, &auth, name).await?
                }
                None => bail!(
                    "Session \"{name}\" is not running. It may have expired. Run `steel browser start --session {name}` to start a new one."
                ),
            },
            None => match state::load(DEFAULT_SESSION)? {
                Some(found) => found,
                None if start_default => {
                    eprintln!("warning: session \"default\" is not running; starting a new one");
                    let options = start::default_options();
                    start::create(&client, mode, &base_url, &auth, DEFAULT_SESSION, &options)
                        .await?
                        .0
                }
                None => bail!("No active browser session. Start one with `steel browser start`."),
            },
        };
        Ok(Self {
            client,
            auth,
            state,
        })
    }

    fn session(&self) -> Session<'_> {
        Session {
            client: &self.client,
            base_url: &self.state.api_url,
            mode: self.state.mode,
            auth: &self.auth,
            id: &self.state.id,
        }
    }

    fn save(&self) -> Result<()> {
        state::save(&self.state)
    }

    fn request(&self, steps: &[Step], diff: bool) -> (Vec<Value>, Value) {
        let commands: Vec<Value> = steps
            .iter()
            .flat_map(|step| step.commands.iter().cloned())
            .map(|command| scoped(command, self.state.tab.as_deref()))
            .collect();
        let mut body = json!({
            "commands": commands,
            "eventsAfter": self.state.events.as_deref().unwrap_or("c-0"),
        });
        if let Some(next) = &self.state.next {
            body["after"] = json!(next);
        }
        if diff {
            let mut observe = json!({"mode": "diff"});
            if let Some(snapshot) = &self.state.snapshot {
                observe["base"] = json!(snapshot.id);
                if snapshot.interactive {
                    observe["interactive"] = json!(true);
                }
                if snapshot.compact {
                    observe["compact"] = json!(true);
                }
                if let Some(depth) = snapshot.depth {
                    observe["depth"] = json!(depth);
                }
            }
            body["observe"] = observe;
        }
        (commands, body)
    }

    async fn send<T>(&self, run: impl Future<Output = Result<T, ApiError>>) -> Result<T> {
        match run.await {
            Ok(value) => Ok(value),
            Err(ApiError::Unreachable { .. }) => {
                let again = if self.state.next.is_some() {
                    "Running the same command again is safe: it will not run twice."
                } else {
                    "Check the page before running it again."
                };
                Err(fail(
                    "delivery_failed",
                    &format!(
                        "Could not reach session {}. The command may or may not have run. {again}",
                        self.state.id
                    ),
                    None,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    fn check(&mut self, reply: Reply) -> Result<Reply> {
        if reply.status == 200 {
            return Ok(reply);
        }
        let code = reply.error_code().unwrap_or("request_failed").to_string();
        let message = reply.error_message().map_or_else(
            || format!("Request failed with HTTP {}.", reply.status),
            str::to_string,
        );
        if matches!(reply.status, 401 | 403) {
            return Err(ApiError::RequestFailed {
                status: reply.status,
                message: message.into(),
                body: Some(reply.body),
            }
            .into());
        }
        if matches!(code.as_str(), "session_ended" | "session_not_found") {
            state::remove(&self.state.name)?;
            let start = session_flag(&self.state.name);
            return Err(fail(
                &code,
                &format!("{message} Run `steel browser start{start}` to start a new one."),
                Some(&reply.body),
            ));
        }
        if reply.status == 404 {
            let fix = if self.state.mode == ApiMode::Local {
                " Run steel-browser with ENABLE_COMMANDS_API=true."
            } else {
                ""
            };
            return Err(fail(
                "missing_capability",
                &format!(
                    "The Steel API at {} does not support browser commands.{fix}",
                    self.state.api_url
                ),
                Some(&reply.body),
            ));
        }
        if let Some(next) = reply.body.get("next").and_then(Value::as_str) {
            self.state.next = Some(next.to_string());
        }
        if code == "dialog_not_pending" {
            self.state.dialog = None;
        }
        self.save()?;
        Err(fail(
            &code,
            &format!("{message}{}", hint(&code, &message)),
            Some(&reply.body),
        ))
    }

    fn absorb(&mut self, commands: &[Value], body: &Value, observe: Option<&Value>) {
        if let Some(next) = body.get("next").and_then(Value::as_str) {
            self.state.next = Some(next.to_string());
        }
        if let Some(tab) = body["page"]["tab"].as_str() {
            self.state.tab = Some(tab.to_string());
        }
        absorb_page(&mut self.state, body);
        let results = body["results"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        for (command, result) in commands.iter().zip(results) {
            if let Some(id) = result["snapshotId"].as_str() {
                self.state.snapshot = Some(snapshot_options(command, id));
            }
            let closed = command["command"] == "tab"
                && command["action"] == "close"
                && result["ok"] == true
                && result["tab"].as_str() == self.state.tab.as_deref();
            if closed {
                self.state.tab = None;
            }
        }
        if let (Some(id), Some(observe)) = (body["observe"]["snapshotId"].as_str(), observe) {
            self.state.snapshot = Some(snapshot_options(observe, id));
        }
        if restarted(body) {
            self.state.tab = None;
            self.state.snapshot = None;
        }
    }

    pub async fn execute(&mut self, steps: Vec<Step>, diff: bool) -> Result<()> {
        for (path, name) in steps.iter().flat_map(|step| step.uploads.iter()) {
            let bytes = std::fs::read(path)
                .with_context(|| format!("Could not read {}", path.display()))?;
            self.send(self.session().upload_file(bytes, name)).await?;
        }
        let events = self.state.events.is_some();
        let tab_commands = steps
            .iter()
            .flat_map(|step| step.commands.iter())
            .any(|c| c["command"] == "tab");
        let mut retried = false;
        loop {
            let injected = self.state.tab.is_some();
            let (commands, body) = self.request(&steps, diff);
            let reply = self.send(self.session().run(body.clone())).await?;
            let mut reply = self.check(reply)?;
            self.absorb(&commands, &reply.body, body.get("observe"));
            let lost_tab = injected
                && !tab_commands
                && !retried
                && reply.body["execution"]["effect"] == "none"
                && failure(&reply.body).is_some_and(|(code, _)| code == "tab_not_found");
            if lost_tab {
                self.state.tab = None;
                self.save()?;
                retried = true;
                continue;
            }
            let result = self.report(&steps, &mut reply.body, events).await;
            self.save()?;
            return result;
        }
    }

    async fn report(&mut self, steps: &[Step], body: &mut Value, events: bool) -> Result<()> {
        let downloading = steps
            .iter()
            .any(|s| matches!(s.output, Output::Download(_)));
        let notes = notices(body, events, self.state.tab.as_deref(), downloading);
        let failed = failure(body);
        let mut lines = Vec::new();
        let mut index = 0;
        for step in steps {
            let at = index;
            index += step.commands.len();
            if body["results"][at]["ok"] != true {
                continue;
            }
            let mut result = body["results"][at].take();
            let rendered = self.render(&step.output, &mut result).await;
            body["results"][at] = result;
            lines.extend(rendered?);
        }
        if failed.is_none() {
            if body["observe"]["full"] == true
                && let Some(snapshot) = body["observe"]["snapshot"].as_str()
            {
                lines.push(snapshot.to_string());
            } else if let Some(diff) = body["observe"]["diff"].as_str() {
                lines.push(if diff.is_empty() {
                    "No changes.".to_string()
                } else {
                    diff.trim_end().to_string()
                });
            }
        }
        if output::is_json() {
            if !notes.is_empty() {
                body["notices"] = json!(notes);
            }
            return match failed {
                None => {
                    output::success_data(body.clone());
                    Ok(())
                }
                Some((code, message)) => Err(fail(&code, &message, Some(body))),
            };
        }
        for line in lines {
            println!("{line}");
        }
        for note in notes {
            eprintln!("note: {note}");
        }
        match failed {
            None => Ok(()),
            Some((code, message)) => Err(fail(&code, &message, None)),
        }
    }

    async fn render(&mut self, output: &Output, result: &mut Value) -> Result<Option<String>> {
        Ok(match output {
            Output::None | Output::Execution | Output::Dialog => None,
            Output::Snapshot => result["snapshot"].as_str().map(str::to_string),
            Output::Value => show(&result["value"]),
            Output::Find => Some(find_lines(result)),
            Output::Tabs => {
                let current = self.state.tab.as_deref();
                let tabs = result["tabs"]
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                Some(
                    tabs.iter()
                        .map(|tab| tab_line(tab, current))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
            Output::TabOpened => result["tab"].as_str().map(str::to_string),
            Output::Switch(wanted) => {
                let tabs = result["tabs"]
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let Some(line) = tabs
                    .iter()
                    .find(|tab| tab["tab"] == wanted.as_str())
                    .map(|tab| tab_line(tab, Some(wanted)))
                else {
                    let open: Vec<&str> = tabs.iter().filter_map(|t| t["tab"].as_str()).collect();
                    bail!("Tab {wanted} is not open. Open tabs: {}.", open.join(", "));
                };
                self.state.tab = Some(wanted.clone());
                result["tab"] = json!(wanted);
                Some(line)
            }
            Output::Saved(path) => {
                let data = result["data"]
                    .as_str()
                    .ok_or_else(|| anyhow!("The browser returned no data to save."))?;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .context("The browser returned data that is not base64.")?;
                std::fs::write(path, bytes)
                    .with_context(|| format!("Could not write {}", path.display()))?;
                if let Some(map) = result.as_object_mut() {
                    map.remove("data");
                }
                result["path"] = json!(path);
                Some(path.display().to_string())
            }
            Output::Download(path) => {
                let file = result["file"]
                    .as_str()
                    .ok_or_else(|| anyhow!("The browser did not name the downloaded file."))?
                    .to_string();
                let target = match path {
                    Some(path) => path.clone(),
                    None => {
                        let name = Path::new(&file)
                            .file_name()
                            .ok_or_else(|| anyhow!("Invalid download name: {file}"))?;
                        std::env::current_dir()?.join(name)
                    }
                };
                let bytes = self.send(self.session().download_file(&file)).await?;
                std::fs::write(&target, bytes)
                    .with_context(|| format!("Could not write {}", target.display()))?;
                result["path"] = json!(target);
                Some(target.display().to_string())
            }
        })
    }

    pub async fn control(&mut self, mut command: Value, output: Output) -> Result<()> {
        if command["command"] == "dialog" && command.get("dialog").is_none() {
            let Some(id) = self.state.dialog.clone() else {
                bail!(
                    "No open dialog is known. Pass its id, such as `steel browser dialog accept d-2`."
                );
            };
            command["dialog"] = json!(id);
        }
        let reply = self
            .send(self.session().control(command.clone(), None))
            .await?;
        let reply = self.check(reply)?;
        let body = reply.body;
        absorb_page(&mut self.state, &body);
        if matches!(output, Output::Dialog) {
            self.state.dialog = None;
        }
        self.save()?;
        let failed = failure(&body);
        if output::is_json() {
            return match failed {
                None => {
                    output::success_data(body);
                    Ok(())
                }
                Some((code, message)) => Err(fail(&code, &message, Some(&body))),
            };
        }
        if let Some(answered) = body.get("answered").filter(|a| a.is_object()) {
            let verb = if answered["accepted"] == true {
                "accepted"
            } else {
                "dismissed"
            };
            println!("Dialog {} {verb}.", answered["id"].as_str().unwrap_or("?"));
        } else {
            let execution = &body["execution"];
            println!(
                "{} {} (effect: {})",
                execution["id"].as_str().unwrap_or("?"),
                execution["state"].as_str().unwrap_or("?"),
                execution["effect"].as_str().unwrap_or("?")
            );
            for result in body["results"].as_array().into_iter().flatten() {
                println!("{result}");
            }
        }
        match failed {
            None => Ok(()),
            Some((code, message)) => Err(fail(&code, &message, None)),
        }
    }
}

pub async fn run(action: ActionCommand, session: Option<&str>, diff: bool) -> Result<()> {
    match action.plan()? {
        Plan::Run(step) => {
            let mut browser = Browser::open(session, true).await?;
            browser.execute(vec![step], diff).await
        }
        Plan::Control(command, output) => {
            let mut browser = Browser::open(session, false).await?;
            browser.control(command, output).await
        }
        Plan::Close => super::stop::run(super::stop::Args { all: false }, session).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(args: &[&str]) -> Result<Plan> {
        #[derive(Parser)]
        #[command(no_binary_name = true)]
        struct Parse {
            #[command(subcommand)]
            action: ActionCommand,
        }
        Parse::try_parse_from(args)
            .map_err(|e| anyhow!(e.to_string()))?
            .action
            .plan()
    }

    fn commands(args: &[&str]) -> Vec<Value> {
        match plan(args).unwrap() {
            Plan::Run(step) => step.commands,
            _ => panic!("expected a run plan"),
        }
    }

    #[test]
    fn click_maps_to_engine_command() {
        assert_eq!(
            commands(&["click", "@e5"]),
            vec![json!({"command": "click", "target": "@e5"})]
        );
        assert_eq!(
            commands(&["click", "#go", "--count", "2"]),
            vec![json!({"command": "dblclick", "target": "#go"})]
        );
    }

    #[test]
    fn fill_and_type_join_words() {
        assert_eq!(
            commands(&["fill", "e1", "herman", "miller"]),
            vec![json!({"command": "fill", "target": "e1", "value": "herman miller"})]
        );
        assert_eq!(
            commands(&["type", "e1", "hi", "--clear", "--delay", "20"]),
            vec![
                json!({"command": "type", "target": "e1", "text": "hi", "clear": true, "delay": 20})
            ]
        );
    }

    #[test]
    fn navigate_with_networkidle_adds_a_wait() {
        assert_eq!(
            commands(&["open", "example.com", "--wait-until", "networkidle"]),
            vec![
                json!({"command": "navigate", "url": "example.com"}),
                json!({"command": "wait", "load": "networkidle"}),
            ]
        );
        assert_eq!(commands(&["navigate", "example.com"]).len(), 1);
    }

    #[test]
    fn get_commands_send_a_target_where_the_engine_needs_one() {
        assert_eq!(
            commands(&["content"]),
            vec![json!({"command": "get", "what": "html", "target": "html"})]
        );
        assert_eq!(
            commands(&["get", "text", "p"]),
            vec![json!({"command": "get", "what": "text", "target": "p"})]
        );
        assert_eq!(
            commands(&["get", "markdown"]),
            vec![json!({"command": "get", "what": "markdown"})]
        );
        assert_eq!(
            commands(&["get", "title"]),
            vec![json!({"command": "get", "what": "title"})]
        );
        assert!(plan(&["get", "text"]).is_err());
        assert!(plan(&["get", "html"]).is_err());
    }

    #[test]
    fn clear_and_selectall_use_engine_commands() {
        assert_eq!(
            commands(&["clear", "e2"]),
            vec![json!({"command": "fill", "target": "e2", "value": ""})]
        );
        assert_eq!(
            commands(&["selectall", "e2"]),
            vec![json!({"command": "press", "target": "e2", "key": "Control+a"})]
        );
    }

    #[test]
    fn snapshot_options_are_sent_only_when_set() {
        assert_eq!(
            commands(&["snapshot"]),
            vec![json!({"command": "snapshot"})]
        );
        assert_eq!(
            commands(&["snapshot", "-i", "-c", "-d", "3"]),
            vec![json!({"command": "snapshot", "interactive": true, "compact": true, "depth": 3})]
        );
    }

    #[test]
    fn screenshot_format_follows_the_extension() {
        let Plan::Run(step) = plan(&["screenshot", "-o", "shot.jpg", "--full"]).unwrap() else {
            panic!("expected a run plan");
        };
        assert_eq!(
            step.commands,
            vec![json!({"command": "screenshot", "fullPage": true, "format": "jpeg"})]
        );
        assert!(matches!(step.output, Output::Saved(ref p) if p.ends_with("shot.jpg")));
    }

    #[test]
    fn wait_url_matches_as_substring_unless_it_has_a_star() {
        assert_eq!(
            commands(&["wait", "--url", "checkout"]),
            vec![json!({"command": "wait", "url": "*checkout*", "timeout": 30000})]
        );
        assert_eq!(
            commands(&["wait", "--url", "https://x.com/*"]),
            vec![json!({"command": "wait", "url": "https://x.com/*", "timeout": 30000})]
        );
    }

    #[test]
    fn wait_ms_and_selector_state() {
        assert_eq!(
            commands(&["wait", "1500"]),
            vec![json!({"command": "wait", "ms": 1500})]
        );
        assert_eq!(
            commands(&[
                "wait",
                "--selector",
                "e4",
                "--state",
                "hidden",
                "--timeout",
                "5000"
            ]),
            vec![json!({"command": "wait", "target": "e4", "state": "hidden", "timeout": 5000})]
        );
    }

    #[test]
    fn wait_needs_exactly_one_condition() {
        assert!(plan(&["wait"]).is_err());
        assert!(plan(&["wait", "--text", "a", "--url", "b"]).is_err());
        assert!(plan(&["wait", "--state", "visible"]).is_err());
    }

    #[test]
    fn find_defaults_to_css() {
        assert_eq!(
            commands(&["find", "button.primary"]),
            vec![json!({"command": "find", "by": "css", "value": "button.primary"})]
        );
        assert_eq!(
            commands(&[
                "find", "button", "--by", "role", "--name", "Save", "--exact"
            ]),
            vec![
                json!({"command": "find", "by": "role", "value": "button", "name": "Save", "exact": true})
            ]
        );
    }

    #[test]
    fn tab_commands() {
        assert_eq!(
            commands(&["tab", "new", "https://example.com"]),
            vec![json!({"command": "tab", "action": "new", "url": "https://example.com"})]
        );
        assert_eq!(
            commands(&["tab", "close", "t3"]),
            vec![json!({"command": "tab", "action": "close", "tab": "t3"})]
        );
        assert!(plan(&["tab", "switch", "2"]).is_err());
        assert!(matches!(
            plan(&["tab", "switch", "t2"]).unwrap(),
            Plan::Run(Step { output: Output::Switch(ref t), .. }) if t == "t2"
        ));
    }

    #[test]
    fn upload_keeps_session_file_names_and_uploads_local_files() {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("cv.pdf");
        std::fs::write(&local, b"pdf").unwrap();
        let Plan::Run(step) =
            plan(&["upload", "e3", local.to_str().unwrap(), "already.txt"]).unwrap()
        else {
            panic!("expected a run plan");
        };
        assert_eq!(
            step.commands,
            vec![json!({"command": "upload", "target": "e3", "files": ["cv.pdf", "already.txt"]})]
        );
        assert_eq!(step.uploads, vec![(local, "cv.pdf".to_string())]);
    }

    #[test]
    fn control_commands() {
        assert!(matches!(
            plan(&["dialog", "accept", "d-2", "--text", "yes"]).unwrap(),
            Plan::Control(ref c, Output::Dialog)
                if *c == json!({"command": "dialog", "dialog": "d-2", "action": "accept", "text": "yes"})
        ));
        assert!(matches!(
            plan(&["status", "x-7"]).unwrap(),
            Plan::Control(ref c, Output::Execution) if *c == json!({"command": "status", "execution": "x-7"})
        ));
        assert!(matches!(plan(&["close"]).unwrap(), Plan::Close));
    }

    #[test]
    fn removed_commands_explain_what_to_use() {
        for args in [
            vec!["cookies"],
            vec!["cookies", "set", "a", "b"],
            vec!["storage", "local", "key"],
            vec!["set", "viewport", "100", "100"],
            vec!["highlight", "e1"],
            vec!["bringtofront"],
            vec!["diff", "snapshot"],
            vec!["scrollintoview", "e1"],
            vec!["setvalue", "e1", "x"],
            vec!["get", "styles", "e1"],
            vec!["click", "e1", "--new-tab"],
            vec!["click", "e1", "--button", "right"],
            vec!["snapshot", "-s", "#main"],
            vec!["screenshot", "--annotate"],
            vec!["wait", "--function", "() => true"],
            vec!["navigate", "x.com", "--header", "A: b"],
        ] {
            let err = plan(&args)
                .err()
                .unwrap_or_else(|| panic!("{args:?} should fail"));
            assert!(
                err.to_string().contains("was removed in 0.5"),
                "{args:?}: {err}"
            );
        }
    }

    #[test]
    fn mouse_needs_a_point_except_for_down_and_up() {
        assert_eq!(
            commands(&["mouse", "wheel", "10", "20", "--dy", "-300"]),
            vec![
                json!({"command": "mouse", "action": "wheel", "x": 10.0, "y": 20.0, "deltaY": -300.0})
            ]
        );
        assert_eq!(
            commands(&["mouse", "up"]),
            vec![json!({"command": "mouse", "action": "up"})]
        );
        for args in [
            vec!["mouse", "wheel", "--dy", "100"],
            vec!["mouse", "move"],
            vec!["mouse", "click", "5"],
        ] {
            assert!(plan(&args).is_err(), "{args:?} should fail");
        }
    }

    #[test]
    fn hints_are_not_repeated_when_the_engine_already_says_so() {
        let failed = |code: &str, message: &str| {
            json!({
                "execution": {"state": "failed", "effect": "none"},
                "results": [{"ok": false, "error": {"code": code, "message": message}}]
            })
        };
        let (_, message) = failure(&failed(
            "tab_not_found",
            "Tab t3 is not open. Run tab list to see open tabs.",
        ))
        .unwrap();
        assert!(!message.contains("steel browser tab list"), "{message}");
        let (_, message) = failure(&failed("tab_not_found", "Tab t3 is closed.")).unwrap();
        assert!(
            message.ends_with("Run `steel browser tab list`."),
            "{message}"
        );
        let (_, message) = failure(&failed(
            "browser_restarted",
            "The browser restarted, so tabs and refs changed. Run tab list again.",
        ))
        .unwrap();
        assert!(!message.contains("steel browser"), "{message}");
    }

    #[test]
    fn refs_are_recognized() {
        assert!(is_ref("e5"));
        assert!(is_ref("@e12"));
        assert!(is_ref("ref=e3"));
        assert!(is_ref(" e7 "));
        assert!(!is_ref("e"));
        assert!(!is_ref("#e5"));
        assert!(!is_ref("button"));
        assert!(!is_ref("email"));
    }

    #[test]
    fn current_tab_is_added_unless_a_ref_names_it() {
        assert_eq!(
            scoped(json!({"command": "snapshot"}), Some("t2")),
            json!({"command": "snapshot", "tab": "t2"})
        );
        assert_eq!(
            scoped(json!({"command": "click", "target": "#go"}), Some("t2")),
            json!({"command": "click", "target": "#go", "tab": "t2"})
        );
        assert_eq!(
            scoped(json!({"command": "click", "target": "@e5"}), Some("t2")),
            json!({"command": "click", "target": "@e5"})
        );
        assert_eq!(
            scoped(
                json!({"command": "drag", "source": "e1", "target": "#b"}),
                Some("t2")
            ),
            json!({"command": "drag", "source": "e1", "target": "#b"})
        );
        assert_eq!(
            scoped(json!({"command": "tab", "action": "list"}), Some("t2")),
            json!({"command": "tab", "action": "list"})
        );
        assert_eq!(
            scoped(json!({"command": "tab", "action": "close"}), Some("t2")),
            json!({"command": "tab", "action": "close", "tab": "t2"})
        );
        assert_eq!(
            scoped(json!({"command": "snapshot"}), None),
            json!({"command": "snapshot"})
        );
    }

    #[test]
    fn failure_reports_the_failing_command() {
        let body = json!({
            "execution": {"id": "x-1", "state": "failed", "effect": "applied"},
            "results": [
                {"ok": true, "effect": "applied"},
                {"ok": false, "effect": "none", "error": {"code": "stale_ref", "message": "Ref e5 is not known."}}
            ]
        });
        let (code, message) = failure(&body).unwrap();
        assert_eq!(code, "stale_ref");
        assert!(message.contains("Ref e5 is not known."));
        assert!(message.contains("effect: applied"));
        assert!(message.ends_with("Run `steel browser snapshot -i`."));
        assert!(failure(&json!({"execution": {"state": "completed"}})).is_none());
    }

    #[test]
    fn notices_cover_replays_dialogs_and_popups() {
        let body = json!({
            "execution": {"state": "completed", "replayed": true},
            "dialog": {"id": "d-2", "type": "confirm", "tab": "t1", "message": "Sure?"},
            "results": [{"ok": true, "loading": true}],
            "events": {"cursor": "c-5", "items": [
                {"type": "tab_opened", "tab": "t3", "opener": "t1"},
                {"type": "navigation", "tab": "t1"},
            ]}
        });
        let notes = notices(&body, true, Some("t1"), false);
        assert!(notes.iter().any(|n| n.contains("Nothing ran again")));
        assert!(notes.iter().any(|n| n.contains("Dialog d-2 (confirm)")));
        assert!(notes.iter().any(|n| n.contains("still loading")));
        assert!(notes.iter().any(|n| n.contains("tab switch t3")));
        assert_eq!(notes.len(), 4);
        assert_eq!(notices(&body, false, Some("t1"), false).len(), 3);
    }

    #[test]
    fn values_print_plainly() {
        assert_eq!(show(&json!("hi")), Some("hi".to_string()));
        assert_eq!(show(&json!(true)), Some("true".to_string()));
        assert_eq!(show(&json!(3)), Some("3".to_string()));
        assert_eq!(show(&Value::Null), None);
        assert!(show(&json!({"x": 1})).unwrap().contains("\"x\": 1"));
    }
}
