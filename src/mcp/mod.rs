//! MCP (Model Context Protocol) server for Steel.
//!
//! Exposes the browser primitives that already power `steel browser <action>`
//! over the MCP wire protocol so any MCP client can drive a Steel
//! cloud browser as a tool.
//!
//! Two transports are supported:
//!
//! - **stdio** — `steel mcp`. The MCP client spawns the binary as a
//!   subprocess and talks JSON-RPC over stdin/stdout. This is the dominant
//!   pattern for desktop clients (Claude Desktop, Cursor, …).
//! - **streamable HTTP** — `steel mcp --http --port 3000`. Mounts the MCP
//!   service at `/mcp` on an axum router so remote clients can connect over
//!   HTTP. This is what powers the hosted `mcp.steel.dev` endpoint.
//!
//! Tools are thin wrappers around `DaemonCommand` — the same wire format used
//! by `steel browser`. Each MCP "session" maps 1:1 to a `steel browser`
//! daemon, so the lifecycle plumbing (Steel session creation, CDP connect,
//! health checks, expiry tracking) is reused as-is.

pub mod server;
pub mod session;
pub mod transport;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct McpArgs {
    /// Run as an HTTP server instead of stdio.
    /// Mount path is `/mcp`. Default port is 3000.
    #[arg(long)]
    pub http: bool,

    /// Port to bind when `--http` is set.
    #[arg(long, default_value_t = 3000)]
    pub port: u16,

    /// Bind address when `--http` is set.
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,
}

pub async fn run(args: McpArgs) -> anyhow::Result<()> {
    if args.http {
        transport::run_http(&args.bind, args.port).await
    } else {
        transport::run_stdio().await
    }
}
