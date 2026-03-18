pub mod browser;
pub mod cache;
pub mod config;
pub mod credentials;
pub mod dev;
pub mod forge;
pub mod login;
pub mod logout;
pub mod pdf;
pub mod profile;
pub mod scrape;
pub mod screenshot;
pub mod update;

use clap::{Parser, Subcommand, builder::styling::Styles};

const LONG_HELP: &str = "\
Global Flags:
  --json                                 Structured JSON output for automation
  -l, --local                            Use local Steel runtime
  --api-url <url>                        Explicit self-hosted API endpoint URL

Quick Actions:
  steel scrape <url>                   Scrape webpage content (markdown by default)
    --format <fmt>                       Output formats: html, readability, cleaned_html, markdown
    -d, --delay <ms>                     Delay before scraping
    --pdf / --screenshot                 Include PDF or screenshot in response
    --use-proxy                          Use Steel-managed residential proxy
    -r, --region <region>                Region for request execution
    -l, --local                          Use local Steel runtime
  steel screenshot <url>               Capture a screenshot
    -f, --full-page                      Full-page capture
  steel pdf <url>                      Generate a PDF from a webpage

Browser Sessions:
  steel browser start [--session <name>]      Create or attach to a browser session
    --stealth                            Enable stealth mode (humanize + auto CAPTCHA)
    -p, --proxy <proxy>                  Use residential proxy
    --session-timeout <ms>               Session timeout
    --session-solve-captcha              Enable manual CAPTCHA solving
    --profile <name>                     Named profile to persist browser state
    --update-profile                     Save state back to profile on session end
    --namespace <ns> / --credentials     Inject stored credentials
  steel browser stop [--session <name>]       Stop a session (-a for all)
  steel browser sessions               List active sessions (use --json for structured output)
  steel browser live [--session <name>]       Open live session viewer

Browser Navigation:
  steel browser navigate <url>         Navigate to URL (aliases: open, goto)
    --wait-until <state>                 load, domcontentloaded, networkidle
    --header <KEY: VALUE>                Set request header (repeatable)
  steel browser back                   Go back in history
  steel browser forward                Go forward in history
  steel browser reload                 Reload current page

Browser Interaction:
  steel browser click <sel>            Click an element
    --button <btn>                       left, right, middle
    --count <n>                          Click count (2 for double-click)
    --new-tab                            Open link in new tab
  steel browser dblclick <sel>         Double-click an element
  steel browser fill <sel> <value>     Clear and fill an input field
  steel browser type <sel> <text>      Type text (appends to existing value)
    --clear                              Clear field first
    --delay <ms>                         Delay between keystrokes
  steel browser press <key>            Press key (Enter, Escape, Tab, Control+a)
  steel browser hover <sel>            Hover over an element
  steel browser focus <sel>            Focus an element
  steel browser check <sel>            Check a checkbox or radio button
  steel browser uncheck <sel>          Uncheck a checkbox
  steel browser select <sel> <vals>    Select dropdown option(s)
  steel browser clear <sel>            Clear an input field
  steel browser selectall <sel>        Select all text in an input
  steel browser scroll [dir] [px]      Scroll page (up/down/left/right, default: down 300)
    -s, --selector <sel>                 Scroll within an element
  steel browser scrollintoview <sel>   Scroll element into view
  steel browser setvalue <sel> <val>   Set input value (no events triggered)

Browser Observation:
  steel browser snapshot               Accessibility tree snapshot
    -i, --interactive                    Only interactive elements
    -s, --selector <sel>                 Scope to CSS selector
    -c, --compact                        Compact output
    -d, --max-depth <n>                  Limit tree depth
    -C, --cursor                         Include cursor position
  steel browser screenshot             Take a screenshot
    --full-page / --full                 Full scrollable page
    -o, --output <path>                  Output path (default: screenshot.png)
    --selector <sel>                     Restrict to element
    --format <fmt>                       png, jpeg, webp
    --quality <n>                        JPEG/WebP quality (0-100)
    --annotate                           Annotate interactive elements
  steel browser eval <script>          Run JavaScript in page
  steel browser find <selector>        Find all matching elements
  steel browser content                Get page HTML

Get Info:  steel browser get <what> [selector]
  text <sel>                             Element text content
  html <sel>                             Inner HTML
  value <sel>                            Input/textarea value
  attr <sel> <name>                      Element attribute
  url                                    Current page URL
  title                                  Current page title
  count <sel>                            Count matching elements
  box <sel>                              Element bounding box
  styles <sel> [--property <p>...]       CSS styles (all computed if no property)

Check State:  steel browser is <what> <selector>
  visible, enabled, checked

Wait:
  steel browser wait                   Wait for a condition
    --timeout <ms>                       Timeout (default: 30000)
    -t, --text <text>                    Wait for text on page
    --selector <sel>                     Wait for CSS selector
    --state <state>                      visible, hidden, attached, detached
    -u, --url <substring>                Wait for URL to contain string
    -f, --function <js>                  Wait for JS function to return truthy
    -l, --load-state <state>             load, domcontentloaded, networkidle

Tabs:
  steel browser tab list               List open tabs
  steel browser tab new [url]          Open new tab
  steel browser tab switch <index>     Switch to tab by index
  steel browser tab close [index]      Close tab (active if no index)

Window:
  steel browser bringtofront           Bring browser window to foreground
  steel browser close                  Close browser session (aliases: quit, exit)

CAPTCHA:
  steel browser captcha solve          Solve CAPTCHA
    --session-id <id>                      Session ID override
    --page-id <id> / --task-id <id>      Target specific CAPTCHA
  steel browser captcha status         Check CAPTCHA status
    -w, --wait                           Wait for terminal status
    --timeout <ms> / --interval <ms>     Poll settings

