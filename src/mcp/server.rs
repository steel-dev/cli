//! MCP server: tool definitions that wrap `DaemonCommand` and route through
//! `SessionMap`. The shape mirrors `microsoft/playwright-mcp` so any LLM
//! already trained on that vocabulary works out-of-the-box.
//!
//! Conventions:
//!
//! - Every tool accepts an optional `session_id`. If omitted, the default
//!   session (`mcp-default`) is auto-created on first use.
//! - Return shape: `Content::text(stringified JSON)` for structured results,
//!   accessibility snapshots, etc. Screenshots are returned as base64 images.
//! - Errors are wrapped as `McpError::internal_error` so clients see them as
//!   tool-call failures rather than transport errors.

use std::collections::HashMap;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::browser::daemon::protocol::DaemonCommand;
use crate::mcp::session::{CreateOptions, SessionMap};

#[derive(Clone)]
pub struct SteelMcp {
    sessions: SessionMap,
    /// Populated by the `#[tool_router]` macro and consumed via the
    /// `#[tool_handler]` macro on `ServerHandler`. Static analysis can't see
    /// the macro-generated read, so the field is flagged as dead.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl Default for SteelMcp {
    fn default() -> Self {
        Self::new()
    }
}

// ── Param schemas ──────────────────────────────────────────────────