Credentials:
  steel credentials list               List stored credentials
    -n, --namespace <ns>                 Filter by namespace
    --origin <url>                       Filter by origin
  steel credentials create             Create credential
    --origin <url>                       Origin URL
    -u <user> -p <pass>                  Username and password
    --totp-secret <secret>               TOTP for 2FA
    -n, --namespace <ns>                 Namespace
    --label <label>                      Human-readable label
  steel credentials update             Update credential (same flags as create)
  steel credentials delete             Delete credential

Profiles:
  steel profile list                   List named browser profiles
  steel profile import --name <n>      Import Chrome profile (--from \"Profile 1\")
  steel profile sync --name <n>        Sync profile from Chrome
  steel profile delete --name <n>      Delete a profile

Development:
  steel dev install                    Install local Steel Browser runtime
    --repo-url <url>                     Custom git repository URL
  steel dev start                      Start local runtime
    -p, --port <port>                    API port
    -d, --docker-check                   Only verify Docker, then exit
  steel dev stop                       Stop local runtime

Project Scaffolding:
  steel forge [template] [-n <name>]   Create a new project from a template

Other:
  steel login                          Login to Steel (alias: auth)
  steel logout                         Logout from Steel
  steel config                         Show current configuration
  steel update                         Update to latest version
  steel cache [-c]                     Manage CLI cache (-c to clean)

Environment:
  STEEL_API_KEY                        API key for Steel (overrides login)
  STEEL_API_URL                        Steel API endpoint
  STEEL_BROWSER_API_URL                Steel Browser API endpoint
  STEEL_LOCAL_API_URL                  Local runtime API endpoint
  STEEL_CONFIG_DIR                     Custom config directory

Examples:
  steel scrape https://example.com
  steel scrape https://example.com --format html,markdown
  steel screenshot https://example.com -f
  steel browser start --session my-session
  steel browser navigate https://example.com
  steel browser snapshot -i
  steel browser click \"button#submit\"
  steel browser fill \"input[name=email]\" \"user@example.com\"
  steel browser type \"textarea\" \"Hello world\" --clear
  steel browser get text \"h1\"
  steel browser is visible \".modal\"
  steel browser wait --text \"Success\" --timeout 5000
  steel browser screenshot --full --annotate
  steel browser eval \"document.title\"
  steel browser tab new https://example.com
  steel browser stop --session my-session";

#[derive(Parser)]
#[command(
    name = "steel",
    version,
    about = "Steel CLI - browser automation for AI agents",
    long_about = "Steel CLI - browser automation for AI agents",
    after_long_help = LONG_HELP,
    styles = Styles::plain(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output all results as JSON (structured output for automation)
    #[arg(long, global = true)]
    pub json: bool,

    /// Skip update check (reserved for future use)
    #[arg(long, global = true, hide = true)]
    pub no_update_check: bool,

    /// Use local Steel runtime instead of cloud
    #[arg(short, long, global = true)]
    pub local: bool,

    /// Explicit self-hosted API endpoint URL
    #[arg(long, global = true)]
    pub api_url: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Internal daemon process
    #[command(name = "__daemon", hide = true)]
    Daemon {
        #[arg(long)]
        session_name: String,
    },

    /// Scrape webpage content (markdown output by default)
    Scrape(scrape::Args),

    /// Capture a screenshot of a webpage
    Screenshot(screenshot::Args),

    /// Generate a PDF from a webpage
    Pdf(pdf::Args),

    /// Browser session management and automation
    Browser(browser::BrowserArgs),

    /// Login to Steel CLI
    #[command(alias = "auth")]
    Login(login::Args),

    /// Logout from Steel CLI
    Logout(logout::Args),

    /// Manage stored credentials
    Credentials {
        #[command(subcommand)]
        command: credentials::Command,
    },

    /// Local development runtime
    Dev {
        #[command(subcommand)]
        command: dev::Command,
    },

    /// Scaffold a new project from a template
    Forge(forge::Args),

    /// Show current configuration
    Config(config::Args),

    /// Update to the latest version
    Update(update::Args),

    /// Manage Steel CLI cache
    Cache(cache::Args),

    /// Manage named Steel browser profiles
    Profile {
        #[command(subcommand)]
        command: profile::Command,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    crate::util::output::set_json_mode(cli.json);
    crate::util::api::init(cli.local, cli.api_url);

    match cli.command {
        Command::Daemon { session_name } => {
            let params_json = std::env::var("STEEL_DAEMON_PARAMS")
                .map_err(|_| anyhow::anyhow!("Missing STEEL_DAEMON_PARAMS"))?;
            let params: crate::browser::daemon::protocol::DaemonCreateParams =
                serde_json::from_str(&params_json)
                    .map_err(|e| anyhow::anyhow!("Invalid STEEL_DAEMON_PARAMS: {e}"))?;
            crate::browser::daemon::server::run(session_name, params).await
        }
        Command::Scrape(args) => scrape::run(args).await,
        Command::Screenshot(args) => screenshot::run(args).await,
        Command::Pdf(args) => pdf::run(args).await,
        Command::Browser(args) => browser::run(args).await,
        Command::Login(args) => login::run(args).await,
        Command::Logout(args) => logout::run(args).await,
        Command::Credentials { command } => credentials::run(command).await,
        Command::Dev { command } => dev::run(command).await,
        Command::Forge(args) => forge::run(args).await,
        Command::Config(args) => config::run(args).await,
        Command::Update(args) => update::run(args).await,
        Command::Cache(args) => cache::run(args).await,
        Command::Profile { command } => profile::run(command).await,
    }
}