/// Shared shape: every browser tool can target a specific session.
fn opt_session(p: &Option<String>) -> Option<&str> {
    p.as_deref()
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct SessionCreateParams {
    /// Custom session name. If omitted, a unique `mcp-<uuid>` is generated.
    #[serde(default)]
    pub name: Option<String>,
    /// Enable stealth mode (humanize + auto CAPTCHA).
    #[serde(default)]
    pub stealth: bool,
    /// Use a residential proxy.
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Session timeout in milliseconds.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Region for session execution.
    #[serde(default)]
    pub region: Option<String>,
    /// Enable Steel CAPTCHA solving.
    #[serde(default)]
    pub solve_captcha: bool,
    /// Attach a named profile.
    #[serde(default)]
    pub profile_id: Option<String>,
    /// Save state back to profile on release.
    #[serde(default)]
    pub persist_profile: bool,
    /// Inject stored credentials from this namespace.
    #[serde(default)]
    pub namespace: Option<String>,
    /// Inject stored credentials.
    #[serde(default)]
    pub credentials: bool,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct SessionIdParams {
    pub session_id: String,
}

#[derive(Deserialize, JsonSchema, Debug, Default)]
pub struct OptionalSession {
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct NavigateParams {
    /// URL to navigate to.
    pub url: String,
    /// Session id. Uses default session if omitted.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Wait condition: load | domcontentloaded | networkidle.
    #[serde(default)]
    pub wait_until: Option<String>,
    /// Extra request headers.
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct ClickParams {
    /// Element selector (CSS) or accessibility ref like `@e3`.
    pub selector: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Mouse button: left | right | middle.
    #[serde(default)]
    pub button: Option<String>,
    /// Click count (2 for double-click).
    #[serde(default)]
    pub count: Option<i32>,
    /// Open the link in a new tab.
    #[serde(default)]
    pub new_tab: bool,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct FillParams {
    /// Element selector or ref.
    pub selector: String,
    /// Value to fill (replaces existing).
    pub value: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct TypeParams {
    /// Element selector or ref.
    pub selector: String,
    /// Text to type.
    pub text: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Clear the field before typing.
    #[serde(default)]
    pub clear: bool,
    /// Delay between keystrokes in milliseconds.
    #[serde(default)]
    pub delay: Option<u64>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct PressParams {
    /// Key to press (e.g. Enter, Escape, Tab, Control+a).
    pub key: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct SelectorParams {
    pub selector: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct SelectParams {
    pub selector: String,
    pub values: Vec<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct ScrollParams {
    /// Scroll direction: up | down | left | right.
    #[serde(default)]
    pub direction: Option<String>,
    /// Pixel amount (default 300).
    #[serde(default)]
    pub amount: Option<f64>,
    /// Optional element selector to scroll within.
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct DragParams {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug, Default)]
pub struct SnapshotParams {
    /// Only show interactive elements.
    #[serde(default)]
    pub interactive: bool,
    /// Restrict to a subtree.
    #[serde(default)]
    pub selector: Option<String>,
    /// Compact output.
    #[serde(default)]
    pub compact: bool,
    /// Maximum nesting depth.
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Include URLs in the snapshot.
    #[serde(default)]
    pub urls: bool,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug, Default)]
pub struct ScreenshotParams {
    #[serde(default)]
    pub full_page: bool,
    #[serde(default)]
    pub selector: Option<String>,
    /// Image format: png | jpeg | webp.
    #[serde(default)]
    pub format: Option<String>,
    /// JPEG/WebP quality (0-100).
    #[serde(default)]
    pub quality: Option<i32>,
    /// Annotate interactive elements.
    #[serde(default)]
    pub annotate: bool,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug)]
pub struct EvalParams {
    /// JavaScript expression to evaluate.
    pub script: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug, Default)]
pub struct WaitParams {
    /// Timeout in milliseconds (default 30000).
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Wait for this text to appear on the page.
    #[serde(default)]
    pub text: Option<String>,
    /// Wait for this selector.
    #[serde(default)]
    pub selector: Option<String>,
    /// Element state: visible | hidden | attached | detached.
    #[serde(default)]
    pub state: Option<String>,
    /// Wait for URL to contain this substring.
    #[serde(default)]
    pub url: Option<String>,
    /// Wait for this JS expression to evaluate truthy.
    #[serde(default)]
    pub function: Option<String>,
    /// Page load state: load | domcontentloaded | networkidle.
    #[serde(default)]
    pub load_state: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug, Default)]
pub struct TabNewParams {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug, Default)]
pub struct TabCloseParams {
    /// Tab index (closes active tab if omitted).
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub session_id: Option<String>,
}

// ── Tool router ────────────────────────────────────────────────────

#[tool_router]
impl SteelMcp {
    pub fn new() -> Self {
        Self {
            sessions: SessionMap::new(),
            tool_router: Self::tool_router(),
        }
    }

    // ── Session lifecycle (hybrid) ────────────────────────────────

    #[tool(
        description = "Create a new Steel browser session. Returns sessionId and live-view URL. Optional: pass stealth/proxy/region/profile_id/credentials to configure the session. If you just want to drive a browser, you can skip this — the default session is auto-created on first use."
    )]
    pub async fn session_create(
        &self,
        Parameters(p): Parameters<SessionCreateParams>,
    ) -> Result<CallToolResult, McpError> {
        let opts = CreateOptions {
            stealth: p.stealth,
            proxy_url: p.proxy_url,
            timeout_ms: p.timeout_ms,
            region: p.region,
            solve_captcha: p.solve_captcha,
            profile_id: p.profile_id,
            persist_profile: p.persist_profile,
            namespace: p.namespace,
            credentials: p.credentials,
        };
        let session_id = self
            .sessions
            .create(p.name, opts)
            .await
            .map_err(internal_error)?;
        // Fetch viewer_url from the daemon.
        let info = self
            .sessions
            .send(Some(&session_id), DaemonCommand::GetSessionInfo)
            .await
            .map_err(internal_error)?;
        let viewer_url = info
            .get("viewer_url")
            .and_then(|v| v.as_str())
            .map(String::from);
        let payload = serde_json::json!({
            "sessionId": session_id,
            "liveViewUrl": viewer_url,
        });
        Ok(success_json(&payload))
    }

    #[tool(description = "Release a Steel browser session.")]
    pub async fn session_release(
        &self,
        Parameters(p): Parameters<SessionIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.sessions
            .release(&p.session_id)
            .await
            .map_err(internal_error)?;
        Ok(success_json(
            &serde_json::json!({ "released": p.session_id }),
        ))
    }

    #[tool(description = "List all Steel browser sessions tracked by this server.")]
    pub async fn session_list(&self) -> Result<CallToolResult, McpError> {
        let ids = self.sessions.list().await;
        Ok(success_json(&serde_json::json!({ "sessions": ids })))
    }

    // ── Navigation ────────────────────────────────────────────────

    #[tool(description = "Navigate the browser to a URL.")]
    pub async fn navigate(
        &self,
        Parameters(p): Parameters<NavigateParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Navigate {
            url: p.url,
            wait_until: p.wait_until,
            headers: p.headers,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Navigate back in history.")]
    pub async fn back(
        &self,
        Parameters(p): Parameters<OptionalSession>,
    ) -> Result<CallToolResult, McpError> {
        self.run(opt_session(&p.session_id), DaemonCommand::Back)
            .await
    }

    #[tool(description = "Navigate forward in history.")]
    pub async fn forward(
        &self,
        Parameters(p): Parameters<OptionalSession>,
    ) -> Result<CallToolResult, McpError> {
        self.run(opt_session(&p.session_id), DaemonCommand::Forward)
            .await
    }

    #[tool(description = "Reload the current page.")]
    pub async fn reload(
        &self,
        Parameters(p): Parameters<OptionalSession>,
    ) -> Result<CallToolResult, McpError> {
        self.run(opt_session(&p.session_id), DaemonCommand::Reload)
            .await
    }

    // ── Interaction ───────────────────────────────────────────────

    #[tool(description = "Click an element by CSS selector or accessibility ref (e.g. @e3).")]
    pub async fn click(
        &self,
        Parameters(p): Parameters<ClickParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Click {
            selector: p.selector,
            button: p.button,
            click_count: p.count,
            new_tab: p.new_tab,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Fill an input field (clears existing value, then sets the new one).")]
    pub async fn fill(
        &self,
        Parameters(p): Parameters<FillParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Fill {
            selector: p.selector,
            value: p.value,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(
        description = "Type text character-by-character, simulating real keystrokes. Use `fill` for fast value setting; use `type` when keystroke events matter."
    )]
    pub async fn type_text(
        &self,
        Parameters(p): Parameters<TypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::TypeText {
            selector: p.selector,
            text: p.text,
            clear: p.clear,
            delay_ms: p.delay,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Press a keyboard key (e.g. Enter, Escape, Tab, Control+a).")]
    pub async fn press(
        &self,
        Parameters(p): Parameters<PressParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Press { key: p.key };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Hover over an element.")]
    pub async fn hover(
        &self,
        Parameters(p): Parameters<SelectorParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Hover {
            selector: p.selector,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Select option(s) from a dropdown.")]
    pub async fn select(
        &self,
        Parameters(p): Parameters<SelectParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Select {
            selector: p.selector,
            values: p.values,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Scroll the page or a specific element.")]
    pub async fn scroll(
        &self,
        Parameters(p): Parameters<ScrollParams>,
    ) -> Result<CallToolResult, McpError> {
        let dir = p.direction.unwrap_or_else(|| "down".to_string());
        let amount = p.amount.unwrap_or(300.0);
        let (dx, dy) = match dir.as_str() {
            "up" => (0.0, -amount),
            "down" => (0.0, amount),
            "left" => (-amount, 0.0),
            "right" => (amount, 0.0),
            other => {
                return Err(McpError::invalid_params(
                    format!("invalid direction '{other}' (expected up|down|left|right)"),
                    None,
                ));
            }
        };
        let cmd = DaemonCommand::Scroll {
            selector: p.selector,
            delta_x: dx,
            delta_y: dy,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Drag and drop from one element to another.")]
    pub async fn drag(
        &self,
        Parameters(p): Parameters<DragParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Drag {
            source: p.source,
            target: p.target,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    // ── Observation ───────────────────────────────────────────────

    #[tool(
        description = "Take an accessibility-tree snapshot of the current page. Returns a structured, LLM-friendly text representation with refs like `@e3` you can pass back to click/type/etc. Use `interactive: true` to get only actionable elements."
    )]
    pub async fn snapshot(
        &self,
        Parameters(p): Parameters<SnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Snapshot {
            interactive_only: p.interactive,
            selector: p.selector,
            compact: p.compact,
            max_depth: p.max_depth,
            urls: p.urls,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Take a screenshot. Returns base64 PNG by default.")]
    pub async fn screenshot(
        &self,
        Parameters(p): Parameters<ScreenshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Screenshot {
            full_page: p.full_page,
            selector: p.selector,
            format: p.format.clone(),
            quality: p.quality,
            annotate: p.annotate,
            path: None,
            screenshot_dir: None,
        };
        let result = self
            .sessions
            .send(opt_session(&p.session_id), cmd)
            .await
            .map_err(internal_error)?;

        // Prefer image content if the daemon returned base64 data.
        if let Some(b64) = result.get("base64").and_then(|v| v.as_str()) {
            let mime = result
                .get("mimeType")
                .and_then(|v| v.as_str())
                .unwrap_or("image/png");
            return Ok(CallToolResult::success(vec![Content::image(
                b64.to_string(),
                mime.to_string(),
            )]));
        }
        Ok(success_json(&result))
    }

    #[tool(description = "Evaluate a JavaScript expression in the page. Returns the value.")]
    pub async fn evaluate(
        &self,
        Parameters(p): Parameters<EvalParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Eval { script: p.script };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(
        description = "Wait for a condition: text to appear, selector to match a given state, URL substring, or a JS expression to become truthy. With no condition set, sleeps for `timeout_ms`."
    )]
    pub async fn wait(
        &self,
        Parameters(p): Parameters<WaitParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::Wait {
            timeout: p.timeout_ms,
            text: p.text,
            selector: p.selector,
            state: p.state,
            url: p.url,
            function: p.function,
            load_state: p.load_state,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    // ── State queries ─────────────────────────────────────────────

    #[tool(description = "Get the text content of an element.")]
    pub async fn get_text(
        &self,
        Parameters(p): Parameters<SelectorParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::GetText {
            selector: p.selector,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Get the current page URL.")]
    pub async fn get_url(
        &self,
        Parameters(p): Parameters<OptionalSession>,
    ) -> Result<CallToolResult, McpError> {
        self.run(opt_session(&p.session_id), DaemonCommand::Url)
            .await
    }

    #[tool(description = "Check if an element is visible.")]
    pub async fn is_visible(
        &self,
        Parameters(p): Parameters<SelectorParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::IsVisible {
            selector: p.selector,
        };
        self.run(opt_session(&p.session_id), cmd).await
    }

    // ── Tabs ──────────────────────────────────────────────────────

    #[tool(description = "List open tabs.")]
    pub async fn tab_list(
        &self,
        Parameters(p): Parameters<OptionalSession>,
    ) -> Result<CallToolResult, McpError> {
        self.run(opt_session(&p.session_id), DaemonCommand::TabList)
            .await
    }

    #[tool(description = "Open a new tab, optionally navigating to a URL.")]
    pub async fn tab_new(
        &self,
        Parameters(p): Parameters<TabNewParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::TabNew { url: p.url };
        self.run(opt_session(&p.session_id), cmd).await
    }

    #[tool(description = "Close a tab by index (closes the active tab if no index is given).")]
    pub async fn tab_close(
        &self,
        Parameters(p): Parameters<TabCloseParams>,
    ) -> Result<CallToolResult, McpError> {
        let cmd = DaemonCommand::TabClose { index: p.index };
        self.run(opt_session(&p.session_id), cmd).await
    }

    // ── Private helpers ───────────────────────────────────────────

    /// Run a `DaemonCommand` and wrap the response.
    async fn run(
        &self,
        session_id: Option<&str>,
        cmd: DaemonCommand,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .sessions
            .send(session_id, cmd)
            .await
            .map_err(internal_error)?;
        Ok(value_to_result(&result))
    }
}

#[tool_handler]
impl ServerHandler for SteelMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.protocol_version = ProtocolVersion::V_2024_11_05;
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = Implementation::new("steel-mcp", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Steel cloud browser tools. The default session is auto-created on first use; \
             call session_create explicitly when you need a named session, proxy, profile, or stealth mode. \
             Prefer `snapshot` (accessibility tree) over `screenshot` (it's 10-100x faster) and gives refs \
             like @e3 you can pass back to click/type/fill."
                .to_string(),
        );
        info
    }
}

// ── Result helpers ─────────────────────────────────────────────────

fn internal_error(e: anyhow::Error) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

fn success_json(v: &Value) -> CallToolResult {
    let text = serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string());
    CallToolResult::success(vec![Content::text(text)])
}

/// Map a `serde_json::Value` returned by the daemon into a `CallToolResult`.
/// Strings come back as plain text; everything else is pretty-printed JSON.
fn value_to_result(v: &Value) -> CallToolResult {
    match v {
        Value::String(s) => CallToolResult::success(vec![Content::text(s.clone())]),
        Value::Null => CallToolResult::success(vec![Content::text("ok".to_string())]),
        other => success_json(other),
    }
}
